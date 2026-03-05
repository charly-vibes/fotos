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

// Reorder annotation from oldIndex to newIndex (after insert-at semantics).
export class ZOrderCommand {
  #id;
  #fromIndex;
  #toIndex;

  constructor(id, fromIndex, toIndex) {
    this.#id = id;
    this.#fromIndex = fromIndex;
    this.#toIndex = toIndex;
  }

  #reorder(annotations, from, to) {
    const arr = [...annotations];
    const [item] = arr.splice(from, 1);
    arr.splice(to, 0, item);
    return arr;
  }

  execute(annotations) {
    return this.#reorder(annotations, this.#fromIndex, this.#toIndex);
  }

  undo(annotations) {
    return this.#reorder(annotations, this.#toIndex, this.#fromIndex);
  }
}

// Toggle the locked state of a single annotation.
export class LockCommand {
  #id;
  #lock; // true = locking, false = unlocking

  constructor(id, lock) {
    this.#id = id;
    this.#lock = lock;
  }

  execute(annotations) {
    return annotations.map(a => a.id === this.#id ? { ...a, locked: this.#lock } : a);
  }

  undo(annotations) {
    return annotations.map(a => a.id === this.#id ? { ...a, locked: !this.#lock } : a);
  }
}

// Batch transform: stores before/after snapshots for multiple annotations.
// Also works for a single annotation (replaces TransformAnnotationCommand in multi-select flow).
export class BatchTransformCommand {
  #pairs; // [{before, after}, ...]

  constructor(befores, afters) {
    this.#pairs = befores.map((b, i) => ({
      before: { ...b, points: b.points ? b.points.map(p => ({ ...p })) : [] },
      after:  { ...afters[i], points: afters[i].points ? afters[i].points.map(p => ({ ...p })) : [] },
    }));
  }

  execute(annotations) {
    const afterMap = new Map(this.#pairs.map(p => [p.after.id, p.after]));
    return annotations.map(a => afterMap.get(a.id) ?? a);
  }

  undo(annotations) {
    const beforeMap = new Map(this.#pairs.map(p => [p.before.id, p.before]));
    return annotations.map(a => beforeMap.get(a.id) ?? a);
  }
}

// Batch delete: removes multiple annotations atomically.
// items: [{annotation, index}, ...] where index is the position in the array at delete time.
export class BatchDeleteCommand {
  #deletedItems; // sorted by descending index for correct splice order

  constructor(items) {
    this.#deletedItems = [...items].sort((a, b) => b.index - a.index);
  }

  execute(annotations) {
    const arr = [...annotations];
    for (const { index } of this.#deletedItems) arr.splice(index, 1);
    return arr;
  }

  undo(annotations) {
    const arr = [...annotations];
    // Restore in ascending index order (reverse of descending sort).
    for (const { annotation, index } of [...this.#deletedItems].reverse()) {
      arr.splice(index, 0, annotation);
    }
    return arr;
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
