/// Color picker — color and opacity selection for annotation tools.

const RECENT_KEY = 'fotos-recent-colors';
const MAX_RECENT = 8;

function loadRecent() {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch { return []; }
}

function saveRecent(colors) {
  localStorage.setItem(RECENT_KEY, JSON.stringify(colors));
}

function addToRecent(color, current) {
  if (!color || color === 'transparent') return current;
  const deduped = [color, ...current.filter(c => c !== color)];
  return deduped.slice(0, MAX_RECENT);
}

export function initColorPicker(store) {
  const trigger = document.getElementById('color-picker-trigger');

  // Build trigger swatches (Photoshop-style stacked stroke/fill indicator)
  const swatchStroke = document.createElement('div');
  swatchStroke.className = 'cp-swatch-stroke';
  const swatchFill = document.createElement('div');
  swatchFill.className = 'cp-swatch-fill';
  trigger.append(swatchStroke, swatchFill);

  // Build popup
  const popup = document.createElement('div');
  popup.id = 'color-picker-popup';
  popup.className = 'hidden';
  popup.innerHTML = `
    <div class="cp-section">
      <div class="cp-label">Stroke</div>
      <div class="cp-color-row">
        <input type="color" id="cp-stroke-input">
        <code class="cp-hex" id="cp-stroke-hex"></code>
      </div>
    </div>
    <div class="cp-section">
      <div class="cp-label">Fill</div>
      <div class="cp-color-row">
        <input type="color" id="cp-fill-input">
        <code class="cp-hex" id="cp-fill-hex"></code>
        <button id="cp-fill-transparent" title="Transparent fill">⊘</button>
      </div>
    </div>
    <div class="cp-section">
      <div class="cp-label">Opacity <span id="cp-opacity-val">100%</span></div>
      <input type="range" id="cp-opacity" min="0" max="100" value="100" step="1">
    </div>
    <div class="cp-section" id="cp-recent-section">
      <div class="cp-label">Recent</div>
      <div id="cp-recent-colors"></div>
    </div>
  `;
  document.body.appendChild(popup);

  const strokeInput = popup.querySelector('#cp-stroke-input');
  const strokeHex = popup.querySelector('#cp-stroke-hex');
  const fillInput = popup.querySelector('#cp-fill-input');
  const fillHex = popup.querySelector('#cp-fill-hex');
  const fillTransparentBtn = popup.querySelector('#cp-fill-transparent');
  const opacitySlider = popup.querySelector('#cp-opacity');
  const opacityVal = popup.querySelector('#cp-opacity-val');
  const recentRow = popup.querySelector('#cp-recent-colors');
  const recentSection = popup.querySelector('#cp-recent-section');

  let recentColors = loadRecent();

  function renderRecent() {
    recentRow.innerHTML = '';
    recentSection.style.display = recentColors.length ? '' : 'none';
    recentColors.forEach(color => {
      const swatch = document.createElement('button');
      swatch.className = 'cp-recent-swatch';
      swatch.style.background = color;
      swatch.title = color;
      swatch.addEventListener('click', () => {
        store.set('strokeColor', color);
        recentColors = addToRecent(color, recentColors);
        saveRecent(recentColors);
        renderRecent();
      });
      recentRow.appendChild(swatch);
    });
  }

  function updateFillSwatch(fillColor) {
    const isTransparent = fillColor === 'transparent';
    swatchFill.classList.toggle('cp-transparent', isTransparent);
    if (!isTransparent) swatchFill.style.background = fillColor;
    else swatchFill.style.background = '';
  }

  function updateFillUI(fillColor) {
    const isTransparent = fillColor === 'transparent';
    fillInput.disabled = isTransparent;
    if (!isTransparent) fillInput.value = fillColor;
    fillHex.textContent = isTransparent ? 'transparent' : fillColor;
    fillTransparentBtn.classList.toggle('active', isTransparent);
    updateFillSwatch(fillColor);
  }

  function syncFromStore() {
    const stroke = store.get('strokeColor') || '#FF0000';
    const fill = store.get('fillColor') || 'transparent';
    const opacity = store.get('opacity') ?? 1.0;

    strokeInput.value = stroke;
    strokeHex.textContent = stroke;
    swatchStroke.style.background = stroke;

    updateFillUI(fill);

    const pct = Math.round(opacity * 100);
    opacitySlider.value = pct;
    opacityVal.textContent = `${pct}%`;
  }

  store.on('strokeColor', syncFromStore);
  store.on('fillColor', syncFromStore);
  store.on('opacity', syncFromStore);

  // Stroke color changes
  strokeInput.addEventListener('input', () => {
    store.set('strokeColor', strokeInput.value);
  });
  strokeInput.addEventListener('change', () => {
    recentColors = addToRecent(strokeInput.value, recentColors);
    saveRecent(recentColors);
    renderRecent();
  });

  // Fill color changes
  fillInput.addEventListener('input', () => {
    store.set('fillColor', fillInput.value);
  });
  fillInput.addEventListener('change', () => {
    recentColors = addToRecent(fillInput.value, recentColors);
    saveRecent(recentColors);
    renderRecent();
  });

  // Transparent fill toggle
  fillTransparentBtn.addEventListener('click', () => {
    store.set('fillColor', 'transparent');
  });

  // Opacity slider
  opacitySlider.addEventListener('input', () => {
    const val = parseInt(opacitySlider.value, 10) / 100;
    store.set('opacity', val);
    opacityVal.textContent = `${opacitySlider.value}%`;
  });

  // Popup open/close
  function openPopup() {
    syncFromStore();
    renderRecent();
    const rect = trigger.getBoundingClientRect();
    popup.style.top = `${rect.bottom + 4}px`;
    popup.style.left = `${rect.left}px`;
    popup.classList.remove('hidden');
  }

  function closePopup() {
    popup.classList.add('hidden');
  }

  trigger.addEventListener('click', (e) => {
    e.stopPropagation();
    popup.classList.contains('hidden') ? openPopup() : closePopup();
  });

  document.addEventListener('click', (e) => {
    if (!popup.contains(e.target) && !trigger.contains(e.target)) {
      closePopup();
    }
  });

  // Initial render
  syncFromStore();
}
