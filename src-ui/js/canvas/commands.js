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

// Stores a before/after snapshot of a single annotation.
// Used by move and resize operations.
export class TransformAnnotationCommand {
  #before;
  #after;

  constructor(before, after) {
    this.#before = { ...before, points: before.points ? before.points.map(p => ({ ...p })) : [] };
    this.#after  = { ...after,  points: after.points  ? after.points.map(p => ({ ...p })) : [] };
  }

  execute(annotations) {
    return annotations.map(a => a.id === this.#after.id ? this.#after : a);
  }

  undo(annotations) {
    return annotations.map(a => a.id === this.#before.id ? this.#before : a);
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
