/// Color picker — color and opacity selection for annotation tools.

const RECENT_STROKE_KEY = 'fotos-recent-stroke';
const RECENT_FILL_KEY = 'fotos-recent-fill';
const MAX_RECENT = 8;

function loadRecent(key) {
  try {
    const raw = localStorage.getItem(key);
    return raw ? JSON.parse(raw) : [];
  } catch { return []; }
}

function saveRecent(key, colors) {
  localStorage.setItem(key, JSON.stringify(colors));
}

function addToRecent(color, current) {
  if (!color || color === 'transparent') return current;
  const deduped = [color, ...current.filter(c => c !== color)];
  return deduped.slice(0, MAX_RECENT);
}

// Exported so app.js can call when an annotation is committed.
let _onAnnotationCommit = null;
export function notifyColorApplied(strokeColor, fillColor) {
  if (_onAnnotationCommit) _onAnnotationCommit(strokeColor, fillColor);
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
    <div class="cp-section" id="cp-recent-stroke-section">
      <div class="cp-label">Recent stroke</div>
      <div id="cp-recent-stroke-colors"></div>
    </div>
    <div class="cp-section" id="cp-recent-fill-section">
      <div class="cp-label">Recent fill</div>
      <div id="cp-recent-fill-colors"></div>
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
  const recentStrokeRow = popup.querySelector('#cp-recent-stroke-colors');
  const recentStrokeSection = popup.querySelector('#cp-recent-stroke-section');
  const recentFillRow = popup.querySelector('#cp-recent-fill-colors');
  const recentFillSection = popup.querySelector('#cp-recent-fill-section');

  let recentStroke = loadRecent(RECENT_STROKE_KEY);
  let recentFill = loadRecent(RECENT_FILL_KEY);

  function renderRecentRow(colors, rowEl, sectionEl, onClick) {
    rowEl.innerHTML = '';
    sectionEl.style.display = colors.length ? '' : 'none';
    colors.forEach(color => {
      const swatch = document.createElement('button');
      swatch.className = 'cp-recent-swatch';
      swatch.style.background = color;
      swatch.title = color;
      swatch.addEventListener('click', () => onClick(color));
      rowEl.appendChild(swatch);
    });
  }

  function renderRecent() {
    renderRecentRow(recentStroke, recentStrokeRow, recentStrokeSection, (color) => {
      store.set('strokeColor', color);
    });
    renderRecentRow(recentFill, recentFillRow, recentFillSection, (color) => {
      store.set('fillColor', color);
    });
  }

  // Called by app.js when an annotation is actually committed.
  _onAnnotationCommit = (stroke, fill) => {
    if (stroke && stroke !== 'transparent') {
      recentStroke = addToRecent(stroke, recentStroke);
      saveRecent(RECENT_STROKE_KEY, recentStroke);
    }
    if (fill && fill !== 'transparent') {
      recentFill = addToRecent(fill, recentFill);
      saveRecent(RECENT_FILL_KEY, recentFill);
    }
    renderRecent();
  };

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

  // Fill color changes
  fillInput.addEventListener('input', () => {
    store.set('fillColor', fillInput.value);
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
