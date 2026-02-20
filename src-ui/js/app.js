/// Fotos — main application entry point.
/// Initializes modules, wires event listeners, and manages global state.

import { store } from './state.js';
import { CanvasEngine } from './canvas/engine.js';
import { initToolbar } from './ui/toolbar.js';
import { ping } from './tauri-bridge.js';

async function init() {
  // Verify Tauri IPC connection
  try {
    const response = await ping();
    console.log('Backend ping:', response);
    document.getElementById('status-message').textContent = 'Backend connected';
  } catch (error) {
    console.error('Backend ping failed:', error);
    document.getElementById('status-message').textContent = 'Backend connection failed';
  }

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
