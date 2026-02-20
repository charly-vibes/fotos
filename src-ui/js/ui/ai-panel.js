/// AI panel — displays OCR results and LLM responses.

export function initAiPanel(store) {
  const panel = document.getElementById('ai-panel');
  const toggleBtn = panel.querySelector('[data-action="toggle-ai-panel"]');
  const ocrResults = document.getElementById('ocr-results');
  const llmResults = document.getElementById('llm-results');

  toggleBtn?.addEventListener('click', () => {
    panel.classList.toggle('collapsed');
  });

  // Render OCR results when available
  store.on('ocrResults', (result) => {
    if (result) {
      ocrResults.textContent = result.text;
      ocrResults.classList.remove('hidden');
      // Expand panel when results arrive
      panel.classList.remove('collapsed');
    } else {
      ocrResults.classList.add('hidden');
    }
  });

  // Render LLM responses when available
  store.on('llmResults', (result) => {
    if (result) {
      llmResults.textContent = result.response;
      llmResults.classList.remove('hidden');
      panel.classList.remove('collapsed');
    } else {
      llmResults.classList.add('hidden');
    }
  });
}
