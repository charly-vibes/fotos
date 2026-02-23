/// Fotos — main application entry point.
/// Initializes modules, wires event listeners, and manages global state.

import { store } from './state.js';
import { CanvasEngine } from './canvas/engine.js';
import { History, DeleteCommand } from './canvas/history.js';
import { AddAnnotationCommand, CropCommand } from './canvas/commands.js';
import { SelectionManager } from './canvas/selection.js';
import { initToolbar } from './ui/toolbar.js';
import { initAiPanel } from './ui/ai-panel.js';
import { ping, takeScreenshot, cropImage, runOcr, saveImage, copyToClipboard } from './tauri-bridge.js';
import { RegionPicker } from './ui/region-picker.js';

const { listen } = window.__TAURI__.event;

let messageTimeout = null;

function setStatusMessage(message, autoClear = true) {
  const statusMsg = document.getElementById('status-message');
  statusMsg.textContent = message;
  if (messageTimeout) clearTimeout(messageTimeout);
  if (autoClear) {
    messageTimeout = setTimeout(() => { statusMsg.textContent = ''; }, 4000);
  }
}

function updateZoomStatus(zoom) {
  document.getElementById('status-zoom').textContent = `${Math.round(zoom * 100)}%`;
}

// Normalize a drag rect so width/height are always positive.
function normalizeRect(x1, y1, x2, y2) {
  return {
    x: Math.min(x1, x2),
    y: Math.min(y1, y2),
    width: Math.abs(x2 - x1),
    height: Math.abs(y2 - y1),
  };
}

// Filter annotations to those intersecting cropRect and adjust coordinates.
function adjustAnnotationsForCrop(annotations, cropRect) {
  const { x: cx, y: cy, width: cw, height: ch } = cropRect;
  return annotations
    .filter(ann => {
      const b = getAnnotationBBox(ann);
      return !(b.x + b.w < cx || b.x > cx + cw || b.y + b.h < cy || b.y > cy + ch);
    })
    .map(ann => ({
      ...ann,
      x: ann.x - cx,
      y: ann.y - cy,
      points: ann.points ? ann.points.map(p => ({ x: p.x - cx, y: p.y - cy })) : ann.points,
    }));
}

function getAnnotationBBox(ann) {
  if ((ann.type === 'arrow' || ann.type === 'freehand') && ann.points?.length > 0) {
    const xs = ann.points.map(p => p.x);
    const ys = ann.points.map(p => p.y);
    const minX = Math.min(...xs), maxX = Math.max(...xs);
    const minY = Math.min(...ys), maxY = Math.max(...ys);
    return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
  }
  if (ann.type === 'step' || ann.type === 'text') {
    return { x: ann.x - 20, y: ann.y - 20, w: 40, h: 40 };
  }
  return { x: ann.x, y: ann.y, w: ann.width || 0, h: ann.height || 0 };
}

async function init() {
  // Verify Tauri IPC connection
  try {
    const response = await ping();
    console.log('Backend ping:', response);
    setStatusMessage('Backend connected');
  } catch (error) {
    console.error('Backend ping failed:', error);
    setStatusMessage('Backend connection failed', false);
  }

  const baseCanvas = document.getElementById('canvas-base');
  const annoCanvas = document.getElementById('canvas-annotations');
  const activeCanvas = document.getElementById('canvas-active');
  const container = document.getElementById('canvas-container');

  const engine = new CanvasEngine(baseCanvas, annoCanvas, activeCanvas);
  const history = new History();
  const selectionManager = new SelectionManager();
  const regionPicker = new RegionPicker();

  initToolbar(store);
  initAiPanel(store);

  // Track the current image data URL so crop undo can reload it.
  let currentImageDataUrl = null;

  async function loadImageAndUpdate(dataUrl, imageId) {
    currentImageDataUrl = dataUrl;
    const { width, height } = await engine.loadImage(dataUrl);
    store.set('currentImageId', imageId);
    document.getElementById('status-dimensions').textContent = `${width}×${height}`;
    return { width, height };
  }

  // Wire status bar and cursor to active tool.
  const TOOL_NAMES = {
    select: 'Select', arrow: 'Arrow', rect: 'Rectangle', ellipse: 'Ellipse',
    text: 'Text', blur: 'Blur', step: 'Step Number', freehand: 'Freehand',
    highlight: 'Highlight', crop: 'Crop',
  };
  const TOOL_CURSORS = {
    select: 'default', arrow: 'crosshair', rect: 'crosshair', ellipse: 'crosshair',
    text: 'text', blur: 'crosshair', step: 'cell', freehand: 'crosshair',
    highlight: 'crosshair', crop: 'crosshair',
  };

  store.on('activeTool', (tool) => {
    document.getElementById('status-tool').textContent = TOOL_NAMES[tool] || tool;
    activeCanvas.style.cursor = TOOL_CURSORS[tool] || 'default';
    // Cancel any active crop selection when switching tools.
    if (tool !== 'crop') {
      isCropDragging = false;
      cropStartImg = null;
      engine.renderCropOverlay(null);
    }
    // Commit any pending text input.
    if (pendingTextArea) pendingTextArea.blur();
  });

  // Wire data-action buttons.
  document.addEventListener('click', async (e) => {
    const action = e.target.dataset.action;
    if (!action) return;

    switch (action) {
      case 'capture-fullscreen':
        try {
          const result = await takeScreenshot('fullscreen');
          await loadImageAndUpdate(result.data_url, result.id);
          store.set('annotations', []);
          setStatusMessage('Screenshot captured');
        } catch (error) {
          setStatusMessage(`Capture failed: ${error}`, false);
        }
        break;

      case 'ocr':
        if (!store.get('currentImageId')) { setStatusMessage('No image loaded', false); return; }
        try {
          setStatusMessage('Running OCR...', false);
          const result = await runOcr(store.get('currentImageId'));
          store.set('ocrResults', result);
          setStatusMessage('OCR complete');
        } catch (error) {
          setStatusMessage(`OCR failed: ${error}`, false);
        }
        break;

      case 'undo': doUndo(); break;
      case 'redo': doRedo(); break;
      case 'save':
      case 'save-as':
        await doSave();
        break;

      case 'capture-region':
        try {
          const result = await takeScreenshot('fullscreen');
          const resp = await fetch(result.data_url);
          const blob = await resp.blob();
          const bitmap = await createImageBitmap(blob);
          regionPicker.show(bitmap, async (ix, iy, iw, ih) => {
            try {
              const cropped = await cropImage(result.id, ix, iy, iw, ih);
              await loadImageAndUpdate(cropped.data_url, cropped.id);
              store.set('annotations', []);
              setStatusMessage('Region captured');
            } catch (err) {
              setStatusMessage(`Crop failed: ${err}`, false);
            }
          }, () => { setStatusMessage('Region capture cancelled', false); });
        } catch (error) {
          setStatusMessage(`Capture failed: ${error}`, false);
        }
        break;

      case 'copy-clipboard':
        if (!store.get('currentImageId')) { setStatusMessage('No image loaded', false); return; }
        try {
          setStatusMessage('Copying to clipboard...', false);
          await copyToClipboard(store.get('currentImageId'), store.get('annotations') || []);
          setStatusMessage('Copied to clipboard');
        } catch (error) {
          setStatusMessage(`Copy failed: ${error}`, false);
        }
        break;

      case 'capture-window':
      case 'auto-blur':
      case 'ai-analyze':
        setStatusMessage(`Action '${action}' not yet implemented`, false);
        break;
    }
  });

  function doUndo() {
    if (!history.canUndo) { setStatusMessage('Nothing to undo', false); return; }
    const newAnnotations = history.undo(store.get('annotations'));
    store.set('annotations', newAnnotations);
    selectionManager.deselect();
    engine.renderAnnotations(newAnnotations, null);
    setStatusMessage('Undone');
  }

  function doRedo() {
    if (!history.canRedo) { setStatusMessage('Nothing to redo', false); return; }
    const newAnnotations = history.redo(store.get('annotations'));
    store.set('annotations', newAnnotations);
    selectionManager.deselect();
    engine.renderAnnotations(newAnnotations, null);
    setStatusMessage('Redone');
  }

  async function doSave() {
    const currentImageId = store.get('currentImageId');
    if (!currentImageId) { setStatusMessage('No image to save', false); return; }
    try {
      setStatusMessage('Saving...', false);
      const savedPath = await saveImage(currentImageId, store.get('annotations'), 'png', '');
      setStatusMessage(`Saved to ${savedPath}`);
    } catch (error) {
      setStatusMessage(`Save failed: ${error}`, false);
    }
  }

  // Apply the crop tool selection: call backend, adjust annotations, record in history.
  async function applyCrop(cropRect) {
    const imageId = store.get('currentImageId');
    if (!imageId) return;
    const annotations = store.get('annotations');
    try {
      const result = await cropImage(imageId, cropRect.x, cropRect.y, cropRect.width, cropRect.height);
      const adjustedAnnotations = adjustAnnotationsForCrop(annotations, cropRect);
      const oldDataUrl = currentImageDataUrl;

      const cropCmd = new CropCommand({
        oldImageId: imageId,
        newImageId: result.id,
        oldAnnotations: annotations,
        newAnnotations: adjustedAnnotations,
        oldDataUrl,
        newDataUrl: result.data_url,
      });

      // Restore old image+annotations on undo.
      cropCmd.onUndo = async (oldId, oldAnns, oldUrl) => {
        await loadImageAndUpdate(oldUrl, oldId);
        store.set('annotations', oldAnns);
        engine.renderAnnotations(oldAnns, null);
      };
      // Re-apply on redo.
      cropCmd.onExecute = async (newId, newAnns, newUrl) => {
        await loadImageAndUpdate(newUrl, newId);
        store.set('annotations', newAnns);
        engine.renderAnnotations(newAnns, null);
      };

      history.record(cropCmd);
      await loadImageAndUpdate(result.data_url, result.id);
      store.set('annotations', adjustedAnnotations);
      engine.renderAnnotations(adjustedAnnotations, null);
      setStatusMessage('Image cropped');
    } catch (err) {
      setStatusMessage(`Crop failed: ${err}`, false);
    }
  }

  // ── Keyboard shortcuts ──────────────────────────────────────────────────────

  let isPanning = false;

  document.addEventListener('keydown', async (e) => {
    // Ctrl+Shift+A — fullscreen capture
    if (e.ctrlKey && e.shiftKey && e.key === 'A') {
      e.preventDefault();
      try {
        const result = await takeScreenshot('fullscreen');
        await loadImageAndUpdate(result.data_url, result.id);
        store.set('annotations', []);
        setStatusMessage('Ready');
      } catch (error) {
        setStatusMessage(`Capture failed: ${error}`, false);
      }
      return;
    }

    // Ctrl+Z — undo
    if (e.ctrlKey && !e.shiftKey && e.key === 'z') {
      e.preventDefault();
      doUndo();
      return;
    }

    // Ctrl+Shift+Z — redo
    if (e.ctrlKey && e.shiftKey && e.key === 'Z') {
      e.preventDefault();
      doRedo();
      return;
    }

    // Ctrl+S — save
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault();
      await doSave();
      return;
    }

    // Ctrl+0 — reset zoom and pan
    if (e.ctrlKey && e.key === '0') {
      e.preventDefault();
      engine.setZoom(1.0);
      engine.setPan(0, 0);
      updateZoomStatus(1.0);
      return;
    }

    // +/= — zoom in
    if (!e.ctrlKey && !e.shiftKey && !e.altKey && (e.key === '+' || e.key === '=')) {
      e.preventDefault();
      engine.setZoom(engine.getZoom() * 1.25);
      updateZoomStatus(engine.getZoom());
      return;
    }

    // - — zoom out
    if (!e.ctrlKey && !e.shiftKey && !e.altKey && e.key === '-') {
      e.preventDefault();
      engine.setZoom(engine.getZoom() / 1.25);
      updateZoomStatus(engine.getZoom());
      return;
    }

    // Enter — confirm crop
    if (e.key === 'Enter' && isCropDragging && cropStartImg && cropEndImg) {
      e.preventDefault();
      const r = normalizeRect(cropStartImg.x, cropStartImg.y, cropEndImg.x, cropEndImg.y);
      isCropDragging = false;
      cropStartImg = null;
      cropEndImg = null;
      engine.renderCropOverlay(null);
      if (r.width > 2 && r.height > 2) await applyCrop(r);
      return;
    }

    // Escape — cancel crop
    if (e.key === 'Escape' && isCropDragging) {
      e.preventDefault();
      isCropDragging = false;
      cropStartImg = null;
      cropEndImg = null;
      engine.renderCropOverlay(null);
      setStatusMessage('Crop cancelled', false);
      return;
    }

    // Delete — delete selected annotation
    if (e.key === 'Delete') {
      const selected = selectionManager.selected;
      if (!selected) return;
      const annotations = store.get('annotations');
      const index = annotations.findIndex(a => a.id === selected.id);
      if (index === -1) return;
      const deleteCmd = new DeleteCommand(selected, index);
      const newAnnotations = history.execute(deleteCmd, annotations);
      store.set('annotations', newAnnotations);
      selectionManager.deselect();
      engine.renderAnnotations(newAnnotations, null);
      setStatusMessage('Annotation deleted');
      return;
    }

    // Space — start pan mode
    if (e.code === 'Space' && !e.ctrlKey && !e.altKey) {
      e.preventDefault();
      isPanning = true;
      container.style.cursor = 'grab';
    }
  });

  document.addEventListener('keyup', (e) => {
    if (e.code === 'Space') {
      isPanning = false;
      isPanDragging = false;
      container.style.cursor = '';
    }
  });

  // Mouse wheel zoom — centered on cursor position.
  container.addEventListener('wheel', (e) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 1 / 1.1;
    const oldZoom = engine.getZoom();
    const newZoom = Math.max(0.1, Math.min(10.0, oldZoom * factor));
    const pan = engine.getPan();
    const newPanX = e.offsetX - (e.offsetX - pan.x) * (newZoom / oldZoom);
    const newPanY = e.offsetY - (e.offsetY - pan.y) * (newZoom / oldZoom);
    engine.setZoom(newZoom);
    engine.setPan(newPanX, newPanY);
    updateZoomStatus(newZoom);
  }, { passive: false });

  // ── Drawing / interaction state ─────────────────────────────────────────────

  let isPanDragging = false;
  let panStartClientX, panStartClientY, panStartX, panStartY;

  // Drag-to-draw tools (rect, arrow, ellipse, blur, highlight).
  let isDrawing = false;
  let drawStartImg;

  // Freehand tool.
  let freehandPoints = [];

  // Crop tool.
  let isCropDragging = false;
  let cropStartImg = null;
  let cropEndImg = null;

  // Text tool — reference to active textarea (if any).
  let pendingTextArea = null;

  const DRAG_DRAW_TOOLS = new Set(['rect', 'arrow', 'ellipse', 'blur', 'highlight']);

  // ── Mouse down ─────────────────────────────────────────────────────────────

  activeCanvas.addEventListener('mousedown', (e) => {
    // Pan mode takes priority.
    if (isPanning) {
      isPanDragging = true;
      panStartClientX = e.clientX;
      panStartClientY = e.clientY;
      const pan = engine.getPan();
      panStartX = pan.x;
      panStartY = pan.y;
      container.style.cursor = 'grabbing';
      return;
    }

    if (!store.get('currentImageId')) return;

    const tool = store.get('activeTool');
    const imgPt = engine.screenToImage(e.offsetX, e.offsetY);

    // Drag-to-draw tools (rect, arrow, ellipse, blur, highlight).
    if (DRAG_DRAW_TOOLS.has(tool)) {
      isDrawing = true;
      drawStartImg = imgPt;
      return;
    }

    // Freehand.
    if (tool === 'freehand') {
      isDrawing = true;
      freehandPoints = [imgPt];
      return;
    }

    // Crop.
    if (tool === 'crop') {
      isCropDragging = true;
      cropStartImg = imgPt;
      cropEndImg = imgPt;
      return;
    }

    // Step — place immediately on mousedown.
    if (tool === 'step') {
      const stepNumber = store.get('nextStepNumber') ?? 1;
      const annotation = {
        id: crypto.randomUUID(),
        type: 'step',
        x: imgPt.x,
        y: imgPt.y,
        width: 0,
        height: 0,
        points: [],
        text: String(stepNumber),
        fontSize: 24,
        fontFamily: 'sans-serif',
        stepNumber,
        strokeColor: store.get('strokeColor'),
        fillColor: 'transparent',
        strokeWidth: store.get('strokeWidth'),
        opacity: store.get('opacity'),
        createdAt: new Date().toISOString(),
        locked: false,
      };
      store.set('nextStepNumber', stepNumber + 1);
      const newAnnotations = history.execute(new AddAnnotationCommand(annotation), store.get('annotations'));
      store.set('annotations', newAnnotations);
      engine.renderAnnotations(newAnnotations);
      return;
    }

    // Text — handled by click event (after press+release) to avoid WebKit blur-on-mouseup.
    if (tool === 'text') {
      if (pendingTextArea) { pendingTextArea.blur(); }
      return;
    }

    // Select / hit test.
    const hitResult = selectionManager.hitTest(imgPt.x, imgPt.y, store.get('annotations'));
    if (hitResult) {
      selectionManager.select(hitResult.annotation);
      engine.renderAnnotations(store.get('annotations'), hitResult.annotation);
    } else {
      selectionManager.deselect();
      engine.renderAnnotations(store.get('annotations'), null);
    }
  });

  // ── Text tool (click) ───────────────────────────────────────────────────────
  // Using 'click' (fires after mouseup) avoids the WebKit/GTK issue where focus()
  // called during 'mousedown' is reverted when the browser processes mouseup.

  activeCanvas.addEventListener('click', (e) => {
    if (store.get('activeTool') !== 'text') return;
    if (!store.get('currentImageId')) return;
    if (isPanning) return;
    if (pendingTextArea) return; // already committed by mousedown

    const imgPt = engine.screenToImage(e.offsetX, e.offsetY);
    const fontSize = 20;
    const ta = document.createElement('textarea');
    ta.style.cssText = [
      `position:absolute`,
      `left:${e.offsetX}px`,
      `top:${e.offsetY}px`,
      `min-width:100px`,
      `min-height:${Math.ceil(fontSize * engine.getZoom() * 1.4)}px`,
      `font-size:${fontSize * engine.getZoom()}px`,
      `font-family:sans-serif`,
      `color:${store.get('strokeColor') || '#FF0000'}`,
      `background:transparent`,
      `border:1px dashed #0066FF`,
      `outline:none`,
      `resize:both`,
      `z-index:100`,
      `padding:2px`,
    ].join(';');
    container.appendChild(ta);
    pendingTextArea = ta;
    ta.focus();

    function commitText() {
      pendingTextArea = null;
      const text = ta.value.trim();
      container.removeChild(ta);
      if (!text) return;
      const annotation = {
        id: crypto.randomUUID(),
        type: 'text',
        x: imgPt.x,
        y: imgPt.y,
        width: 0,
        height: 0,
        points: [],
        text,
        fontSize,
        fontFamily: 'sans-serif',
        strokeColor: store.get('strokeColor'),
        fillColor: 'transparent',
        strokeWidth: store.get('strokeWidth'),
        opacity: store.get('opacity'),
        createdAt: new Date().toISOString(),
        locked: false,
      };
      const newAnnotations = history.execute(new AddAnnotationCommand(annotation), store.get('annotations'));
      store.set('annotations', newAnnotations);
      engine.renderAnnotations(newAnnotations);
    }

    ta.addEventListener('blur', commitText, { once: true });
    ta.addEventListener('keydown', (ke) => {
      if (ke.key === 'Escape') { ta.value = ''; ta.blur(); }
      ke.stopPropagation(); // don't trigger app-level shortcuts
    });
  });

  // ── Mouse move ─────────────────────────────────────────────────────────────

  activeCanvas.addEventListener('mousemove', (e) => {
    if (isPanDragging) {
      const dx = e.clientX - panStartClientX;
      const dy = e.clientY - panStartClientY;
      engine.setPan(panStartX + dx, panStartY + dy);
      return;
    }

    const tool = store.get('activeTool');
    const imgPt = engine.screenToImage(e.offsetX, e.offsetY);

    if (isDrawing && DRAG_DRAW_TOOLS.has(tool)) {
      const r = normalizeRect(drawStartImg.x, drawStartImg.y, imgPt.x, imgPt.y);
      const preview = buildPreviewShape(tool, drawStartImg, imgPt, r);
      engine.renderActive(preview);
      return;
    }

    if (isDrawing && tool === 'freehand') {
      freehandPoints.push(imgPt);
      engine.renderActive({
        type: 'freehand',
        points: freehandPoints,
        strokeColor: store.get('strokeColor'),
        strokeWidth: store.get('strokeWidth'),
        opacity: store.get('opacity'),
      });
      return;
    }

    if (isCropDragging && cropStartImg) {
      cropEndImg = imgPt;
      const r = normalizeRect(cropStartImg.x, cropStartImg.y, imgPt.x, imgPt.y);
      engine.renderCropOverlay(r);
      return;
    }
  });

  // ── Mouse up ───────────────────────────────────────────────────────────────

  activeCanvas.addEventListener('mouseup', async (e) => {
    if (isPanDragging) {
      isPanDragging = false;
      container.style.cursor = isPanning ? 'grab' : '';
      return;
    }

    const tool = store.get('activeTool');
    const imgPt = engine.screenToImage(e.offsetX, e.offsetY);

    if (isDrawing && DRAG_DRAW_TOOLS.has(tool)) {
      isDrawing = false;
      const r = normalizeRect(drawStartImg.x, drawStartImg.y, imgPt.x, imgPt.y);
      engine.renderActive(null);
      if (r.width < 2 || r.height < 2) return;
      const annotation = buildCommittedShape(tool, drawStartImg, imgPt, r);
      const newAnnotations = history.execute(new AddAnnotationCommand(annotation), store.get('annotations'));
      store.set('annotations', newAnnotations);
      engine.renderAnnotations(newAnnotations);
      return;
    }

    if (isDrawing && tool === 'freehand') {
      isDrawing = false;
      engine.renderActive(null);
      if (freehandPoints.length < 2) return;
      const annotation = {
        id: crypto.randomUUID(),
        type: 'freehand',
        x: Math.min(...freehandPoints.map(p => p.x)),
        y: Math.min(...freehandPoints.map(p => p.y)),
        width: 0,
        height: 0,
        points: [...freehandPoints],
        strokeColor: store.get('strokeColor'),
        fillColor: 'transparent',
        strokeWidth: store.get('strokeWidth'),
        opacity: store.get('opacity'),
        createdAt: new Date().toISOString(),
        locked: false,
      };
      freehandPoints = [];
      const newAnnotations = history.execute(new AddAnnotationCommand(annotation), store.get('annotations'));
      store.set('annotations', newAnnotations);
      engine.renderAnnotations(newAnnotations);
      return;
    }

    // Crop drag ends on mouseup — apply immediately.
    if (isCropDragging && cropStartImg) {
      isCropDragging = false;
      const r = normalizeRect(cropStartImg.x, cropStartImg.y, imgPt.x, imgPt.y);
      cropStartImg = null;
      cropEndImg = null;
      engine.renderCropOverlay(null);
      if (r.width > 2 && r.height > 2) await applyCrop(r);
    }
  });

  // Re-render annotations when state changes.
  store.on('annotations', (annotations) => {
    engine.renderAnnotations(annotations, selectionManager.selected);
  });

  // Listen for screenshot-ready events (for future async portal flow).
  await listen('screenshot-ready', (event) => {
    console.log('Screenshot ready event:', event.payload);
  });

  console.log('Fotos initialized');
}

// ── Helpers ────────────────────────────────────────────────────────────────

// Build a preview shape object for drag-draw tools.
function buildPreviewShape(tool, startImg, endImg, r) {
  const base = {
    x: r.x, y: r.y, width: r.width, height: r.height,
    strokeColor: store.get('strokeColor'),
    fillColor: store.get('fillColor'),
    strokeWidth: store.get('strokeWidth'),
    opacity: store.get('opacity'),
  };
  switch (tool) {
    case 'rect':      return { ...base, type: 'rect' };
    case 'ellipse':   return { ...base, type: 'ellipse' };
    case 'blur':      return { ...base, type: 'blur', blurRadius: 10 };
    case 'highlight': return { ...base, type: 'highlight', highlightColor: '#FFFF00' };
    case 'arrow':
      return {
        type: 'arrow',
        points: [{ x: startImg.x, y: startImg.y }, { x: endImg.x, y: endImg.y }],
        strokeColor: store.get('strokeColor'),
        strokeWidth: store.get('strokeWidth'),
        opacity: store.get('opacity'),
      };
    default: return null;
  }
}

// Build a committed annotation object for drag-draw tools.
function buildCommittedShape(tool, startImg, endImg, r) {
  const common = {
    id: crypto.randomUUID(),
    x: r.x, y: r.y, width: r.width, height: r.height,
    points: [],
    strokeColor: store.get('strokeColor'),
    fillColor: store.get('fillColor'),
    strokeWidth: store.get('strokeWidth'),
    opacity: store.get('opacity'),
    createdAt: new Date().toISOString(),
    locked: false,
  };
  switch (tool) {
    case 'rect':      return { ...common, type: 'rect' };
    case 'ellipse':   return { ...common, type: 'ellipse' };
    case 'blur':      return { ...common, type: 'blur', blurRadius: 10 };
    case 'highlight': return { ...common, type: 'highlight', fillColor: 'transparent', highlightColor: '#FFFF00' };
    case 'arrow':
      return {
        id: common.id,
        type: 'arrow',
        x: r.x, y: r.y, width: r.width, height: r.height,
        points: [{ x: startImg.x, y: startImg.y }, { x: endImg.x, y: endImg.y }],
        strokeColor: store.get('strokeColor'),
        fillColor: 'transparent',
        strokeWidth: store.get('strokeWidth'),
        opacity: store.get('opacity'),
        createdAt: common.createdAt,
        locked: false,
      };
    default: return null;
  }
}

document.addEventListener('DOMContentLoaded', init);
