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

        // Load image into canvas
        const { width, height } = await engine.loadImage(result.data_url);

        // Update state
        store.set('currentImageId', result.id);

        // Update status bar
        document.getElementById('status-dimensions').textContent = `${width}×${height}`;
        document.getElementById('status-message').textContent = 'Ready';
      } catch (error) {
        console.error('Screenshot failed:', error);
        document.getElementById('status-message').textContent = `Capture failed: ${error}`;
      }
    }
  });

  // Listen for screenshot-ready events (for future use)
  await listen('screenshot-ready', (event) => {
    console.log('Screenshot ready event:', event.payload);
  });

  // TODO: initialize settings from backend

  console.log('Fotos initialized');
}

document.addEventListener('DOMContentLoaded', init);
