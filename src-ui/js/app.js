/// Fotos — main application entry point.
/// Initializes modules, wires event listeners, and manages global state.

import { store } from './state.js';
import { CanvasEngine } from './canvas/engine.js';
import { History, DeleteCommand } from './canvas/history.js';
import { AddAnnotationCommand } from './canvas/commands.js';
import { SelectionManager } from './canvas/selection.js';
import { initToolbar } from './ui/toolbar.js';
import { initAiPanel } from './ui/ai-panel.js';
import { ping, takeScreenshot, runOcr, saveImage } from './tauri-bridge.js';

const { listen } = window.__TAURI__.event;

let messageTimeout = null;

function setStatusMessage(message, autoClear = true) {
  const statusMsg = document.getElementById('status-message');
  statusMsg.textContent = message;

  if (messageTimeout) clearTimeout(messageTimeout);

  if (autoClear) {
    messageTimeout = setTimeout(() => {
      statusMsg.textContent = '';
    }, 4000);
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

  initToolbar(store);
  initAiPanel(store);

  // Wire status bar updates
  store.on('activeTool', (tool) => {
    const toolNames = {
      'select': 'Select',
      'arrow': 'Arrow',
      'rect': 'Rectangle',
      'ellipse': 'Ellipse',
      'text': 'Text',
      'blur': 'Blur',
      'step': 'Step Number',
      'freehand': 'Freehand',
      'highlight': 'Highlight',
      'crop': 'Crop',
    };
    document.getElementById('status-tool').textContent = toolNames[tool] || tool;
    activeCanvas.style.cursor = tool === 'rect' ? 'crosshair' : 'default';
  });

  // Wire data-action buttons
  document.addEventListener('click', async (e) => {
    const action = e.target.dataset.action;
    if (!action) return;

    switch (action) {
      case 'capture-fullscreen':
        try {
          const result = await takeScreenshot('fullscreen');
          const { width, height } = await engine.loadImage(result.data_url);
          store.set('currentImageId', result.id);
          document.getElementById('status-dimensions').textContent = `${width}×${height}`;
          setStatusMessage('Screenshot captured');
        } catch (error) {
          setStatusMessage(`Capture failed: ${error}`, false);
        }
        break;

      case 'ocr':
        if (!store.get('currentImageId')) {
          setStatusMessage('No image loaded', false);
          return;
        }
        try {
          setStatusMessage('Running OCR...', false);
          const result = await runOcr(store.get('currentImageId'));
          store.set('ocrResults', result);
          setStatusMessage('OCR complete');
        } catch (error) {
          setStatusMessage(`OCR failed: ${error}`, false);
        }
        break;

      case 'undo':
        doUndo();
        break;

      case 'redo':
        doRedo();
        break;

      case 'save':
      case 'save-as':
        await doSave();
        break;

      // Placeholder for other actions
      case 'capture-region':
      case 'capture-window':
      case 'auto-blur':
      case 'ai-analyze':
      case 'copy-clipboard':
        setStatusMessage(`Action '${action}' not yet implemented`, false);
        break;
    }
  });

  function doUndo() {
    if (!history.canUndo) {
      setStatusMessage('Nothing to undo', false);
      return;
    }
    const newAnnotations = history.undo(store.get('annotations'));
    store.set('annotations', newAnnotations);
    selectionManager.deselect();
    engine.renderAnnotations(newAnnotations, null);
    setStatusMessage('Undone');
  }

  function doRedo() {
    if (!history.canRedo) {
      setStatusMessage('Nothing to redo', false);
      return;
    }
    const newAnnotations = history.redo(store.get('annotations'));
    store.set('annotations', newAnnotations);
    selectionManager.deselect();
    engine.renderAnnotations(newAnnotations, null);
    setStatusMessage('Redone');
  }

  async function doSave() {
    const currentImageId = store.get('currentImageId');
    if (!currentImageId) {
      setStatusMessage('No image to save', false);
      return;
    }
    try {
      setStatusMessage('Saving...', false);
      const savedPath = await saveImage(currentImageId, store.get('annotations'), 'png', '');
      setStatusMessage(`Saved to ${savedPath}`);
    } catch (error) {
      setStatusMessage(`Save failed: ${error}`, false);
    }
  }

  // --- Keyboard shortcuts ---

  let isPanning = false;

  document.addEventListener('keydown', async (e) => {
    // Ctrl+Shift+A — fullscreen capture
    if (e.ctrlKey && e.shiftKey && e.key === 'A') {
      e.preventDefault();
      try {
        const result = await takeScreenshot('fullscreen');
        const { width, height } = await engine.loadImage(result.data_url);
        store.set('currentImageId', result.id);
        document.getElementById('status-dimensions').textContent = `${width}×${height}`;
        setStatusMessage('Ready');
      } catch (error) {
        console.error('Screenshot failed:', error);
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

  // Mouse wheel zoom — centered on cursor position
  container.addEventListener('wheel', (e) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 1 / 1.1;
    const oldZoom = engine.getZoom();
    const newZoom = Math.max(0.1, Math.min(10.0, oldZoom * factor));
    const pan = engine.getPan();
    // Adjust pan so the point under the cursor stays fixed
    const newPanX = e.offsetX - (e.offsetX - pan.x) * (newZoom / oldZoom);
    const newPanY = e.offsetY - (e.offsetY - pan.y) * (newZoom / oldZoom);
    engine.setZoom(newZoom);
    engine.setPan(newPanX, newPanY);
    updateZoomStatus(newZoom);
  }, { passive: false });

  // --- Pan drag state ---
  let isPanDragging = false;
  let panStartClientX, panStartClientY, panStartX, panStartY;

  // --- Rectangle draw state ---
  let isDrawing = false;
  let drawStartImg; // { x, y } in image space

  activeCanvas.addEventListener('mousedown', (e) => {
    // Pan mode takes priority
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

    if (store.get('activeTool') === 'rect') {
      isDrawing = true;
      drawStartImg = engine.screenToImage(e.offsetX, e.offsetY);
      return;
    }

    // Selection mode — hit test in image space
    const imgPt = engine.screenToImage(e.offsetX, e.offsetY);
    const hitResult = selectionManager.hitTest(imgPt.x, imgPt.y, store.get('annotations'));
    if (hitResult) {
      selectionManager.select(hitResult.annotation);
      engine.renderAnnotations(store.get('annotations'), hitResult.annotation);
    } else {
      selectionManager.deselect();
      engine.renderAnnotations(store.get('annotations'), null);
    }
  });

  activeCanvas.addEventListener('mousemove', (e) => {
    if (isPanDragging) {
      const dx = e.clientX - panStartClientX;
      const dy = e.clientY - panStartClientY;
      engine.setPan(panStartX + dx, panStartY + dy);
      return;
    }

    if (!isDrawing) return;

    const curImg = engine.screenToImage(e.offsetX, e.offsetY);
    const r = normalizeRect(drawStartImg.x, drawStartImg.y, curImg.x, curImg.y);

    engine.renderActive({
      type: 'rect',
      x: r.x,
      y: r.y,
      width: r.width,
      height: r.height,
      strokeColor: store.get('strokeColor'),
      fillColor: store.get('fillColor'),
      strokeWidth: store.get('strokeWidth'),
      opacity: store.get('opacity'),
    });
  });

  activeCanvas.addEventListener('mouseup', (e) => {
    if (isPanDragging) {
      isPanDragging = false;
      container.style.cursor = isPanning ? 'grab' : '';
      return;
    }

    if (!isDrawing) return;
    isDrawing = false;

    const curImg = engine.screenToImage(e.offsetX, e.offsetY);
    const r = normalizeRect(drawStartImg.x, drawStartImg.y, curImg.x, curImg.y);

    // Ignore accidental clicks (sub-2px)
    if (r.width < 2 || r.height < 2) {
      engine.renderActive(null);
      return;
    }

    const annotation = {
      id: crypto.randomUUID(),
      type: 'rect',
      x: r.x,
      y: r.y,
      width: r.width,
      height: r.height,
      points: [],
      strokeColor: store.get('strokeColor'),
      fillColor: store.get('fillColor'),
      strokeWidth: store.get('strokeWidth'),
      opacity: store.get('opacity'),
      createdAt: new Date().toISOString(),
      locked: false,
    };

    const newAnnotations = history.execute(new AddAnnotationCommand(annotation), store.get('annotations'));
    store.set('annotations', newAnnotations);
    engine.renderAnnotations(newAnnotations);
    engine.renderActive(null);
  });

  // Re-render annotations when state changes
  store.on('annotations', (annotations) => {
    engine.renderAnnotations(annotations, selectionManager.selected);
  });

  // Listen for screenshot-ready events (for future async portal flow)
  await listen('screenshot-ready', (event) => {
    console.log('Screenshot ready event:', event.payload);
  });

  console.log('Fotos initialized');
}

document.addEventListener('DOMContentLoaded', init);
