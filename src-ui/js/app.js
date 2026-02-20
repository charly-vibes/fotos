/// Fotos — main application entry point.
/// Initializes modules, wires event listeners, and manages global state.

import { store } from './state.js';
import { CanvasEngine } from './canvas/engine.js';
import { History, DeleteCommand } from './canvas/history.js';
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

      // Placeholder for other actions
      case 'capture-region':
      case 'capture-window':
      case 'auto-blur':
      case 'ai-analyze':
      case 'copy-clipboard':
      case 'save':
      case 'save-as':
      case 'undo':
      case 'redo':
        setStatusMessage(`Action '${action}' not yet implemented`, false);
        break;
    }
  });

  // Wire keyboard shortcuts
  document.addEventListener('keydown', async (e) => {
    // Ctrl+Shift+A for fullscreen capture (alternative to PrintScreen)
    if (e.ctrlKey && e.shiftKey && e.key === 'A') {
      e.preventDefault();
      try {
        const result = await takeScreenshot('fullscreen');
        console.log('Screenshot captured:', result.id);

        // Load image into canvas
        const { width, height } = await engine.loadImage(result.data_url);

        // Update state
        store.set('currentImageId', result.id);

        // Update status bar
        document.getElementById('status-dimensions').textContent = `${width}×${height}`;
        setStatusMessage('Ready');
      } catch (error) {
        console.error('Screenshot failed:', error);
        setStatusMessage(`Capture failed: ${error}`, false);
      }
    }

    // Delete key - delete selected annotation
    if (e.key === 'Delete') {
      const selected = selectionManager.selected;
      if (!selected) return;

      const annotations = store.get('annotations');
      const index = annotations.findIndex(a => a.id === selected.id);
      if (index === -1) return;

      // Create and execute delete command
      const deleteCmd = new DeleteCommand(selected, index);
      const newAnnotations = history.execute(deleteCmd, annotations);

      // Update state and deselect
      store.set('annotations', newAnnotations);
      selectionManager.deselect();
      engine.renderAnnotations(newAnnotations, null);
      setStatusMessage('Annotation deleted');
    }

    // Ctrl+Z - undo
    if (e.ctrlKey && e.key === 'z') {
      e.preventDefault();

      if (!history.canUndo) {
        setStatusMessage('Nothing to undo', false);
        return;
      }

      const annotations = store.get('annotations');
      const newAnnotations = history.undo(annotations);

      // Update state and deselect
      store.set('annotations', newAnnotations);
      selectionManager.deselect();
      engine.renderAnnotations(newAnnotations, null);
      setStatusMessage('Undone');
    }

    // Ctrl+S - save image
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault();

      const currentImageId = store.get('currentImageId');
      if (!currentImageId) {
        setStatusMessage('No image to save', false);
        return;
      }

      try {
        setStatusMessage('Saving...', false);
        const annotations = store.get('annotations');
        // Empty path triggers default path generation
        const savedPath = await saveImage(currentImageId, annotations, 'png', '');
        setStatusMessage(`Saved to ${savedPath}`);
      } catch (error) {
        setStatusMessage(`Save failed: ${error}`, false);
      }
    }
  });

  // Listen for screenshot-ready events (for future use)
  await listen('screenshot-ready', (event) => {
    console.log('Screenshot ready event:', event.payload);
  });

  // Wire rectangle tool mouse handlers
  let isDrawing = false;
  let startX, startY;

  activeCanvas.addEventListener('mousedown', (e) => {
    if (!store.get('currentImageId')) return;

    const rect = activeCanvas.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const clickY = e.clientY - rect.top;

    // Rectangle drawing mode
    if (store.get('activeTool') === 'rect') {
      isDrawing = true;
      startX = clickX;
      startY = clickY;
      return;
    }

    // Selection mode - hit test for annotations
    const hitResult = selectionManager.hitTest(clickX, clickY, store.get('annotations'));
    if (hitResult) {
      selectionManager.select(hitResult.annotation);
      // Re-render with selection indicator
      engine.renderAnnotations(store.get('annotations'), hitResult.annotation);
    } else {
      selectionManager.deselect();
      // Re-render without selection indicator
      engine.renderAnnotations(store.get('annotations'), null);
    }
  });

  activeCanvas.addEventListener('mousemove', (e) => {
    if (!isDrawing) return;

    const rect = activeCanvas.getBoundingClientRect();
    const currentX = e.clientX - rect.left;
    const currentY = e.clientY - rect.top;

    const width = currentX - startX;
    const height = currentY - startY;

    // Render preview on active layer
    const previewRect = {
      type: 'rect',
      x: startX,
      y: startY,
      width,
      height,
      stroke_color: store.get('strokeColor'),
      stroke_width: store.get('strokeWidth'),
    };

    engine.renderActive(previewRect);
  });

  activeCanvas.addEventListener('mouseup', (e) => {
    if (!isDrawing) return;
    isDrawing = false;

    const rect = activeCanvas.getBoundingClientRect();
    const currentX = e.clientX - rect.left;
    const currentY = e.clientY - rect.top;

    const width = currentX - startX;
    const height = currentY - startY;

    // Only create annotation if rectangle has size
    if (Math.abs(width) < 2 || Math.abs(height) < 2) {
      engine.renderActive(null); // Clear preview
      return;
    }

    // Create annotation object with snake_case to match Rust struct
    const annotation = {
      id: crypto.randomUUID(),
      type: 'rect',
      x: startX,
      y: startY,
      width,
      height,
      stroke_color: store.get('strokeColor'),
      stroke_width: store.get('strokeWidth'),
    };

    // Add to state
    const annotations = [...store.get('annotations'), annotation];
    store.set('annotations', annotations);

    // Render annotations and clear active layer
    engine.renderAnnotations(annotations);
    engine.renderActive(null);
  });

  // Re-render annotations when they change
  store.on('annotations', (annotations) => {
    engine.renderAnnotations(annotations, selectionManager.selected);
  });

  // Make engine available for rectangle tool
  window.canvasEngine = engine;

  console.log('Fotos initialized');
}

document.addEventListener('DOMContentLoaded', init);
