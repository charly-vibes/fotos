/// Fotos — main application entry point.
/// Initializes modules, wires event listeners, and manages global state.

import { store } from './state.js';
import { CanvasEngine } from './canvas/engine.js';
import { History, DeleteCommand } from './canvas/history.js';
import { AddAnnotationCommand, CropCommand, TransformAnnotationCommand } from './canvas/commands.js';
import { SelectionManager } from './canvas/selection.js';
import { initToolbar } from './ui/toolbar.js';
import { initColorPicker } from './ui/color-picker.js';
import { initAiPanel } from './ui/ai-panel.js';
import { ping, takeScreenshot, cropImage, runOcr, saveImage, compositeImage, showSaveDialog, exportAnnotations, importAnnotations } from './tauri-bridge.js';
import { RegionPicker } from './ui/region-picker.js';

let messageTimeout = null;
let toastTimeout = null;

function setStatusMessage(message, autoClear = true) {
  const statusMsg = document.getElementById('status-message');
  statusMsg.textContent = message;
  if (messageTimeout) clearTimeout(messageTimeout);
  if (autoClear) {
    messageTimeout = setTimeout(() => { statusMsg.textContent = ''; }, 4000);
  }
}

function showToast(message, type = 'success') {
  const toast = document.getElementById('toast');
  if (toastTimeout) clearTimeout(toastTimeout);
  toast.textContent = message;
  toast.className = type;
  // Force reflow so transition fires even if toast was already visible.
  void toast.offsetWidth;
  toast.classList.add('show');
  toastTimeout = setTimeout(() => { toast.classList.remove('show'); }, 2800);
}

function base64PngToBlob(b64) {
  const bytes = atob(b64);
  const arr = new Uint8Array(bytes.length);
  for (let i = 0; i < bytes.length; i++) arr[i] = bytes.charCodeAt(i);
  return new Blob([arr], { type: 'image/png' });
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
  if (!window.__TAURI__) {
    document.body.innerHTML = '<pre style="color:red;padding:20px;font-size:14px">Fotos failed to start: window.__TAURI__ is not defined.\nThe Tauri IPC bridge was not injected — this usually means the app was\nbuilt incorrectly or the frontend is being served outside the Tauri context.</pre>';
    return;
  }

  const { listen } = window.__TAURI__.event;

  // Wire custom titlebar controls
  const appWindow = window.__TAURI__.window.getCurrentWindow();
  document.getElementById('btn-minimize').onclick = () => appWindow.minimize();
  document.getElementById('btn-maximize').onclick = () => appWindow.toggleMaximize();
  document.getElementById('btn-close').onclick = () => appWindow.close();

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
  initColorPicker(store);
  initAiPanel(store);

  // Track the current image data URL so crop undo can reload it.
  let currentImageDataUrl = null;

  async function loadImageAndUpdate(dataUrl, imageId) {
    currentImageDataUrl = dataUrl;
    const { width, height } = await engine.loadImage(dataUrl);
    store.set('currentImageId', imageId);
    document.getElementById('status-dimensions').textContent = `${width}×${height}`;
    // Auto-fit the new image to the viewport.
    updateZoomStatus(engine.fitToPage());
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
    // Clear selection handles when leaving select tool.
    if (tool !== 'select') {
      isSelectDragging = false;
      selectOrigAnnotation = null;
      engine.renderHandles(null);
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
        await doSave();
        break;

      case 'save-as':
        await doSaveAs();
        break;

      case 'capture-region':
        await doCaptureRegion();
        break;

      case 'copy-clipboard':
        if (!store.get('currentImageId')) { setStatusMessage('No image loaded', false); return; }
        try {
          setStatusMessage('Copying to clipboard...', false);
          // Call clipboard.write() before any await so the user-activation token
          // (from the click event) is still valid.  The image is resolved async
          // inside the ClipboardItem Promise.
          const imagePromise = compositeImage(store.get('currentImageId'), store.get('annotations') || [])
            .then(base64PngToBlob);
          await navigator.clipboard.write([new ClipboardItem({ 'image/png': imagePromise })]);
          setStatusMessage('');
          showToast('Copied to clipboard');
        } catch (error) {
          setStatusMessage('');
          showToast(`Copy failed: ${error}`, 'error');
        }
        break;

      case 'zoom-fit':
        updateZoomStatus(engine.fitToPage());
        break;

      case 'zoom-100': {
        const cont = document.getElementById('canvas-container');
        engine.setZoomAndPan(1.0,
          (cont.clientWidth - engine.imageWidth) / 2,
          (cont.clientHeight - engine.imageHeight) / 2,
        );
        updateZoomStatus(1.0);
        break;
      }

      case 'zoom-in':
        engine.setZoom(engine.getZoom() * 1.25);
        updateZoomStatus(engine.getZoom());
        break;

      case 'zoom-out':
        engine.setZoom(engine.getZoom() / 1.25);
        updateZoomStatus(engine.getZoom());
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

  // Accepts a pre-captured { id, data_url } object and shows the region picker.
  async function startRegionPickerWithCapture(result) {
    // WebKitGTK does not support fetch() for data: URLs; decode directly.
    const [header, base64] = result.data_url.split(',', 2);
    const mimeType = header.match(/:(.*?);/)[1];
    const bytes = atob(base64);
    const array = new Uint8Array(bytes.length);
    for (let i = 0; i < bytes.length; i++) array[i] = bytes.charCodeAt(i);
    const blob = new Blob([array], { type: mimeType });
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
  }

  // Existing flow (called by button + local shortcut): capture then show picker.
  async function doCaptureRegion() {
    try {
      const result = await takeScreenshot('fullscreen');
      await startRegionPickerWithCapture(result);
    } catch (error) {
      setStatusMessage(`Capture failed: ${error}`, false);
    }
  }

  async function doSave() {
    const currentImageId = store.get('currentImageId');
    if (!currentImageId) { setStatusMessage('No image to save', false); return; }
    try {
      setStatusMessage('Saving...', false);
      const savedPath = await saveImage(currentImageId, store.get('annotations'), 'png', '');
      setStatusMessage('');
      showToast(`Saved to ${savedPath}`);
    } catch (error) {
      setStatusMessage('');
      showToast(`Save failed: ${error}`, 'error');
    }
  }

  async function doSaveAs() {
    const currentImageId = store.get('currentImageId');
    if (!currentImageId) { setStatusMessage('No image to save', false); return; }
    const path = await showSaveDialog({
      filters: [{ name: 'PNG Image', extensions: ['png'] }],
      defaultPath: 'screenshot.png',
    });
    if (!path) return; // cancelled
    try {
      setStatusMessage('Saving...', false);
      const savedPath = await saveImage(currentImageId, store.get('annotations'), 'png', path);
      setStatusMessage('');
      showToast(`Saved to ${savedPath}`);
    } catch (error) {
      setStatusMessage('');
      showToast(`Save failed: ${error}`, 'error');
    }
  }

  async function doExportAnnotations() {
    const currentImageId = store.get('currentImageId');
    if (!currentImageId) { setStatusMessage('No image loaded', false); return; }
    const annotations = store.get('annotations');
    if (!annotations.length) { setStatusMessage('No annotations to export', false); return; }
    try {
      const savedPath = await exportAnnotations(currentImageId, annotations);
      showToast(`Annotations exported to ${savedPath}`);
    } catch (error) {
      if (String(error) !== 'cancelled') {
        showToast(`Export failed: ${error}`, 'error');
      }
    }
  }

  async function doImportAnnotations() {
    try {
      const imported = await importAnnotations();
      store.set('annotations', imported);
      engine.render(imported);
      showToast(`Imported ${imported.length} annotation${imported.length !== 1 ? 's' : ''}`);
    } catch (error) {
      if (String(error) !== 'cancelled') {
        showToast(`Import failed: ${error}`, 'error');
      }
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
    // Ctrl+Shift+S — region capture
    if (e.ctrlKey && e.shiftKey && e.key === 'S') {
      e.preventDefault();
      await doCaptureRegion();
      return;
    }

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

    // Ctrl+E — export annotations to JSON
    if (e.ctrlKey && !e.shiftKey && e.key === 'e') {
      e.preventDefault();
      await doExportAnnotations();
      return;
    }

    // Ctrl+Shift+E — import annotations from JSON
    if (e.ctrlKey && e.shiftKey && e.key === 'E') {
      e.preventDefault();
      await doImportAnnotations();
      return;
    }

    // Ctrl+0 — fit to page
    if (e.ctrlKey && e.key === '0') {
      e.preventDefault();
      updateZoomStatus(engine.fitToPage());
      return;
    }

    // Ctrl+1 — 100% zoom, image centered
    if (e.ctrlKey && e.key === '1') {
      e.preventDefault();
      const cw = document.getElementById('canvas-container').clientWidth;
      const ch = document.getElementById('canvas-container').clientHeight;
      engine.setZoomAndPan(1.0,
        (cw - engine.imageWidth) / 2,
        (ch - engine.imageHeight) / 2,
      );
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

    // Escape — cancel select drag or deselect
    if (e.key === 'Escape' && store.get('activeTool') === 'select') {
      e.preventDefault();
      isSelectDragging = false;
      selectOrigAnnotation = null;
      selectDragStartImg = null;
      selectionManager.deselect();
      refreshSelectionUI(null);
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
  // Use setZoomAndPan() to update both in one render call (avoids artifact traces).
  container.addEventListener('wheel', (e) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 1 / 1.1;
    const oldZoom = engine.getZoom();
    const newZoom = Math.max(0.1, Math.min(10.0, oldZoom * factor));
    const pan = engine.getPan();
    const newPanX = e.offsetX - (e.offsetX - pan.x) * (newZoom / oldZoom);
    const newPanY = e.offsetY - (e.offsetY - pan.y) * (newZoom / oldZoom);
    engine.setZoomAndPan(newZoom, newPanX, newPanY);
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

  // Select tool drag state.
  let isSelectDragging = false;
  let selectDragType = null;      // 'move' | 'resize'
  let selectDragHandleId = null;  // handle id for resize
  let selectDragStartImg = null;  // {x, y} image coords at drag start
  let selectOrigAnnotation = null; // annotation snapshot at drag start

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

    // Select tool.
    if (tool === 'select') {
      const selected = selectionManager.selected;

      // 1. Check if click is on a resize/endpoint handle of the current selection.
      if (selected) {
        const handle = selectionManager.hitTestHandle(
          imgPt.x, imgPt.y, selected, engine.handleHitRadius(),
        );
        if (handle) {
          isSelectDragging = true;
          selectDragType = 'resize';
          selectDragHandleId = handle.id;
          selectDragStartImg = imgPt;
          selectOrigAnnotation = { ...selected, points: selected.points ? selected.points.map(p => ({ ...p })) : [] };
          return;
        }
      }

      // 2. Hit-test all annotations.
      const hitResult = selectionManager.hitTest(imgPt.x, imgPt.y, store.get('annotations'));
      if (hitResult) {
        selectionManager.select(hitResult.annotation);
        refreshSelectionUI(hitResult.annotation);
        // Start a move drag immediately.
        isSelectDragging = true;
        selectDragType = 'move';
        selectDragStartImg = imgPt;
        selectOrigAnnotation = { ...hitResult.annotation, points: hitResult.annotation.points ? hitResult.annotation.points.map(p => ({ ...p })) : [] };
      } else {
        selectionManager.deselect();
        refreshSelectionUI(null);
        isSelectDragging = false;
      }
      return;
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

    // Select tool — update cursor and show handles at drag position.
    if (tool === 'select') {
      if (isSelectDragging && selectOrigAnnotation && selectDragStartImg) {
        const dx = imgPt.x - selectDragStartImg.x;
        const dy = imgPt.y - selectDragStartImg.y;
        const preview = selectDragType === 'move'
          ? selectionManager.applyMove(selectOrigAnnotation, dx, dy)
          : selectionManager.applyResize(selectOrigAnnotation, selectDragHandleId, dx, dy);
        engine.renderHandles(selectionManager.getHandles(preview));
        return;
      }

      // Hovering — update cursor to reflect what the pointer is over.
      const selected = selectionManager.selected;
      if (selected) {
        const handle = selectionManager.hitTestHandle(
          imgPt.x, imgPt.y, selected, engine.handleHitRadius(),
        );
        if (handle) {
          activeCanvas.style.cursor = handle.cursor;
          return;
        }
        if (selectionManager.hitTest(imgPt.x, imgPt.y, [selected])) {
          activeCanvas.style.cursor = 'move';
          return;
        }
      }
      activeCanvas.style.cursor = 'default';
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

    // Select tool drag — commit move/resize to history on mouseup.
    if (isSelectDragging && selectOrigAnnotation && selectDragStartImg) {
      isSelectDragging = false;
      const dx = imgPt.x - selectDragStartImg.x;
      const dy = imgPt.y - selectDragStartImg.y;
      selectDragStartImg = null;

      // Only record if there was meaningful movement.
      if (Math.abs(dx) > 0.5 || Math.abs(dy) > 0.5) {
        const finalAnnotation = selectDragType === 'move'
          ? selectionManager.applyMove(selectOrigAnnotation, dx, dy)
          : selectionManager.applyResize(selectOrigAnnotation, selectDragHandleId, dx, dy);
        const cmd = new TransformAnnotationCommand(selectOrigAnnotation, finalAnnotation);
        const newAnnotations = history.execute(cmd, store.get('annotations'));
        store.set('annotations', newAnnotations);
        selectionManager.updateSelected(finalAnnotation);
        refreshSelectionUI(finalAnnotation);
      }
      selectOrigAnnotation = null;
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

  // Refresh selection indicator + handles after any annotation/selection change.
  function refreshSelectionUI(annotation) {
    const annotations = store.get('annotations');
    engine.renderAnnotations(annotations, annotation);
    engine.renderHandles(annotation ? selectionManager.getHandles(annotation) : null);
  }

  // Re-render annotations when state changes.
  store.on('annotations', (annotations) => {
    const sel = selectionManager.selected;
    if (sel) {
      // Keep the selected reference up-to-date (e.g. after undo/redo replaces objects)
      const updated = annotations.find(a => a.id === sel.id);
      if (updated) {
        selectionManager.updateSelected(updated);
        engine.renderAnnotations(annotations, updated);
        engine.renderHandles(selectionManager.getHandles(updated));
      } else {
        selectionManager.deselect();
        engine.renderAnnotations(annotations, null);
        engine.renderHandles(null);
      }
    } else {
      engine.renderAnnotations(annotations, null);
    }
  });

  // Listen for screenshot-ready events (for future async portal flow).
  await listen('screenshot-ready', (event) => {
    console.log('Screenshot ready event:', event.payload);
  });

  // Global shortcut: Ctrl+Shift+S — Rust hid window, captured, now show region picker.
  await listen('global-capture-region', async (event) => {
    const payload = event.payload;
    if (payload.error) { setStatusMessage(`Capture failed: ${payload.error}`, false); return; }
    try { await startRegionPickerWithCapture(payload); }
    catch (err) { setStatusMessage(`Region picker failed: ${err}`, false); }
  });

  // Global shortcut: Ctrl+Shift+A — Rust hid window, captured, load directly.
  await listen('global-capture-fullscreen', async (event) => {
    const payload = event.payload;
    if (payload.error) { setStatusMessage(`Capture failed: ${payload.error}`, false); return; }
    try {
      await loadImageAndUpdate(payload.data_url, payload.id);
      store.set('annotations', []);
      setStatusMessage('Screenshot captured');
    } catch (err) { setStatusMessage(`Load failed: ${err}`, false); }
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

document.addEventListener('DOMContentLoaded', () => {
  init().catch(err => {
    document.body.innerHTML = `<pre style="color:red;padding:20px;font-size:14px">Fotos init error:\n${err?.stack ?? err}</pre>`;
  });
});
