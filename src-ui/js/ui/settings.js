/// Settings modal — preferences and API key management.

import {
  getApiKey, setApiKey, deleteApiKey, testApiKey,
  getSettings, setSettings,
  tessdataAvailable, downloadTessdata,
} from '../tauri-bridge.js';

const SETTINGS_VERSION = 2;

// Static named providers (Claude and Gemini have unique wire formats).
const NAMED_PROVIDERS = [
  { id: 'anthropic', name: 'Anthropic (Claude)', placeholder: 'sk-ant-...' },
  { id: 'gemini',    name: 'Google (Gemini)',     placeholder: 'AIza...' },
];

const DEFAULT_ENDPOINTS = [
  { id: 'openai',       name: 'OpenAI',        baseUrl: 'https://api.openai.com/v1', model: 'gpt-4o' },
  { id: 'ollama-local', name: 'Ollama (local)', baseUrl: 'http://localhost:11434/v1', model: 'llava:7b' },
];

const DEFAULTS = {
  capture: {
    defaultMode: 'region',
    defaultFormat: 'png',
    jpegQuality: 90,
    saveDirectory: '~/Pictures/Fotos',
    copyToClipboardAfterCapture: true,
    includeMouseCursor: false,
    delayMs: 0,
  },
  annotation: {
    defaultStrokeColor: '#FF0000',
    defaultStrokeWidth: 2,
    defaultFontSize: 16,
    defaultFontFamily: 'sans-serif',
    stepNumberColor: '#FF0000',
    stepNumberSize: 24,
    blurRadius: 10,
  },
  ai: {
    ocrLanguage: 'eng',
    defaultLlmProvider: 'claude',
    endpoints: DEFAULT_ENDPOINTS,
    claudeModel: 'claude-sonnet-4-20250514',
    geminiModel: 'gemini-2.0-flash',
  },
  ui: {
    theme: 'system',
    showAiPanel: true,
    showStatusBar: true,
    smoothZoom: true,
  },
};

let saveTimer = null;
// In-memory endpoint list (source of truth while modal is open).
let _endpoints = [];

function getModal() {
  return document.getElementById('settings-modal');
}

async function updateTessdataUI(lang) {
  const tessdataRow = document.getElementById('tessdata-row');
  const tessdataStatus = document.getElementById('tessdata-status');
  const downloadBtn = document.getElementById('btn-download-tessdata');
  if (!tessdataRow) return;

  if (lang === 'eng') {
    tessdataRow.classList.add('hidden');
    return;
  }
  tessdataRow.classList.remove('hidden');
  tessdataStatus.textContent = 'Checking…';
  downloadBtn.classList.add('hidden');
  try {
    const available = await tessdataAvailable(lang);
    if (available) {
      tessdataStatus.textContent = 'Language data available';
    } else {
      tessdataStatus.textContent = 'Language data not downloaded';
      downloadBtn.classList.remove('hidden');
    }
  } catch {
    tessdataStatus.textContent = '';
    downloadBtn.classList.remove('hidden');
  }
}

export function showSettingsModal() {
  getModal().classList.remove('hidden');
  loadSettings().then(() => {
    const lang = document.getElementById('pref-ai-ocrLanguage')?.value ?? 'eng';
    updateTessdataUI(lang);
  });
  refreshKeyStatuses();
}

function hideSettingsModal() {
  getModal().classList.add('hidden');
}

// ─── preferences load / save ──────────────────────────────────────────────────

async function loadSettings() {
  try {
    const settings = await getSettings();
    applyToForm(settings);
  } catch (e) {
    console.error('Failed to load settings:', e);
  }
}

// Deep-merge settings with DEFAULTS so any missing keys are filled in.
function mergeWithDefaults(settings) {
  const ai = { ...DEFAULTS.ai, ...(settings.ai ?? {}) };
  // Ensure endpoints is always an array.
  if (!Array.isArray(ai.endpoints) || ai.endpoints.length === 0) {
    ai.endpoints = DEFAULT_ENDPOINTS;
  }
  return {
    capture: { ...DEFAULTS.capture, ...(settings.capture ?? {}) },
    annotation: { ...DEFAULTS.annotation, ...(settings.annotation ?? {}) },
    ai,
    ui: { ...DEFAULTS.ui, ...(settings.ui ?? {}) },
  };
}

function applyToForm(rawSettings) {
  const { capture, annotation, ai, ui } = mergeWithDefaults(rawSettings);

  // Capture
  setVal('pref-capture-defaultMode', capture.defaultMode);
  setVal('pref-capture-defaultFormat', capture.defaultFormat);
  setVal('pref-capture-jpegQuality', capture.jpegQuality);
  setVal('pref-capture-saveDirectory', capture.saveDirectory);
  setCheck('pref-capture-copyToClipboard', capture.copyToClipboardAfterCapture);
  setCheck('pref-capture-includeMouseCursor', capture.includeMouseCursor);
  setVal('pref-capture-delayMs', capture.delayMs);
  updateRangeDisplay('pref-capture-jpegQuality', capture.jpegQuality);

  // Annotation
  setVal('pref-annotation-strokeColor', annotation.defaultStrokeColor);
  setVal('pref-annotation-strokeWidth', annotation.defaultStrokeWidth);
  setVal('pref-annotation-fontSize', annotation.defaultFontSize);
  setVal('pref-annotation-fontFamily', annotation.defaultFontFamily);
  setVal('pref-annotation-stepColor', annotation.stepNumberColor);
  setVal('pref-annotation-stepSize', annotation.stepNumberSize);
  setVal('pref-annotation-blurRadius', annotation.blurRadius);

  // AI — static fields
  setVal('pref-ai-ocrLanguage', ai.ocrLanguage);
  setVal('pref-ai-claudeModel', ai.claudeModel);
  setVal('pref-ai-geminiModel', ai.geminiModel);

  // AI — dynamic endpoint list
  _endpoints = ai.endpoints.map(e => ({ ...e }));
  renderEndpointList();
  populateProviderSelector(ai.defaultLlmProvider);

  // UI
  setVal('pref-ui-theme', ui.theme);
  setCheck('pref-ui-showAiPanel', ui.showAiPanel);
  setCheck('pref-ui-showStatusBar', ui.showStatusBar);
  setCheck('pref-ui-smoothZoom', ui.smoothZoom ?? true);
  applyTheme(ui.theme ?? 'system');
}

function applyTheme(theme) {
  const root = document.documentElement;
  if (theme === 'system') {
    root.removeAttribute('data-theme');
  } else {
    root.setAttribute('data-theme', theme);
  }
}

function readFromForm() {
  return {
    capture: {
      defaultMode: getVal('pref-capture-defaultMode'),
      defaultFormat: getVal('pref-capture-defaultFormat'),
      jpegQuality: parseInt(getVal('pref-capture-jpegQuality'), 10),
      saveDirectory: getVal('pref-capture-saveDirectory'),
      copyToClipboardAfterCapture: getCheck('pref-capture-copyToClipboard'),
      includeMouseCursor: getCheck('pref-capture-includeMouseCursor'),
      delayMs: parseInt(getVal('pref-capture-delayMs'), 10),
    },
    annotation: {
      defaultStrokeColor: getVal('pref-annotation-strokeColor'),
      defaultStrokeWidth: parseFloat(getVal('pref-annotation-strokeWidth')),
      defaultFontSize: parseFloat(getVal('pref-annotation-fontSize')),
      defaultFontFamily: getVal('pref-annotation-fontFamily'),
      stepNumberColor: getVal('pref-annotation-stepColor'),
      stepNumberSize: parseFloat(getVal('pref-annotation-stepSize')),
      blurRadius: parseFloat(getVal('pref-annotation-blurRadius')),
    },
    ai: {
      ocrLanguage: getVal('pref-ai-ocrLanguage'),
      defaultLlmProvider: getVal('pref-ai-defaultProvider'),
      endpoints: _endpoints.map(e => ({ ...e })),
      claudeModel: getVal('pref-ai-claudeModel'),
      geminiModel: getVal('pref-ai-geminiModel'),
    },
    ui: {
      theme: getVal('pref-ui-theme'),
      showAiPanel: getCheck('pref-ui-showAiPanel'),
      showStatusBar: getCheck('pref-ui-showStatusBar'),
      smoothZoom: getCheck('pref-ui-smoothZoom'),
    },
  };
}

function scheduleSave() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    try {
      await setSettings(readFromForm());
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }, 400);
}

// ─── endpoint list ────────────────────────────────────────────────────────────

function generateId() {
  // 8-char hex ID (like a UUID v4 prefix).
  return Array.from(crypto.getRandomValues(new Uint8Array(4)))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

function renderEndpointList() {
  const list = document.getElementById('endpoint-list');
  if (!list) return;
  list.innerHTML = '';
  for (const endpoint of _endpoints) {
    list.appendChild(createEndpointRow(endpoint));
  }
}

function createEndpointRow(endpoint) {
  const row = document.createElement('div');
  row.className = 'endpoint-row';
  row.dataset.endpointId = endpoint.id;

  row.innerHTML = `
    <div class="endpoint-fields">
      <input type="text"  class="endpoint-name"  placeholder="Name"                        value="${esc(endpoint.name)}">
      <input type="url"   class="endpoint-url"   placeholder="https://api.openai.com/v1"   value="${esc(endpoint.baseUrl)}">
      <input type="text"  class="endpoint-model" placeholder="gpt-4o"                      value="${esc(endpoint.model)}">
    </div>
    <div class="endpoint-actions">
      <button class="btn-endpoint-key">Set Key</button>
      <button class="btn-endpoint-del-key">Delete Key</button>
      <button class="btn-endpoint-remove" title="Remove endpoint">×</button>
    </div>
    <div class="endpoint-key-area hidden">
      <input type="password" class="endpoint-key-input" placeholder="API key (leave empty for local servers)" autocomplete="off" spellcheck="false">
      <button class="btn-endpoint-key-save">Save</button>
      <button class="btn-endpoint-key-cancel">Cancel</button>
    </div>
  `;

  // Sync field changes back to _endpoints and schedule save.
  row.querySelector('.endpoint-name').addEventListener('input', e => {
    updateEndpoint(endpoint.id, { name: e.target.value });
    populateProviderSelector(getVal('pref-ai-defaultProvider'));
    scheduleSave();
  });
  row.querySelector('.endpoint-url').addEventListener('input', e => {
    updateEndpoint(endpoint.id, { baseUrl: e.target.value });
    scheduleSave();
  });
  row.querySelector('.endpoint-model').addEventListener('input', e => {
    updateEndpoint(endpoint.id, { model: e.target.value });
    scheduleSave();
  });

  const keyArea = row.querySelector('.endpoint-key-area');
  const keyInput = row.querySelector('.endpoint-key-input');

  row.querySelector('.btn-endpoint-key').addEventListener('click', () => {
    keyArea.classList.toggle('hidden');
    if (!keyArea.classList.contains('hidden')) keyInput.focus();
  });

  row.querySelector('.btn-endpoint-key-cancel').addEventListener('click', () => {
    keyArea.classList.add('hidden');
    keyInput.value = '';
  });

  row.querySelector('.btn-endpoint-key-save').addEventListener('click', async () => {
    const key = keyInput.value.trim();
    if (!key) return;
    try {
      await setApiKey(`endpoint:${endpoint.id}`, key);
      keyArea.classList.add('hidden');
      keyInput.value = '';
    } catch (e) {
      console.error('Failed to save key:', e);
    }
  });

  row.querySelector('.btn-endpoint-del-key').addEventListener('click', async () => {
    try {
      await deleteApiKey(`endpoint:${endpoint.id}`);
    } catch (e) {
      console.error('Failed to delete key:', e);
    }
  });

  row.querySelector('.btn-endpoint-remove').addEventListener('click', () => {
    _endpoints = _endpoints.filter(e => e.id !== endpoint.id);
    // If the removed endpoint was the selected provider, fall back to claude.
    const sel = document.getElementById('pref-ai-defaultProvider');
    if (sel && sel.value === `endpoint:${endpoint.id}`) {
      populateProviderSelector('claude');
    } else {
      populateProviderSelector(sel?.value ?? 'claude');
    }
    row.remove();
    scheduleSave();
  });

  return row;
}

function updateEndpoint(id, patch) {
  const ep = _endpoints.find(e => e.id === id);
  if (ep) Object.assign(ep, patch);
}

function esc(str) {
  return String(str ?? '').replace(/&/g, '&amp;').replace(/"/g, '&quot;');
}

// ─── provider selector ────────────────────────────────────────────────────────

function populateProviderSelector(selectedValue) {
  const sel = document.getElementById('pref-ai-defaultProvider');
  if (!sel) return;
  sel.innerHTML = '';

  const add = (value, label) => {
    const opt = document.createElement('option');
    opt.value = value;
    opt.textContent = label;
    if (value === selectedValue) opt.selected = true;
    sel.appendChild(opt);
  };

  add('claude', 'Claude (Anthropic)');
  add('gemini', 'Gemini (Google)');

  for (const ep of _endpoints) {
    add(`endpoint:${ep.id}`, ep.name || ep.id);
  }

  // If selectedValue wasn't matched (e.g. deleted endpoint), default to claude.
  if (!sel.value || sel.value !== selectedValue) {
    sel.value = 'claude';
  }
}

// ─── helpers ──────────────────────────────────────────────────────────────────

function setVal(id, value) {
  const el = document.getElementById(id);
  if (el) el.value = value ?? '';
}

function setCheck(id, value) {
  const el = document.getElementById(id);
  if (el) el.checked = Boolean(value);
}

function getVal(id) {
  const el = document.getElementById(id);
  return el ? el.value : '';
}

function getCheck(id) {
  const el = document.getElementById(id);
  return el ? el.checked : false;
}

function updateRangeDisplay(id, value) {
  const el = document.getElementById(`${id}-display`);
  if (el) el.textContent = value;
}

// ─── API key management (static providers) ────────────────────────────────────

async function refreshKeyStatuses() {
  for (const { id } of NAMED_PROVIDERS) {
    const row = getModal().querySelector(`[data-provider="${id}"]`);
    if (!row) continue;
    setRowStatus(row, 'loading', 'Checking...');
    try {
      const masked = await getApiKey(id);
      if (masked) {
        setRowStatus(row, 'set', masked);
      } else {
        setRowStatus(row, 'missing', 'No key set');
      }
    } catch {
      setRowStatus(row, 'missing', 'No key set');
    }
  }
}

function setRowStatus(row, state, text) {
  const el = row.querySelector('.key-status');
  el.textContent = text;
  el.className = `key-status key-status--${state}`;
}

// ─── about ────────────────────────────────────────────────────────────────────

async function loadAboutInfo() {
  const el = document.getElementById('about-version');
  if (!el || el.textContent !== '—') return;
  try {
    const version = await window.__TAURI__.app.getVersion();
    el.textContent = version;
  } catch {
    el.textContent = 'unknown';
  }
}

// ─── init ─────────────────────────────────────────────────────────────────────

export async function applyThemeFromSettings() {
  try {
    const settings = await getSettings();
    applyTheme(settings.ui?.theme ?? 'system');
  } catch { /* ignore, defaults to system theme */ }
}

export function initSettings() {
  const modal = getModal();

  // Close handlers
  modal.querySelector('.modal-backdrop').addEventListener('click', hideSettingsModal);
  modal.querySelectorAll('.modal-close, .btn-close-settings').forEach(btn => {
    btn.addEventListener('click', hideSettingsModal);
  });
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape' && !modal.classList.contains('hidden')) hideSettingsModal();
  });

  // Tab switching
  modal.querySelectorAll('.settings-tab-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      modal.querySelectorAll('.settings-tab-btn').forEach(b => b.classList.remove('active'));
      modal.querySelectorAll('.settings-tab-panel').forEach(p => p.classList.remove('active'));
      btn.classList.add('active');
      modal.querySelector(`#tab-${btn.dataset.tab}`).classList.add('active');

      // Load API key statuses lazily when that tab is shown
      if (btn.dataset.tab === 'keys') refreshKeyStatuses();
      if (btn.dataset.tab === 'about') loadAboutInfo();
    });
  });

  // Range display update + auto-save
  const jpegRange = modal.querySelector('#pref-capture-jpegQuality');
  jpegRange.addEventListener('input', () => {
    updateRangeDisplay('pref-capture-jpegQuality', jpegRange.value);
    scheduleSave();
  });

  // Apply theme immediately on change (before debounced save)
  modal.querySelector('#pref-ui-theme').addEventListener('change', (e) => {
    applyTheme(e.target.value);
  });

  // Auto-save on any static preference change
  modal.querySelectorAll(
    '.settings-tab-panel select, ' +
    '.settings-tab-panel input[type="text"], ' +
    '.settings-tab-panel input[type="number"], ' +
    '.settings-tab-panel input[type="url"], ' +
    '.settings-tab-panel input[type="color"], ' +
    '.settings-tab-panel input[type="checkbox"]'
  ).forEach(el => {
    el.addEventListener('change', scheduleSave);
  });

  // Add endpoint button
  document.getElementById('btn-add-endpoint')?.addEventListener('click', () => {
    const endpoint = {
      id: generateId(),
      name: '',
      baseUrl: '',
      model: '',
    };
    _endpoints.push(endpoint);
    const list = document.getElementById('endpoint-list');
    list?.appendChild(createEndpointRow(endpoint));
    populateProviderSelector(getVal('pref-ai-defaultProvider'));
    scheduleSave();
  });

  // Reset to Defaults
  modal.querySelector('.btn-reset-defaults').addEventListener('click', async () => {
    if (!confirm('Reset all preferences to their default values? API keys will not be affected.')) return;
    _endpoints = DEFAULT_ENDPOINTS.map(e => ({ ...e }));
    applyToForm(DEFAULTS);
    try {
      await setSettings(DEFAULTS);
    } catch (e) {
      console.error('Failed to reset settings:', e);
    }
  });

  // OCR tessdata download
  const langSelect = modal.querySelector('#pref-ai-ocrLanguage');
  const downloadBtn = document.getElementById('btn-download-tessdata');
  const tessdataStatus = document.getElementById('tessdata-status');

  langSelect?.addEventListener('change', (e) => {
    updateTessdataUI(e.target.value);
  });

  downloadBtn?.addEventListener('click', async () => {
    const lang = langSelect.value;
    downloadBtn.disabled = true;
    tessdataStatus.textContent = 'Downloading…';
    try {
      await downloadTessdata(lang);
      tessdataStatus.textContent = 'Language data available';
      downloadBtn.classList.add('hidden');
    } catch (e) {
      tessdataStatus.textContent = `Download failed: ${e}`;
      downloadBtn.disabled = false;
    }
  });

  // Listen for background progress events (in case download is triggered elsewhere).
  if (window.__TAURI__?.event) {
    window.__TAURI__.event.listen('tessdata:progress', ({ payload }) => {
      if (payload.lang === langSelect?.value && payload.downloaded === payload.total && payload.total > 0) {
        tessdataStatus.textContent = 'Language data available';
        downloadBtn.classList.add('hidden');
        downloadBtn.disabled = false;
      }
    });
  }

  // API key rows (static named providers: Anthropic, Gemini)
  for (const row of modal.querySelectorAll('.api-key-row')) {
    const provider = row.dataset.provider;
    const input = row.querySelector('.api-key-input');

    row.querySelector('.btn-show-hide').addEventListener('click', function () {
      const isPassword = input.type === 'password';
      input.type = isPassword ? 'text' : 'password';
      this.textContent = isPassword ? 'Hide' : 'Show';
    });

    row.querySelector('.btn-save-key').addEventListener('click', async () => {
      const key = input.value.trim();
      if (!key) return;
      setRowStatus(row, 'loading', 'Saving...');
      try {
        await setApiKey(provider, key);
        input.value = '';
        input.type = 'password';
        row.querySelector('.btn-show-hide').textContent = 'Show';
        const masked = await getApiKey(provider);
        setRowStatus(row, 'set', masked || 'Saved');
      } catch (e) {
        setRowStatus(row, 'error', `Save failed: ${e}`);
      }
    });

    row.querySelector('.btn-test-key').addEventListener('click', async function () {
      this.disabled = true;
      setRowStatus(row, 'loading', 'Testing...');
      try {
        await testApiKey(provider);
        setRowStatus(row, 'ok', 'Connected');
      } catch (e) {
        setRowStatus(row, 'error', String(e));
      } finally {
        this.disabled = false;
      }
    });

    row.querySelector('.btn-delete-key').addEventListener('click', async () => {
      try {
        await deleteApiKey(provider);
        input.value = '';
        input.type = 'password';
        row.querySelector('.btn-show-hide').textContent = 'Show';
        setRowStatus(row, 'missing', 'No key set');
      } catch (e) {
        setRowStatus(row, 'error', `Delete failed: ${e}`);
      }
    });
  }
}
