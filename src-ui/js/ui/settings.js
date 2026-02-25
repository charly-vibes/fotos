/// Settings modal — API key management and user preferences.

import { getApiKey, setApiKey, deleteApiKey, testApiKey } from '../tauri-bridge.js';

const PROVIDERS = [
  { id: 'anthropic', name: 'Anthropic (Claude)', placeholder: 'sk-ant-...' },
  { id: 'openai',    name: 'OpenAI (GPT-4o)',    placeholder: 'sk-...' },
  { id: 'gemini',    name: 'Google (Gemini)',     placeholder: 'AIza...' },
];

function getModal() {
  return document.getElementById('settings-modal');
}

export function showSettingsModal() {
  getModal().classList.remove('hidden');
  refreshKeyStatuses();
}

function hideSettingsModal() {
  getModal().classList.add('hidden');
}

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

export function initSettings() {
  const modal = getModal();

  modal.querySelector('.modal-backdrop').addEventListener('click', hideSettingsModal);
  modal.querySelectorAll('.modal-close, .btn-close-settings').forEach(btn => {
    btn.addEventListener('click', hideSettingsModal);
  });
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape' && !modal.classList.contains('hidden')) hideSettingsModal();
  });

  for (const row of modal.querySelectorAll('.api-key-row')) {
    const provider = row.dataset.provider;
    const input = row.querySelector('.api-key-input');

    // Show/hide toggle
    row.querySelector('.btn-show-hide').addEventListener('click', function () {
      const isPassword = input.type === 'password';
      input.type = isPassword ? 'text' : 'password';
      this.textContent = isPassword ? 'Hide' : 'Show';
    });

    // Save
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

    // Test
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

    // Delete
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
