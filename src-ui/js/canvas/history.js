/// Command pattern for undo/redo.
/// Stores deltas, not full canvas snapshots.

export class History {
  #undoStack = [];
  #redoStack = [];
  #maxSize = 100;

  execute(command, annotations) {
    const result = command.execute(annotations);
    this.#undoStack.push(command);
    this.#redoStack = [];
    if (this.#undoStack.length > this.#maxSize) this.#undoStack.shift();
    return result;
  }

  undo(annotations) {
    const cmd = this.#undoStack.pop();
    if (!cmd) return annotations;
    this.#redoStack.push(cmd);
    return cmd.undo(annotations);
  }

  redo(annotations) {
    const cmd = this.#redoStack.pop();
    if (!cmd) return annotations;
    this.#undoStack.push(cmd);
    return cmd.execute(annotations);
  }

  // Add a pre-executed command to the undo stack without calling execute().
  // Used for operations like crop that are applied via async side-channels
  // before the command object is created.
  record(cmd) {
    this.#undoStack.push(cmd);
    this.#redoStack = [];
    if (this.#undoStack.length > this.#maxSize) this.#undoStack.shift();
  }

  get canUndo() { return this.#undoStack.length > 0; }
  get canRedo() { return this.#redoStack.length > 0; }
}

/// Command: delete an annotation.
/// Delta stores the deleted annotation and its index.
export class DeleteCommand {
  #deletedAnnotation;
  #deletedIndex;

  constructor(annotation, index) {
    this.#deletedAnnotation = annotation;
    this.#deletedIndex = index;
  }

  execute(annotations) {
    // Remove annotation from array
    const newAnnotations = [...annotations];
    newAnnotations.splice(this.#deletedIndex, 1);
    return newAnnotations;
  }

  undo(annotations) {
    // Restore annotation at original index
    const newAnnotations = [...annotations];
    newAnnotations.splice(this.#deletedIndex, 0, this.#deletedAnnotation);
    return newAnnotations;
  }
}
