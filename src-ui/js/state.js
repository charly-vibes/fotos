/// Central state store with event emitter pattern.
/// No framework — simple pub/sub for reactive UI updates.

class StateStore {
  #state = {};
  #listeners = new Map();

  constructor(initial) {
    this.#state = structuredClone(initial);
  }

  get(key) {
    return this.#state[key];
  }

  set(key, value) {
    const old = this.#state[key];
    this.#state[key] = value;
    if (old !== value) this.#emit(key, value, old);
  }

  on(key, fn) {
    if (!this.#listeners.has(key)) this.#listeners.set(key, new Set());
    this.#listeners.get(key).add(fn);
    return () => this.#listeners.get(key).delete(fn);
  }

  #emit(key, value, old) {
    this.#listeners.get(key)?.forEach(fn => fn(value, old));
  }
}

export const store = new StateStore({
  activeTool: 'arrow',
  strokeColor: '#FF0000',
  fillColor: 'transparent',
  strokeWidth: 2,
  fontSize: 16,
  opacity: 1.0,
  zoom: 1.0,
  panX: 0,
  panY: 0,
  currentImageId: null,
  annotations: [],
  undoStack: [],
  redoStack: [],
  nextStepNumber: 1,
  ocrResults: null,
  llmResults: null,
  isProcessing: false,
});
