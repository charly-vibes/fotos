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

  get canUndo() { return this.#undoStack.length > 0; }
  get canRedo() { return this.#redoStack.length > 0; }
}
