/// Triple-layer canvas engine.
///
/// Layer 0 (canvas-base):        Screenshot image — redrawn only on load/zoom/pan
/// Layer 1 (canvas-annotations): Committed annotations — redrawn on annotation change
/// Layer 2 (canvas-active):      Active tool preview — redrawn on every mouse move

export class CanvasEngine {
  #baseCanvas;
  #annoCanvas;
  #activeCanvas;
  #baseCtx;
  #annoCtx;
  #activeCtx;
  #image = null;
  #zoom = 1;
  #panX = 0;
  #panY = 0;

  constructor(baseCanvas, annoCanvas, activeCanvas) {
    this.#baseCanvas = baseCanvas;
    this.#annoCanvas = annoCanvas;
    this.#activeCanvas = activeCanvas;
    this.#baseCtx = baseCanvas.getContext('2d');
    this.#annoCtx = annoCanvas.getContext('2d');
    this.#activeCtx = activeCanvas.getContext('2d');

    // TODO: attach resize observer for dynamic sizing
  }

  async loadImage(dataUrl) {
    // Convert base64 data URL to Blob
    const response = await fetch(dataUrl);
    const blob = await response.blob();

    // Create ImageBitmap for efficient rendering
    this.#image = await createImageBitmap(blob);

    // Size all canvas layers to match image dimensions
    const width = this.#image.width;
    const height = this.#image.height;

    this.#baseCanvas.width = width;
    this.#baseCanvas.height = height;
    this.#annoCanvas.width = width;
    this.#annoCanvas.height = height;
    this.#activeCanvas.width = width;
    this.#activeCanvas.height = height;

    // Render the base layer
    this.renderBase();

    return { width, height };
  }

  screenToImage(screenX, screenY) {
    // TODO: apply inverse of current transform (zoom + pan)
    return {
      x: (screenX - this.#panX) / this.#zoom,
      y: (screenY - this.#panY) / this.#zoom,
    };
  }

  renderBase() {
    if (!this.#image) return;

    // Clear canvas
    this.#baseCtx.clearRect(0, 0, this.#baseCanvas.width, this.#baseCanvas.height);

    // Draw image at 1:1 scale (no zoom/pan for tracer-bullet)
    this.#baseCtx.drawImage(this.#image, 0, 0);
  }

  renderAnnotations(annotations) {
    if (!annotations) return;

    // Clear annotations canvas
    this.#annoCtx.clearRect(0, 0, this.#annoCanvas.width, this.#annoCanvas.height);

    // Draw each annotation
    for (const anno of annotations) {
      this.#drawShape(this.#annoCtx, anno);
    }
  }

  renderActive(previewShape) {
    // Clear active canvas
    this.#activeCtx.clearRect(0, 0, this.#activeCanvas.width, this.#activeCanvas.height);

    // Draw preview shape if provided
    if (previewShape) {
      this.#drawShape(this.#activeCtx, previewShape);
    }
  }

  #drawShape(ctx, shape) {
    if (!shape) return;

    ctx.strokeStyle = shape.stroke_color || '#FF0000';
    ctx.lineWidth = shape.stroke_width || 2;

    if (shape.type === 'rect') {
      ctx.strokeRect(shape.x, shape.y, shape.width, shape.height);
    }
    // TODO: add other shape types (ellipse, arrow, etc.) when implemented
  }

  exportComposite(annotations, format = 'png') {
    // TODO: create offscreen canvas at original image dimensions,
    // draw base image, draw all annotations at original scale, return as Blob
  }
}
