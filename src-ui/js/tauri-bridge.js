/// Typed wrappers around Tauri invoke() calls.
/// Provides a clean JS API over the Rust backend commands.

const { invoke } = window.__TAURI__.core;

export async function ping() {
  return invoke('ping');
}

export async function takeScreenshot(mode, monitor = null) {
  return invoke('take_screenshot', { mode, monitor });
}

export async function listMonitors() {
  return invoke('list_monitors');
}

export async function listWindows() {
  return invoke('list_windows');
}

export async function runOcr(imageId, lang = null) {
  return invoke('run_ocr', { imageId, lang });
}

export async function autoBlurPii(imageId) {
  return invoke('auto_blur_pii', { imageId });
}

export async function analyzeLlm(imageId, prompt = null, provider = 'claude') {
  return invoke('analyze_llm', { imageId, prompt, provider });
}

export async function saveImage(imageId, annotations, format, path) {
  return invoke('save_image', { imageId, annotations, format, path });
}

export async function copyToClipboard(imageId, annotations) {
  return invoke('copy_to_clipboard', { imageId, annotations });
}

export async function exportAnnotations(imageId, annotations) {
  return invoke('export_annotations', { imageId, annotations });
}

export async function getSettings() {
  return invoke('get_settings');
}

export async function setSettings(settings) {
  return invoke('set_settings', { settings });
}

export async function setApiKey(provider, key) {
  return invoke('set_api_key', { provider, key });
}
