# Selection & History

Capability spec for annotation selection, manipulation, and undo/redo history in Fotos (`io.github.charly.fotos`).

## Purpose

This capability covers the selection tool (select, move, resize, delete annotations), select-all, and the command-pattern undo/redo system with history limits.

**Source files:**
- `src-ui/js/canvas/selection.js` — Object select, move, resize handles
- `src-ui/js/canvas/history.js` — Command pattern undo/redo

## Requirements

### Requirement: Selection Tool

The application SHALL provide a Selection tool that allows the user to select an annotation by clicking on it. The Selection tool SHALL be activated by pressing the `V` key or by clicking the Select button in the toolbar. When the Selection tool is active, clicking on an annotation SHALL select it, rendering visible selection handles around its bounding box. Clicking on an empty area of the canvas SHALL deselect any currently selected annotation.

#### Scenario: Activate selection tool via keyboard
- **WHEN** the user presses the `V` key and no text input is focused
- **THEN** the active tool SHALL change to the Selection tool
- **THEN** the toolbar SHALL visually indicate that the Select tool is active

#### Scenario: Select annotation by clicking
- **WHEN** the Selection tool is active and the user clicks on an annotation
- **THEN** that annotation SHALL become selected
- **THEN** selection handles SHALL be rendered around the annotation's bounding box

#### Scenario: Click empty canvas area to deselect
- **WHEN** the Selection tool is active and an annotation is selected and the user clicks on an empty area of the canvas
- **THEN** the annotation SHALL be deselected
- **THEN** selection handles SHALL be removed

#### Scenario: Deselect via Escape key
- **WHEN** an annotation is selected and the user presses the `Escape` key
- **THEN** the selection SHALL be cleared
- **THEN** selection handles SHALL be removed

---

### Requirement: Move Annotation

The application SHALL allow the user to move a selected annotation by clicking and dragging it to a new position. The move operation SHALL update the annotation's coordinates in image space. The move SHALL be recorded as a command in the undo/redo history, storing the original and destination positions as a delta.

#### Scenario: Drag selected annotation to new position
- **WHEN** an annotation is selected and the user clicks inside the annotation (not on a resize handle) and drags to a new position
- **THEN** the annotation SHALL follow the pointer during the drag, rendered on the active (preview) canvas layer
- **THEN** on mouse release the annotation's position SHALL be updated to the new coordinates
- **THEN** a MoveAnnotationCommand SHALL be pushed onto the undo stack with the original and final positions

#### Scenario: Move preserves annotation properties
- **WHEN** an annotation is moved to a new position
- **THEN** all annotation properties other than position (stroke color, fill, text content, dimensions) SHALL remain unchanged

---

### Requirement: Resize Annotation

The application SHALL render resize handles on the bounding box of a selected annotation. The user SHALL be able to drag these handles to resize the annotation. The resize operation SHALL be recorded as a command in the undo/redo history, storing the original and new geometry as a delta.

#### Scenario: Resize via corner handle
- **WHEN** an annotation is selected and the user clicks on a corner resize handle and drags
- **THEN** the annotation's bounding box SHALL resize proportionally from the opposite corner
- **THEN** the resize preview SHALL be rendered on the active canvas layer during the drag
- **THEN** on mouse release, a resize command SHALL be pushed onto the undo stack with the original and new geometry

#### Scenario: Resize via edge handle
- **WHEN** an annotation is selected and the user clicks on an edge resize handle and drags
- **THEN** the annotation SHALL resize along the axis of that edge
- **THEN** on mouse release, a resize command SHALL be pushed onto the undo stack

---

### Requirement: Delete Annotation

The application SHALL allow the user to delete a selected annotation by pressing the `Delete` key. The deletion SHALL be recorded as a command in the undo/redo history so it can be undone.

#### Scenario: Delete selected annotation via Delete key
- **WHEN** an annotation is selected and the user presses the `Delete` key
- **THEN** the selected annotation SHALL be removed from the annotations array
- **THEN** a DeleteAnnotationCommand SHALL be pushed onto the undo stack
- **THEN** the annotation layer SHALL be re-rendered without the deleted annotation

#### Scenario: Delete with nothing selected
- **WHEN** no annotation is selected and the user presses the `Delete` key
- **THEN** no action SHALL be taken and the annotations array SHALL remain unchanged

---

### Requirement: Select All Annotations

The application SHALL allow the user to select all annotations on the canvas by pressing `Ctrl+A`. When multiple annotations are selected, move and delete operations SHALL apply to all selected annotations as a group.

#### Scenario: Select all via Ctrl+A
- **WHEN** the user presses `Ctrl+A` and at least one annotation exists on the canvas
- **THEN** all annotations SHALL become selected
- **THEN** selection handles SHALL be rendered around the combined bounding box of all selected annotations

#### Scenario: Select all with no annotations
- **WHEN** the user presses `Ctrl+A` and no annotations exist on the canvas
- **THEN** no selection SHALL be created and no error SHALL occur

---

### Requirement: Undo via Command Pattern

The application SHALL implement undo using the command pattern. Each annotation action (add, move, resize, delete, modify style) SHALL be represented as a command object with `execute()` and `undo()` methods. Commands SHALL store deltas (before/after state) rather than full canvas snapshots. Pressing `Ctrl+Z` SHALL undo the most recent command by calling its `undo()` method and moving it to the redo stack.

#### Scenario: Undo the most recent action
- **WHEN** the undo stack contains at least one command and the user presses `Ctrl+Z`
- **THEN** the most recent command SHALL be popped from the undo stack
- **THEN** the command's `undo()` method SHALL be called to reverse the action
- **THEN** the command SHALL be pushed onto the redo stack
- **THEN** the annotation layer SHALL be re-rendered to reflect the reverted state

#### Scenario: Undo when undo stack is empty
- **WHEN** the undo stack is empty and the user presses `Ctrl+Z`
- **THEN** no action SHALL be taken and the canvas SHALL remain unchanged

#### Scenario: Commands store deltas not snapshots
- **WHEN** a MoveAnnotationCommand is created
- **THEN** it SHALL store only the annotation ID, the original position, and the destination position
- **THEN** it SHALL NOT store a full copy of the annotations array or a canvas bitmap

---

### Requirement: Redo

The application SHALL implement redo by re-executing undone commands. Pressing `Ctrl+Shift+Z` SHALL pop the most recent command from the redo stack, call its `execute()` method, and push it back onto the undo stack. Performing any new action (add, move, resize, delete, modify) SHALL clear the redo stack entirely.

#### Scenario: Redo an undone action
- **WHEN** the redo stack contains at least one command and the user presses `Ctrl+Shift+Z`
- **THEN** the most recent command SHALL be popped from the redo stack
- **THEN** the command's `execute()` method SHALL be called to reapply the action
- **THEN** the command SHALL be pushed onto the undo stack
- **THEN** the annotation layer SHALL be re-rendered to reflect the reapplied state

#### Scenario: Redo when redo stack is empty
- **WHEN** the redo stack is empty and the user presses `Ctrl+Shift+Z`
- **THEN** no action SHALL be taken and the canvas SHALL remain unchanged

#### Scenario: New action clears redo stack
- **WHEN** the redo stack contains commands and the user performs a new annotation action (e.g., adds an annotation)
- **THEN** the redo stack SHALL be cleared entirely
- **THEN** the new command SHALL be pushed onto the undo stack

---

### Requirement: History Limit

The undo history SHALL be limited to a maximum of 100 commands. When the undo stack exceeds 100 entries, the oldest command SHALL be evicted using FIFO (first-in, first-out) order. The redo stack is not independently limited; it is bounded by the undo stack size since redo entries originate from undone commands.

#### Scenario: Evict oldest command when limit exceeded
- **WHEN** the undo stack contains 100 commands and a new command is executed
- **THEN** the oldest (first) command in the undo stack SHALL be removed
- **THEN** the new command SHALL be pushed onto the end of the undo stack
- **THEN** the undo stack size SHALL remain at 100

#### Scenario: History within limit
- **WHEN** the undo stack contains fewer than 100 commands and a new command is executed
- **THEN** the new command SHALL be appended to the undo stack without evicting any existing commands
