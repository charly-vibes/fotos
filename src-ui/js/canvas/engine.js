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
    // WebKitGTK does not support fetch() for data: URLs (throws "TypeError: Load failed").
    // Decode the data URL directly into a Blob instead.
    const [header, base64] = dataUrl.split(',', 2);
    const mimeType = header.match(/:(.*?);/)[1];
    const bytes = atob(base64);
    const array = new Uint8Array(bytes.length);
    for (let i = 0; i < bytes.length; i++) array[i] = bytes.charCodeAt(i);
    const blob = new Blob([array], { type: mimeType });
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

  // Set zoom and pan in one call — avoids the double render that causes artifacts.
  setZoomAndPan(z, panX, panY) {
    this.#zoom = Math.max(0.1, Math.min(10.0, z));
    this.#panX = panX;
    this.#panY = panY;
    this.#renderAll();
  }

  setPan(x, y) {
    this.#panX = x;
    this.#panY = y;
    this.#renderAll();
  }

  getPan() { return { x: this.#panX, y: this.#panY }; }

  // Scale image to fit the container, centered, with padding.  Returns the zoom level.
  fitToPage() {
    if (!this.#image) return this.#zoom;
    const cw = this.#container.clientWidth;
    const ch = this.#container.clientHeight;
    const pad = 20;
    const z = Math.min(
      (cw - pad * 2) / this.#image.width,
      (ch - pad * 2) / this.#image.height,
      10.0,
    );
    this.#zoom = Math.max(0.05, z);
    this.#panX = (cw - this.#image.width * this.#zoom) / 2;
    this.#panY = (ch - this.#image.height * this.#zoom) / 2;
    this.#renderAll();
    return this.#zoom;
  }

  get hasImage() { return this.#image !== null; }
  get imageWidth() { return this.#image?.width ?? 0; }
  get imageHeight() { return this.#image?.height ?? 0; }

  renderBase() {
    const ctx = this.#baseCtx;
    ctx.resetTransform();
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
    ctx.resetTransform();
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
    ctx.resetTransform();
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

    switch (shape.type) {
      case 'rect': {
        if (shape.fillColor && shape.fillColor !== 'transparent') {
          ctx.fillStyle = shape.fillColor;
          ctx.fillRect(shape.x, shape.y, shape.width, shape.height);
        }
        ctx.strokeRect(shape.x, shape.y, shape.width, shape.height);
        break;
      }

      case 'arrow': {
        const pts = shape.points;
        if (!pts || pts.length < 2) break;
        const [p1, p2] = pts;
        ctx.beginPath();
        ctx.moveTo(p1.x, p1.y);
        ctx.lineTo(p2.x, p2.y);
        ctx.stroke();
        // Filled arrowhead triangle at p2
        const headLen = Math.max((shape.strokeWidth ?? 2) * 5, 12);
        const angle = Math.atan2(p2.y - p1.y, p2.x - p1.x);
        const wing = Math.PI / 6;
        ctx.beginPath();
        ctx.moveTo(p2.x, p2.y);
        ctx.lineTo(p2.x - headLen * Math.cos(angle - wing), p2.y - headLen * Math.sin(angle - wing));
        ctx.lineTo(p2.x - headLen * Math.cos(angle + wing), p2.y - headLen * Math.sin(angle + wing));
        ctx.closePath();
        ctx.fillStyle = shape.strokeColor || '#FF0000';
        ctx.fill();
        break;
      }

      case 'ellipse': {
        const cx = shape.x + shape.width / 2;
        const cy = shape.y + shape.height / 2;
        ctx.beginPath();
        ctx.ellipse(cx, cy, Math.abs(shape.width / 2), Math.abs(shape.height / 2), 0, 0, Math.PI * 2);
        if (shape.fillColor && shape.fillColor !== 'transparent') {
          ctx.fillStyle = shape.fillColor;
          ctx.fill();
        }
        ctx.stroke();
        break;
      }

      case 'text': {
        ctx.font = `${shape.fontSize || 20}px ${shape.fontFamily || 'sans-serif'}`;
        ctx.fillStyle = shape.strokeColor || '#FF0000';
        ctx.textBaseline = 'top';
        ctx.fillText(shape.text || '', shape.x, shape.y);
        break;
      }

      case 'blur': {
        if (!this.#image) break;
        const blockSize = Math.max(1, shape.blurRadius || 10);
        const sw = Math.max(1, Math.ceil(shape.width / blockSize));
        const sh = Math.max(1, Math.ceil(shape.height / blockSize));
        const off = new OffscreenCanvas(sw, sh);
        const offCtx = off.getContext('2d');
        offCtx.imageSmoothingEnabled = false;
        offCtx.drawImage(this.#image, shape.x, shape.y, shape.width, shape.height, 0, 0, sw, sh);
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(off, 0, 0, sw, sh, shape.x, shape.y, shape.width, shape.height);
        ctx.imageSmoothingEnabled = true;
        break;
      }

      case 'step': {
        const size = shape.fontSize || 24;
        const radius = size / 2;
        ctx.beginPath();
        ctx.arc(shape.x, shape.y, radius, 0, Math.PI * 2);
        ctx.fillStyle = shape.strokeColor || '#FF0000';
        ctx.fill();
        ctx.fillStyle = '#FFFFFF';
        ctx.font = `bold ${Math.floor(size * 0.6)}px sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(String(shape.stepNumber ?? 1), shape.x, shape.y);
        break;
      }

      case 'freehand': {
        const pts = shape.points;
        if (!pts || pts.length < 2) break;
        ctx.beginPath();
        ctx.moveTo(pts[0].x, pts[0].y);
        for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i].x, pts[i].y);
        ctx.stroke();
        break;
      }

      case 'highlight': {
        // Always 0.4 opacity per spec, regardless of shape.opacity
        ctx.globalAlpha = 0.4;
        ctx.fillStyle = shape.highlightColor || '#FFFF00';
        ctx.fillRect(shape.x, shape.y, shape.width, shape.height);
        break;
      }
    }

    ctx.restore();
  }

  // Returns bounding box {x, y, w, h} for selection indicator.
  #getShapeBBox(shape) {
    switch (shape.type) {
      case 'rect':
      case 'ellipse':
      case 'blur':
      case 'highlight':
        return { x: shape.x, y: shape.y, w: shape.width || 0, h: shape.height || 0 };
      case 'arrow':
      case 'freehand': {
        const pts = shape.points;
        if (!pts || pts.length === 0) return null;
        const xs = pts.map(p => p.x);
        const ys = pts.map(p => p.y);
        const minX = Math.min(...xs), maxX = Math.max(...xs);
        const minY = Math.min(...ys), maxY = Math.max(...ys);
        return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
      }
      case 'text': {
        const size = shape.fontSize || 20;
        const approxW = size * (shape.text?.length || 1) * 0.6;
        return { x: shape.x, y: shape.y, w: approxW, h: size * 1.4 };
      }
      case 'step': {
        const r = (shape.fontSize || 24) / 2;
        return { x: shape.x - r, y: shape.y - r, w: r * 2, h: r * 2 };
      }
      default:
        return null;
    }
  }

  #drawSelectionIndicator(ctx, shape) {
    if (!shape) return;
    const bbox = this.#getShapeBBox(shape);
    if (!bbox) return;
    ctx.save();
    ctx.strokeStyle = '#0066FF';
    ctx.lineWidth = 2 / this.#zoom;
    ctx.setLineDash([5 / this.#zoom, 5 / this.#zoom]);
    const p = 4 / this.#zoom;
    ctx.strokeRect(bbox.x - p, bbox.y - p, bbox.w + p * 2, bbox.h + p * 2);
    ctx.restore();
  }

  // Draw resize/move handles for the selected annotation on the active layer.
  // handles: array of {x, y} in image coords, or null/[] to clear.
  renderHandles(handles) {
    const ctx = this.#activeCtx;
    ctx.resetTransform();
    ctx.clearRect(0, 0, this.#activeCanvas.width, this.#activeCanvas.height);
    if (!handles || handles.length === 0) return;

    this.#applyTransform(ctx);
    const hs = 5 / this.#zoom; // half-size of each handle square, in image pixels
    ctx.save();
    ctx.fillStyle = '#FFFFFF';
    ctx.strokeStyle = '#0066FF';
    ctx.lineWidth = 1.5 / this.#zoom;
    for (const h of handles) {
      ctx.fillRect(h.x - hs, h.y - hs, hs * 2, hs * 2);
      ctx.strokeRect(h.x - hs, h.y - hs, hs * 2, hs * 2);
    }
    ctx.restore();
  }

  // Handle radius in image pixels for hit testing (matches renderHandles size).
  handleHitRadius() {
    return 7 / (this.#zoom || 1);
  }

  // Draw a crop selection overlay: dims everything outside rect and shows a
  // dashed border.  Pass null to clear.
  renderCropOverlay(rect) {
    const ctx = this.#activeCtx;
    ctx.clearRect(0, 0, this.#activeCanvas.width, this.#activeCanvas.height);
    if (!rect) return;

    const W = this.#container.clientWidth;
    const H = this.#container.clientHeight;

    // Semi-transparent dark overlay (CSS pixel space)
    ctx.save();
    ctx.setTransform(this.#dpr, 0, 0, this.#dpr, 0, 0);
    ctx.fillStyle = 'rgba(0,0,0,0.5)';
    ctx.fillRect(0, 0, W, H);
    ctx.restore();

    // Punch a hole for the selected region (image space)
    ctx.save();
    ctx.globalCompositeOperation = 'destination-out';
    this.#applyTransform(ctx);
    ctx.fillStyle = '#000';
    ctx.fillRect(rect.x, rect.y, rect.width, rect.height);
    ctx.restore();

    // White dashed border (image space)
    ctx.save();
    this.#applyTransform(ctx);
    ctx.strokeStyle = '#FFFFFF';
    ctx.lineWidth = 1 / this.#zoom;
    ctx.setLineDash([5 / this.#zoom, 3 / this.#zoom]);
    ctx.strokeRect(rect.x, rect.y, rect.width, rect.height);
    ctx.restore();
  }
}
