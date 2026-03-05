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

  // Highlight active tool and update aria-pressed
  store.on('activeTool', (tool) => {
    toolbar.querySelectorAll('[data-tool]').forEach(btn => {
      const active = btn.dataset.tool === tool;
      btn.classList.toggle('active', active);
      btn.setAttribute('aria-pressed', active ? 'true' : 'false');
    });
  });

  // Wire keyboard shortcuts for tools
  document.addEventListener('keydown', (e) => {
    if (e.target.tagName === 'TEXTAREA' || e.target.tagName === 'INPUT') return;
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const tool = TOOL_SHORTCUTS[e.key.toLowerCase()];
    if (tool) store.set('activeTool', tool);
  });
}
