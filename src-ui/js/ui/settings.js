/// Settings modal — preferences and API key management.

import {
  getApiKey, setApiKey, deleteApiKey, testApiKey,
  getSettings, setSettings,
} from '../tauri-bridge.js';

const PROVIDERS = [
  { id: 'anthropic', name: 'Anthropic (Claude)', placeholder: 'sk-ant-...' },
  { id: 'openai',    name: 'OpenAI (GPT-4o)',    placeholder: 'sk-...' },
  { id: 'gemini',    name: 'Google (Gemini)',     placeholder: 'AIza...' },
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
    ollamaUrl: 'http://localhost:11434',
    ollamaModel: 'llava:7b',
    claudeModel: 'claude-sonnet-4-20250514',
    openaiModel: 'gpt-4o',
    geminiModel: 'gemini-2.0-flash',
  },
  ui: {
    theme: 'system',
    showAiPanel: true,
    showStatusBar: true,
  },
};

let saveTimer = null;

function getModal() {
  return document.getElementById('settings-modal');
}

export function showSettingsModal() {
  getModal().classList.remove('hidden');
  loadSettings();
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

function applyToForm({ capture, annotation, ai, ui }) {
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

  // AI
  setVal('pref-ai-ocrLanguage', ai.ocrLanguage);
  setVal('pref-ai-defaultProvider', ai.defaultLlmProvider);
  setVal('pref-ai-claudeModel', ai.claudeModel);
  setVal('pref-ai-openaiModel', ai.openaiModel);
  setVal('pref-ai-geminiModel', ai.geminiModel);
  setVal('pref-ai-ollamaUrl', ai.ollamaUrl);
  setVal('pref-ai-ollamaModel', ai.ollamaModel);

  // UI
  setVal('pref-ui-theme', ui.theme);
  setCheck('pref-ui-showAiPanel', ui.showAiPanel);
  setCheck('pref-ui-showStatusBar', ui.showStatusBar);
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
      claudeModel: getVal('pref-ai-claudeModel'),
      openaiModel: getVal('pref-ai-openaiModel'),
      geminiModel: getVal('pref-ai-geminiModel'),
      ollamaUrl: getVal('pref-ai-ollamaUrl'),
      ollamaModel: getVal('pref-ai-ollamaModel'),
    },
    ui: {
      theme: getVal('pref-ui-theme'),
      showAiPanel: getCheck('pref-ui-showAiPanel'),
      showStatusBar: getCheck('pref-ui-showStatusBar'),
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

// ─── API key management ───────────────────────────────────────────────────────

async function refreshKeyStatuses() {
  for (const { id } of PROVIDERS) {
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

// ─── init ─────────────────────────────────────────────────────────────────────

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
    });
  });

  // Range display update + auto-save
  const jpegRange = modal.querySelector('#pref-capture-jpegQuality');
  jpegRange.addEventListener('input', () => {
    updateRangeDisplay('pref-capture-jpegQuality', jpegRange.value);
    scheduleSave();
  });

  // Auto-save on any preference change
  modal.querySelectorAll(
    '.settings-tab-panel select, ' +
    '.settings-tab-panel input[type="text"], ' +
    '.settings-tab-panel input[type="number"], ' +
    '.settings-tab-panel input[type="url"], ' +
    '.settings-tab-panel input[type="color"], ' +
    '.settings-tab-panel input[type="checkbox"]'
  ).forEach(el => {
    const event = el.type === 'checkbox' || el.tagName === 'SELECT' ? 'change' : 'change';
    el.addEventListener(event, scheduleSave);
  });

  // Reset to Defaults
  modal.querySelector('.btn-reset-defaults').addEventListener('click', async () => {
    if (!confirm('Reset all preferences to their default values? API keys will not be affected.')) return;
    applyToForm(DEFAULTS);
    try {
      await setSettings(DEFAULTS);
    } catch (e) {
      console.error('Failed to reset settings:', e);
    }
  });

  // API key rows
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
