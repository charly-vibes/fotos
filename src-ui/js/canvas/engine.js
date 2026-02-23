/// Triple-layer canvas engine.
///
/// Layer 0 (canvas-base):        Screenshot image — redrawn only on load/zoom/pan
/// Layer 1 (canvas-annotations): Committed annotations — redrawn on annotation change
/// Layer 2 (canvas-active):      Active tool preview — redrawn on every mouse move
///
/// Coordinate systems:
///   - Image space:  pixels in the source image (used for annotation storage)
///   - CSS space:    device-independent pixels in the browser (mouse events)
///   - Backing store: physical pixels = CSS pixels × DPR
///
/// Transform: backing → CSS via ctx.setTransform(dpr,0,0,dpr,0,0)
///            CSS → image via translate(panX, panY) then scale(zoom, zoom)

export class CanvasEngine {
  #baseCanvas;
  #annoCanvas;
  #activeCanvas;
  #baseCtx;
  #annoCtx;
  #activeCtx;
  #container;
  #image = null;
  #zoom = 1;
  #panX = 0;
  #panY = 0;
  #dpr = 1;
  // Cached for re-render on resize
  #annotations = [];
  #selectedAnnotation = null;

  constructor(baseCanvas, annoCanvas, activeCanvas) {
    this.#baseCanvas = baseCanvas;
    this.#annoCanvas = annoCanvas;
    this.#activeCanvas = activeCanvas;
    this.#baseCtx = baseCanvas.getContext('2d');
    this.#annoCtx = annoCanvas.getContext('2d');
    this.#activeCtx = activeCanvas.getContext('2d');
    this.#container = baseCanvas.parentElement;
    this.#dpr = window.devicePixelRatio || 1;

    new ResizeObserver(() => this.#resize()).observe(this.#container);
    this.#watchDpr();
    this.#resize();
  }

  // Re-register DPR listener each time it fires (DPR changes when window moves monitors)
  #watchDpr() {
    matchMedia(`(resolution: ${this.#dpr}dppx)`).addEventListener('change', () => {
      this.#dpr = window.devicePixelRatio || 1;
      this.#resize();
      this.#watchDpr();
    }, { once: true });
  }

  #resize() {
    const w = Math.floor(this.#container.clientWidth * this.#dpr);
    const h = Math.floor(this.#container.clientHeight * this.#dpr);
    for (const canvas of [this.#baseCanvas, this.#annoCanvas, this.#activeCanvas]) {
      canvas.width = w;
      canvas.height = h;
      // CSS display size is handled by stylesheet (position:absolute; width:100%; height:100%)
    }
    this.#renderAll();
  }

  // Set context transform: backing-store scale (DPR) + pan + zoom.
  // After this call, all drawing coordinates are in image pixel space.
  #applyTransform(ctx) {
    ctx.setTransform(this.#dpr, 0, 0, this.#dpr, 0, 0);
    ctx.translate(this.#panX, this.#panY);
    ctx.scale(this.#zoom, this.#zoom);
  }

  #renderAll() {
    this.renderBase();
    this.#renderAnnotationsInternal();
    // Active layer clears on resize — don't re-show a stale preview
    this.#activeCtx.clearRect(0, 0, this.#activeCanvas.width, this.#activeCanvas.height);
  }

  // --- Public API ---

  async loadImage(dataUrl) {
    const response = await fetch(dataUrl);
    const blob = await response.blob();
    this.#image = await createImageBitmap(blob);
    this.#renderAll();
    return { width: this.#image.width, height: this.#image.height };
  }

  // Convert CSS-pixel mouse coordinates to image-pixel coordinates.
  screenToImage(screenX, screenY) {
    return {
      x: (screenX - this.#panX) / this.#zoom,
      y: (screenY - this.#panY) / this.#zoom,
    };
  }

  setZoom(z) {
    this.#zoom = Math.max(0.1, Math.min(10.0, z));
    this.#renderAll();
  }

  getZoom() { return this.#zoom; }
  zoomBy(factor) { this.setZoom(this.#zoom * factor); }

  setPan(x, y) {
    this.#panX = x;
    this.#panY = y;
    this.#renderAll();
  }

  getPan() { return { x: this.#panX, y: this.#panY }; }

  get hasImage() { return this.#image !== null; }
  get imageWidth() { return this.#image?.width ?? 0; }
  get imageHeight() { return this.#image?.height ?? 0; }

  renderBase() {
    const ctx = this.#baseCtx;
    ctx.clearRect(0, 0, this.#baseCanvas.width, this.#baseCanvas.height);

    if (!this.#image) {
      // Empty-state placeholder — draw in CSS pixel space (no zoom/pan)
      ctx.save();
      ctx.setTransform(this.#dpr, 0, 0, this.#dpr, 0, 0);
      const color = getComputedStyle(document.documentElement)
        .getPropertyValue('--text-secondary').trim() || '#888';
      ctx.fillStyle = color;
      ctx.font = '16px system-ui, sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(
        'Capture or open a screenshot to begin',
        this.#container.clientWidth / 2,
        this.#container.clientHeight / 2,
      );
      ctx.restore();
      return;
    }

    this.#applyTransform(ctx);
    ctx.drawImage(this.#image, 0, 0);
  }

  renderAnnotations(annotations, selectedAnnotation = null) {
    this.#annotations = annotations ?? [];
    this.#selectedAnnotation = selectedAnnotation ?? null;
    this.#renderAnnotationsInternal();
  }

  #renderAnnotationsInternal() {
    const ctx = this.#annoCtx;
    ctx.clearRect(0, 0, this.#annoCanvas.width, this.#annoCanvas.height);
    if (!this.#annotations.length && !this.#selectedAnnotation) return;

    this.#applyTransform(ctx);
    for (const anno of this.#annotations) {
      this.#drawShape(ctx, anno);
    }
    if (this.#selectedAnnotation) {
      this.#drawSelectionIndicator(ctx, this.#selectedAnnotation);
    }
  }

  renderActive(previewShape) {
    const ctx = this.#activeCtx;
    ctx.clearRect(0, 0, this.#activeCanvas.width, this.#activeCanvas.height);
    if (previewShape) {
      this.#applyTransform(ctx);
      this.#drawShape(ctx, previewShape);
    }
  }

  #drawShape(ctx, shape) {
    if (!shape) return;
    ctx.save();
    ctx.globalAlpha = shape.opacity ?? 1;
    ctx.lineWidth = shape.strokeWidth ?? 2;
    ctx.strokeStyle = shape.strokeColor || '#FF0000';

    if (shape.type === 'rect') {
      if (shape.fillColor && shape.fillColor !== 'transparent') {
        ctx.fillStyle = shape.fillColor;
        ctx.fillRect(shape.x, shape.y, shape.width, shape.height);
      }
      ctx.strokeRect(shape.x, shape.y, shape.width, shape.height);
    }
    ctx.restore();
  }

  #drawSelectionIndicator(ctx, shape) {
    if (!shape) return;
    ctx.save();
    ctx.strokeStyle = '#0066FF';
    // Keep selection border visually 2px regardless of zoom level
    ctx.lineWidth = 2 / this.#zoom;
    ctx.setLineDash([5 / this.#zoom, 5 / this.#zoom]);

    if (shape.type === 'rect') {
      const padding = 4 / this.#zoom;
      ctx.strokeRect(
        shape.x - padding,
        shape.y - padding,
        shape.width + padding * 2,
        shape.height + padding * 2,
      );
    }
    ctx.restore();
  }
}
