/// Fotos — main application entry point.
/// Initializes modules, wires event listeners, and manages global state.

import { store } from './state.js';
import { CanvasEngine } from './canvas/engine.js';
import { initToolbar } from './ui/toolbar.js';

async function init() {
  const baseCanvas = document.getElementById('canvas-base');
  const annoCanvas = document.getElementById('canvas-annotations');
  const activeCanvas = document.getElementById('canvas-active');

  const engine = new CanvasEngine(baseCanvas, annoCanvas, activeCanvas);

  initToolbar(store);

  // TODO: wire keyboard shortcuts
  // TODO: wire Tauri event listeners (screenshot-ready, etc.)
  // TODO: initialize settings from backend

  console.log('Fotos initialized');
}

document.addEventListener('DOMContentLoaded', init);
