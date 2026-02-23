/// Command pattern for annotation creation.
/// Pairs with History in history.js (handles 100-command limit, FIFO eviction, redo clearing).

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
