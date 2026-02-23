/// Command pattern for annotation creation.
/// Pairs with History in history.js (handles 100-command limit, FIFO eviction, redo clearing).

/// Command: crop the image to a rectangle.
/// Unlike annotation-only commands this also modifies the base image.
/// The command is added to history via history.record() (not execute()) because
/// the Tauri crop call happens before the command is created.
///
/// onUndo and onExecute are async callbacks the caller wires up for image
/// reloading.  They are fired via Promise.resolve().then() so undo/redo stay
/// synchronous from History's perspective.
export class CropCommand {
  #oldImageId;
  #newImageId;
  #oldAnnotations;
  #newAnnotations;
  #oldDataUrl;
  #newDataUrl;

  /// Fired on undo: (oldImageId, oldAnnotations, oldDataUrl) => void
  onUndo = null;
  /// Fired on redo (NOT on first execute via record()): same signature
  onExecute = null;

  constructor({ oldImageId, newImageId, oldAnnotations, newAnnotations, oldDataUrl, newDataUrl }) {
    this.#oldImageId = oldImageId;
    this.#newImageId = newImageId;
    this.#oldAnnotations = oldAnnotations;
    this.#newAnnotations = newAnnotations;
    this.#oldDataUrl = oldDataUrl;
    this.#newDataUrl = newDataUrl;
  }

  // Called on redo by History.
  execute(_annotations) {
    const cb = this.onExecute;
    if (cb) Promise.resolve().then(() => cb(this.#newImageId, this.#newAnnotations, this.#newDataUrl));
    return this.#newAnnotations;
  }

  undo(_annotations) {
    const cb = this.onUndo;
    if (cb) Promise.resolve().then(() => cb(this.#oldImageId, this.#oldAnnotations, this.#oldDataUrl));
    return this.#oldAnnotations;
  }
}

export class AddAnnotationCommand {
  #annotation;

  constructor(annotation) {
    this.#annotation = annotation;
  }

  execute(annotations) {
    return [...annotations, this.#annotation];
  }

  undo(annotations) {
    return annotations.filter(a => a.id !== this.#annotation.id);
  }
}
