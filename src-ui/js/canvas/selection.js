/// Selection tool — select, move, resize, and delete annotations.

export class SelectionManager {
  #selected = null;

  select(annotation) {
    this.#selected = annotation;
  }

  deselect() {
    this.#selected = null;
  }

  get selected() {
    return this.#selected;
  }

  hitTest(x, y, annotations) {
    // TODO: iterate annotations in reverse order, check bounding box
    return null;
  }
}
