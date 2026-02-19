/// AI panel — displays OCR results and LLM responses.

export function initAiPanel(store) {
  const panel = document.getElementById('ai-panel');
  const toggleBtn = panel.querySelector('[data-action="toggle-ai-panel"]');

  toggleBtn?.addEventListener('click', () => {
    panel.classList.toggle('collapsed');
  });

  // TODO: render OCR results when available
  // TODO: render LLM responses when available
}
