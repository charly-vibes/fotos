/// Triple-layer canvas engine.
///
/// Layer 0 (canvas-base):        Screenshot image — redrawn only on load/zoom/pan
/// Layer 1 (canvas-annotations): Committed annotations — redrawn on annotation change
/// Layer 2 (canvas-active):      Active tool preview — redrawn on every mouse move

export class CanvasEngine {
  #baseCtx;
  #annoCtx;
  #activeCtx;
  #image = null;
  #zoom = 1;
  #panX = 0;
  #panY = 0;

  constructor(baseCanvas, annoCanvas, activeCanvas) {
    this.#baseCtx = baseCanvas.getContext('2d');
    this.#annoCtx = annoCanvas.getContext('2d');
    this.#activeCtx = activeCanvas.getContext('2d');

    // TODO: size canvases, attach resize observer
  }

  loadImage(imageData) {
    // TODO: create ImageBitmap from ArrayBuffer, store, trigger redraw
  }

  screenToImage(screenX, screenY) {
    // TODO: apply inverse of current transform (zoom + pan)
    return {
      x: (screenX - this.#panX) / this.#zoom,
      y: (screenY - this.#panY) / this.#zoom,
    };
  }

  renderBase() {
    // TODO: clear, apply transform, drawImage
  }

  renderAnnotations(annotations) {
    // TODO: clear, apply transform, iterate annotations, draw each
  }

  renderActive(previewShape) {
    // TODO: clear, apply transform, draw single shape
  }

  exportComposite(annotations, format = 'png') {
    // TODO: create offscreen canvas at original image dimensions,
    // draw base image, draw all annotations at original scale, return as Blob
  }
}
