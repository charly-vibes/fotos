/// RegionPicker — fullscreen overlay for in-app region selection.
/// Shows a captured screenshot dimmed behind a crosshair drag-to-select UI.
/// Calls onSelect(imgX, imgY, imgW, imgH) with image-space coordinates on confirm,
/// or onCancel() if the user presses Escape.

export class RegionPicker {
  #overlay; #canvas; #ctx;
  #image = null;         // ImageBitmap of the full screenshot
  #imgW = 0; #imgH = 0;  // source pixel dimensions
  #scale = 1; #offX = 0; #offY = 0;  // display transform (contain scaling)

  #dragging = false;
  #startX = 0; #startY = 0;  // canvas CSS pixels
  #sel = null;               // { x, y, w, h } in canvas CSS pixels

  #onSelect = null;
  #onCancel = null;

  constructor() {
    this.#overlay = document.getElementById('region-picker-overlay');
    this.#canvas  = document.getElementById('region-picker-canvas');
    this.#ctx     = this.#canvas.getContext('2d');
    this._bindEvents();
  }

  // Public: show picker with a captured image.
  // onSelect(imgX, imgY, imgW, imgH) called with image-space coords on confirm.
  show(imageBitmap, onSelect, onCancel) {
    this.#image    = imageBitmap;
    this.#imgW     = imageBitmap.width;
    this.#imgH     = imageBitmap.height;
    this.#onSelect = onSelect;
    this.#onCancel = onCancel;
    this.#sel      = null;
    this.#overlay.classList.remove('hidden');
    this.#resize();
    this.#draw();
  }

  hide() {
    this.#overlay.classList.add('hidden');
    this.#sel = null;
  }

  #resize() {
    const W = this.#overlay.clientWidth;
    const H = this.#overlay.clientHeight;
    const dpr = window.devicePixelRatio || 1;
    this.#canvas.width  = Math.floor(W * dpr);
    this.#canvas.height = Math.floor(H * dpr);
    this.#canvas.style.width  = W + 'px';
    this.#canvas.style.height = H + 'px';
    // Compute contain-scale and centering offsets (in CSS pixels)
    this.#scale = Math.min(W / this.#imgW, H / this.#imgH);
    this.#offX  = (W - this.#imgW * this.#scale) / 2;
    this.#offY  = (H - this.#imgH * this.#scale) / 2;
  }

  #draw() {
    if (!this.#image) return;
    const dpr = window.devicePixelRatio || 1;
    const cw  = this.#canvas.width;
    const ch  = this.#canvas.height;
    const ctx = this.#ctx;

    ctx.save();
    ctx.scale(dpr, dpr);  // work in CSS pixels

    const dispW = this.#imgW * this.#scale;
    const dispH = this.#imgH * this.#scale;

    // 1. Draw screenshot
    ctx.drawImage(this.#image, this.#offX, this.#offY, dispW, dispH);

    // 2. Dim overlay
    ctx.fillStyle = 'rgba(0,0,0,0.5)';
    ctx.fillRect(0, 0, cw / dpr, ch / dpr);

    if (this.#sel) {
      const { x, y, w, h } = this.#sel;

      // 3. Reveal selection by redrawing screenshot clipped to selection
      ctx.save();
      ctx.beginPath();
      ctx.rect(x, y, w, h);
      ctx.clip();
      ctx.drawImage(this.#image, this.#offX, this.#offY, dispW, dispH);
      ctx.restore();

      // 4. Selection border
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 1.5;
      ctx.setLineDash([5, 3]);
      ctx.strokeRect(x, y, w, h);

      // 5. Dimension label
      const iw = Math.round(Math.abs(w) / this.#scale);
      const ih = Math.round(Math.abs(h) / this.#scale);
      ctx.setLineDash([]);
      ctx.font = '12px monospace';
      ctx.fillStyle = 'rgba(0,0,0,0.7)';
      ctx.fillRect(x + 4, y + 4, 80, 20);
      ctx.fillStyle = '#fff';
      ctx.fillText(`${iw} × ${ih}`, x + 8, y + 17);
    }

    ctx.restore();
  }

  // Convert canvas CSS-pixel coord to image-space coord.
  #toImage(cx, cy) {
    return {
      x: (cx - this.#offX) / this.#scale,
      y: (cy - this.#offY) / this.#scale,
    };
  }

  _bindEvents() {
    this.#canvas.addEventListener('mousedown', e => {
      this.#dragging = true;
      this.#startX = e.offsetX;
      this.#startY = e.offsetY;
      this.#sel = null;
    });

    this.#canvas.addEventListener('mousemove', e => {
      if (!this.#dragging) return;
      const x = Math.min(this.#startX, e.offsetX);
      const y = Math.min(this.#startY, e.offsetY);
      const w = Math.abs(e.offsetX - this.#startX);
      const h = Math.abs(e.offsetY - this.#startY);
      this.#sel = { x, y, w, h };
      this.#draw();
    });

    this.#canvas.addEventListener('mouseup', e => {
      if (!this.#dragging) return;
      this.#dragging = false;
      if (!this.#sel || this.#sel.w < 4 || this.#sel.h < 4) {
        this.#sel = null;
        this.#draw();
        return;
      }
      // Convert to image-space, clamp to image bounds
      const tl = this.#toImage(this.#sel.x, this.#sel.y);
      const br = this.#toImage(this.#sel.x + this.#sel.w, this.#sel.y + this.#sel.h);
      const ix = Math.max(0, Math.round(tl.x));
      const iy = Math.max(0, Math.round(tl.y));
      const iw = Math.min(this.#imgW - ix, Math.round(br.x - tl.x));
      const ih = Math.min(this.#imgH - iy, Math.round(br.y - tl.y));
      this.hide();
      this.#onSelect?.(ix, iy, iw, ih);
    });

    window.addEventListener('keydown', e => {
      if (this.#overlay.classList.contains('hidden')) return;
      if (e.key === 'Escape') {
        this.hide();
        this.#onCancel?.();
      }
    });

    window.addEventListener('resize', () => {
      if (this.#overlay.classList.contains('hidden')) return;
      this.#resize();
      this.#draw();
    });
  }
}
