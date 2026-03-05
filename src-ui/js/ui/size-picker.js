/// Size picker — stroke width and font size controls.

const STROKE_PRESETS = [1, 2, 4, 8, 16];
const FONT_PRESETS = [8, 12, 16, 24, 36, 48];

export function initSizePicker(store) {
  const container = document.getElementById('size-picker');

  // Build trigger: a small canvas showing the current stroke line sample
  const trigger = document.createElement('div');
  trigger.id = 'size-picker-trigger';
  trigger.title = 'Size picker';
  trigger.setAttribute('aria-label', 'Size picker');
  trigger.setAttribute('role', 'button');
  trigger.setAttribute('tabindex', '0');
  trigger.setAttribute('aria-haspopup', 'true');

  const sampleCanvas = document.createElement('canvas');
  sampleCanvas.width = 24;
  sampleCanvas.height = 24;
  trigger.appendChild(sampleCanvas);
  container.appendChild(trigger);

  function drawSample(width) {
    const ctx = sampleCanvas.getContext('2d');
    ctx.clearRect(0, 0, 24, 24);
    ctx.beginPath();
    ctx.moveTo(3, 12);
    ctx.lineTo(21, 12);
    ctx.strokeStyle = getComputedStyle(document.documentElement).getPropertyValue('--text-primary').trim() || '#333';
    ctx.lineWidth = Math.min(width, 12);
    ctx.lineCap = 'round';
    ctx.stroke();
  }

  // Build popup
  const popup = document.createElement('div');
  popup.id = 'size-picker-popup';
  popup.className = 'hidden';
  popup.innerHTML = `
    <div class="sp-section">
      <div class="sp-label">Stroke width</div>
      <div class="sp-line-sample" id="sp-line-sample"></div>
      <div class="sp-slider-row">
        <input type="range" id="sp-stroke-slider" min="1" max="20" step="1">
        <span class="sp-value" id="sp-stroke-val">2px</span>
      </div>
      <div class="sp-presets" id="sp-stroke-presets"></div>
    </div>
    <div class="sp-section">
      <div class="sp-label">Font size</div>
      <div class="sp-presets" id="sp-font-presets"></div>
    </div>
  `;
  document.body.appendChild(popup);

  const strokeSlider = popup.querySelector('#sp-stroke-slider');
  const strokeVal = popup.querySelector('#sp-stroke-val');
  const lineSample = popup.querySelector('#sp-line-sample');
  const strokePresetsEl = popup.querySelector('#sp-stroke-presets');
  const fontPresetsEl = popup.querySelector('#sp-font-presets');

  // Draw stroke width line sample in the popup
  function drawLineSample(width) {
    lineSample.innerHTML = '';
    const c = document.createElement('canvas');
    c.width = 176;
    c.height = 16;
    c.style.display = 'block';
    const ctx = c.getContext('2d');
    ctx.beginPath();
    ctx.moveTo(8, 8);
    ctx.lineTo(168, 8);
    ctx.strokeStyle = getComputedStyle(document.documentElement).getPropertyValue('--text-primary').trim() || '#333';
    ctx.lineWidth = Math.min(width, 12);
    ctx.lineCap = 'round';
    ctx.stroke();
    lineSample.appendChild(c);
  }

  // Populate stroke width presets
  STROKE_PRESETS.forEach(w => {
    const btn = document.createElement('button');
    btn.className = 'sp-preset';
    btn.textContent = `${w}`;
    btn.dataset.strokePreset = w;
    btn.addEventListener('click', () => {
      store.set('strokeWidth', w);
      syncStroke(w);
    });
    strokePresetsEl.appendChild(btn);
  });

  // Populate font size presets
  FONT_PRESETS.forEach(s => {
    const btn = document.createElement('button');
    btn.className = 'sp-preset';
    btn.textContent = `${s}`;
    btn.dataset.fontPreset = s;
    btn.addEventListener('click', () => {
      store.set('fontSize', s);
      syncFont(s);
    });
    fontPresetsEl.appendChild(btn);
  });

  function syncStroke(width) {
    strokeSlider.value = width;
    strokeVal.textContent = `${width}px`;
    drawLineSample(width);
    drawSample(width);
    strokePresetsEl.querySelectorAll('.sp-preset').forEach(btn => {
      btn.classList.toggle('active', Number(btn.dataset.strokePreset) === width);
    });
  }

  function syncFont(size) {
    fontPresetsEl.querySelectorAll('.sp-preset').forEach(btn => {
      btn.classList.toggle('active', Number(btn.dataset.fontPreset) === size);
    });
  }

  strokeSlider.addEventListener('input', () => {
    const w = parseInt(strokeSlider.value, 10);
    store.set('strokeWidth', w);
    syncStroke(w);
  });

  store.on('strokeWidth', (w) => syncStroke(w));
  store.on('fontSize', (s) => syncFont(s));

  function openPopup() {
    syncStroke(store.get('strokeWidth') ?? 2);
    syncFont(store.get('fontSize') ?? 16);
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

  // Initial trigger sample
  syncStroke(store.get('strokeWidth') ?? 2);
}
