/// Toolbar — tool button management and event wiring.

import { TOOL_SHORTCUTS } from '../canvas/tools.js';

export function initToolbar(store) {
  const toolbar = document.getElementById('toolbar');

  // Wire annotation tool buttons
  toolbar.querySelectorAll('[data-tool]').forEach(btn => {
    btn.addEventListener('click', () => {
      store.set('activeTool', btn.dataset.tool);
    });
  });

  // Highlight active tool
  store.on('activeTool', (tool) => {
    toolbar.querySelectorAll('[data-tool]').forEach(btn => {
      btn.classList.toggle('active', btn.dataset.tool === tool);
    });
  });

  // Wire keyboard shortcuts for tools
  document.addEventListener('keydown', (e) => {
    if (e.target.tagName === 'TEXTAREA' || e.target.tagName === 'INPUT') return;
    const tool = TOOL_SHORTCUTS[e.key.toLowerCase()];
    if (tool) store.set('activeTool', tool);
  });
}
