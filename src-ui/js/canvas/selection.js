/// Selection tool — select, move, resize, and delete annotations.

import { pointInRect } from '../utils/geometry.js';

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
    // Iterate in reverse order (last drawn = top-most)
    for (let i = annotations.length - 1; i >= 0; i--) {
      const anno = annotations[i];

      // Currently only support rect hit testing
      if (anno.type === 'rect') {
        if (pointInRect(x, y, anno)) {
          return { annotation: anno, index: i };
        }
      }
    }

    return null;
  }
}
