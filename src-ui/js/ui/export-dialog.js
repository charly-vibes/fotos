/// Export dialog — save, copy, and export-annotations options.

import { saveImage, showSaveDialog, exportAnnotations, compositeImage } from '../tauri-bridge.js';

function getModal() {
  return document.getElementById('export-modal');
}

export function showExportDialog({ getImageId, getAnnotations, onToast, onStatus }) {
  const modal = getModal();

  const formatSelect = modal.querySelector('#export-format');
  const qualityRow = modal.querySelector('#export-quality-row');
  const qualityInput = modal.querySelector('#export-quality');
  const qualityValue = modal.querySelector('#export-quality-value');

  // Show/hide quality slider based on format.
  function updateQualityVisibility() {
    qualityRow.classList.toggle('hidden', formatSelect.value !== 'jpeg');
  }
  formatSelect.addEventListener('change', updateQualityVisibility);
  updateQualityVisibility();

  // Live quality label.
  qualityInput.addEventListener('input', () => {
    qualityValue.textContent = qualityInput.value;
  });

  modal.classList.remove('hidden');

  function hide() {
    modal.classList.add('hidden');
    formatSelect.removeEventListener('change', updateQualityVisibility);
  }

  modal.querySelector('.modal-close').onclick = hide;
  modal.querySelector('.modal-backdrop').onclick = hide;

  function onKey(e) {
    if (e.key === 'Escape') { hide(); document.removeEventListener('keydown', onKey); }
  }
  document.addEventListener('keydown', onKey);

  modal.querySelector('#export-btn-save').onclick = async () => {
    const imageId = getImageId();
    if (!imageId) { onToast('No image loaded', 'error'); return; }
    const format = formatSelect.value;
    hide();

    const now = new Date();
    const ts = now.toISOString().replace(/T/, '-').replace(/:/g, '').slice(0, 15);
    const ext = format === 'jpeg' ? 'jpg' : format;
    const path = await showSaveDialog({
      filters: [{ name: format.toUpperCase(), extensions: [ext] }],
      defaultPath: `fotos-${ts}.${ext}`,
    });
    if (!path) return;

    try {
      onStatus('Saving…', false);
      await saveImage(imageId, getAnnotations(), format, path);
      onStatus('');
      onToast(`Saved to ${path}`);
    } catch (err) {
      onStatus('');
      onToast(`Save failed: ${err}`, 'error');
    }
  };

  modal.querySelector('#export-btn-copy').onclick = async () => {
    const imageId = getImageId();
    if (!imageId) { onToast('No image loaded', 'error'); return; }
    hide();
    try {
      onStatus('Copying to clipboard…', false);
      const imagePromise = compositeImage(imageId, getAnnotations()).then(base64PngToBlob);
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': imagePromise })]);
      onStatus('');
      onToast('Copied to clipboard');
    } catch (err) {
      onStatus('');
      onToast(`Copy failed: ${err}`, 'error');
    }
  };

  modal.querySelector('#export-btn-annotations').onclick = async () => {
    const imageId = getImageId();
    if (!imageId) { onToast('No image loaded', 'error'); return; }
    hide();
    try {
      await exportAnnotations(imageId, getAnnotations());
      onToast('Annotations exported');
    } catch (err) {
      onToast(`Export failed: ${err}`, 'error');
    }
  };
}

function base64PngToBlob(base64) {
  const bin = atob(base64.replace(/^data:[^;]+;base64,/, ''));
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return new Blob([bytes], { type: 'image/png' });
}
