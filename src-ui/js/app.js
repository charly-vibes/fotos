/// Fotos — main application entry point.
/// Initializes modules, wires event listeners, and manages global state.

import { store } from './state.js';
import { CanvasEngine } from './canvas/engine.js';
import { initToolbar } from './ui/toolbar.js';
import { ping, takeScreenshot } from './tauri-bridge.js';

const { listen } = window.__TAURI__.event;

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

  // Wire keyboard shortcuts
  document.addEventListener('keydown', async (e) => {
    // PrintScreen key for fullscreen capture
    if (e.key === 'PrintScreen') {
      e.preventDefault();
      try {
        const result = await takeScreenshot('fullscreen');
        console.log('Screenshot captured:', result.id);
      } catch (error) {
        console.error('Screenshot failed:', error);
        document.getElementById('status-message').textContent = `Capture failed: ${error}`;
      }
    }
  });

  // Listen for screenshot-ready events
  await listen('screenshot-ready', (event) => {
    console.log('Screenshot ready:', event.payload);
    const { id, width, height } = event.payload;
    document.getElementById('status-message').textContent = `Screenshot captured: ${width}x${height}`;
    // TODO: Load image into canvas (fotos-jub will implement this)
  });

  // TODO: initialize settings from backend

  console.log('Fotos initialized');
}

document.addEventListener('DOMContentLoaded', init);
