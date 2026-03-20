Selection & History Implementation

Implemented the command pattern for undo/redo with delete as the first operation.

## Key Design Decisions:

1. **DeleteCommand Delta Storage**: Stores the deleted annotation object and its index (not a full array snapshot). This follows the 'deltas not snapshots' principle for efficient memory usage.

2. **Hit Testing**: Iterates annotations in reverse order (top-most first) to match visual layering. Returns both the annotation and its index for efficient deletion.

3. **Selection Indicator**: Blue dashed border with 4px padding drawn on annotation canvas layer. Simple and clear visual feedback.

4. **Selection Scope**: Click-to-select works globally (not limited to 'select' tool mode), but only when not actively drawing with the rectangle tool. This provides intuitive UX.

5. **Undo Behavior**: Always deselects after undo to avoid stale selection references.

## Implementation:
- DeleteCommand class in canvas/history.js
- SelectionManager.hitTest() with pointInRect utility
- Canvas engine #drawSelectionIndicator() method
- Keyboard shortcuts: Delete and Ctrl+Z in app.js
- Selection state managed by SelectionManager (not in store)

Ready for end-to-end smoke test.
