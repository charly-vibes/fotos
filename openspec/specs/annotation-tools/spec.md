# Capability: Annotation Tools

## Purpose

Annotation tools for Fotos (io.github.charly.fotos) -- the drawing and manipulation tools available in the canvas annotation toolbar. Each tool is activated by a single-key shortcut and produces a serializable annotation object stored in the central annotations array.

## Requirements

### Requirement: Arrow Tool

The system SHALL provide an arrow annotation tool, activated by the keyboard shortcut **A**, that draws a straight line between two points with an arrowhead at the endpoint.

The user clicks (or presses) to set the start point and drags to the end point. While dragging, the active-tool canvas layer SHALL render a live preview of the arrow. On release, the arrow annotation SHALL be committed to the annotations array.

The resulting annotation object SHALL have `type: "arrow"` and store its geometry in the `points` array as exactly two entries (`[{x, y}, {x, y}]`) in image coordinates. The arrowhead SHALL be rendered at the second point. The arrow SHALL respect the current `strokeColor`, `strokeWidth`, and `opacity` style properties.

#### Scenario: Draw an arrow between two points
- **WHEN** the Arrow tool is active and the user presses at point (100, 200) and drags to point (400, 300) and releases
- **THEN** an annotation with `type: "arrow"` SHALL be added to the annotations array with `points: [{x: 100, y: 200}, {x: 400, y: 300}]` in image coordinates, rendered with the current stroke color, stroke width, and opacity, and an arrowhead at the endpoint

#### Scenario: Live preview while dragging
- **WHEN** the Arrow tool is active and the user is dragging from the start point
- **THEN** the active-tool canvas layer SHALL continuously render a preview arrow from the start point to the current cursor position without committing an annotation

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **A** key and no text input is focused
- **THEN** the active tool SHALL switch to the Arrow tool

---

### Requirement: Rectangle Tool

The system SHALL provide a rectangle annotation tool, activated by the keyboard shortcut **R**, that draws an axis-aligned rectangle defined by a bounding box.

The user clicks to set one corner and drags to the opposite corner. The rectangle SHALL be defined by `x`, `y`, `width`, and `height` in image coordinates. The resulting annotation object SHALL have `type: "rect"` and respect the current `strokeColor`, `fillColor`, `strokeWidth`, and `opacity` style properties.

#### Scenario: Draw a rectangle
- **WHEN** the Rectangle tool is active and the user presses at point (50, 50) and drags to point (250, 150) and releases
- **THEN** an annotation with `type: "rect"` SHALL be added with `x: 50`, `y: 50`, `width: 200`, `height: 100` in image coordinates, rendered with the current stroke and fill styles

#### Scenario: Rectangle with negative drag direction
- **WHEN** the user drags from bottom-right to top-left (i.e., the release point has smaller x/y than the press point)
- **THEN** the system SHALL normalize the coordinates so that `x` and `y` represent the top-left corner and `width` and `height` are positive values

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **R** key and no text input is focused
- **THEN** the active tool SHALL switch to the Rectangle tool

---

### Requirement: Ellipse Tool

The system SHALL provide an ellipse annotation tool, activated by the keyboard shortcut **E**, that draws an axis-aligned ellipse inscribed within a bounding box.

The user clicks to set one corner of the bounding box and drags to the opposite corner. The ellipse SHALL be defined by `x`, `y`, `width`, and `height` representing the bounding rectangle in image coordinates. The resulting annotation object SHALL have `type: "ellipse"` and respect the current `strokeColor`, `fillColor`, `strokeWidth`, and `opacity` style properties.

#### Scenario: Draw an ellipse
- **WHEN** the Ellipse tool is active and the user presses at point (100, 100) and drags to point (300, 200) and releases
- **THEN** an annotation with `type: "ellipse"` SHALL be added with `x: 100`, `y: 100`, `width: 200`, `height: 100`, rendered as an ellipse inscribed within that bounding box

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **E** key and no text input is focused
- **THEN** the active tool SHALL switch to the Ellipse tool

---

### Requirement: Text Tool

The system SHALL provide a text annotation tool, activated by the keyboard shortcut **T**, that places editable text on the canvas.

When the user clicks a point on the canvas with the Text tool active, a floating `<textarea>` input SHALL appear at that position, allowing the user to type text. When the user confirms the text (by clicking outside the textarea or pressing Escape), the system SHALL commit a text annotation at that location.

The resulting annotation object SHALL have `type: "text"` and store the text content in the `text` field. The annotation SHALL include `fontSize` and `fontFamily` properties and respect the current `strokeColor` (used as text color) and `opacity`. The annotation position SHALL be stored in `x` and `y` in image coordinates.

#### Scenario: Place and edit a text annotation
- **WHEN** the Text tool is active and the user clicks at point (200, 150)
- **THEN** a floating textarea SHALL appear at the corresponding screen position, and upon the user typing "Bug here" and clicking outside the textarea, an annotation with `type: "text"`, `text: "Bug here"`, `x: 200`, `y: 150` SHALL be committed with the current `fontSize`, `fontFamily`, stroke color, and opacity

#### Scenario: Cancel empty text input
- **WHEN** the user opens the floating textarea and confirms without typing any text
- **THEN** no annotation SHALL be added to the annotations array

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **T** key and no text input is focused
- **THEN** the active tool SHALL switch to the Text tool

---

### Requirement: Blur Tool

The system SHALL provide a blur annotation tool, activated by the keyboard shortcut **B**, that applies a pixelation effect (block-averaging) to a rectangular region of the image. The effect SHALL divide the region into blocks and replace each block with its average color, producing a mosaic appearance.

The user clicks to set one corner and drags to the opposite corner of the region to blur. The resulting annotation object SHALL have `type: "blur"` with `x`, `y`, `width`, and `height` defining the region in image coordinates, and a `blurRadius` property controlling the pixelation block size in image pixels. The `blurRadius` SHALL default to the value from settings (`annotation.blurRadius`, default 10) and MUST be configurable.

#### Scenario: Blur a rectangular region
- **WHEN** the Blur tool is active and the user drags to select a region from (300, 100) to (500, 200) and releases
- **THEN** an annotation with `type: "blur"`, `x: 300`, `y: 100`, `width: 200`, `height: 100`, and `blurRadius: 10` (default) SHALL be added, and the canvas SHALL render that region with a pixelation effect

#### Scenario: Configurable blur radius
- **WHEN** the user changes the blur radius setting to 20 and then uses the Blur tool
- **THEN** the resulting blur annotation SHALL have `blurRadius: 20`

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **B** key and no text input is focused
- **THEN** the active tool SHALL switch to the Blur tool

---

### Requirement: Step Number Tool

The system SHALL provide a step number annotation tool, activated by the keyboard shortcut **N**, that places auto-incrementing numbered circles on the canvas.

Each click with the Step Number tool SHALL place a numbered circle at the clicked position. The number SHALL auto-increment starting from 1 (tracked by the `nextStepNumber` state). The resulting annotation object SHALL have `type: "step"` with `x` and `y` for position in image coordinates, a `stepNumber` field containing the displayed number, and a `text` field containing the number as a string.

The step number circle SHALL be rendered using the `stepNumberColor` and `stepNumberSize` from annotation settings (defaults: color `#FF0000`, size `24`).

#### Scenario: Place sequential step numbers
- **WHEN** the Step Number tool is active and the user clicks at (100, 100), then clicks at (200, 200), then clicks at (300, 300)
- **THEN** three annotations with `type: "step"` SHALL be created with `stepNumber` values of 1, 2, and 3 respectively, each rendered as a numbered circle at the clicked coordinates

#### Scenario: Step counter persists across tool switches
- **WHEN** the user places step number 1, switches to the Arrow tool, switches back to the Step Number tool, and clicks again
- **THEN** the next step number SHALL be 2 (the counter MUST NOT reset on tool switch)

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **N** key and no text input is focused
- **THEN** the active tool SHALL switch to the Step Number tool

---

### Requirement: Freehand Tool

The system SHALL provide a freehand drawing tool, activated by the keyboard shortcut **F**, that records the mouse path as a polyline annotation.

While the user holds the mouse button down, the system SHALL continuously sample the cursor position and append points to the polyline. On release, the freehand annotation SHALL be committed. The resulting annotation object SHALL have `type: "freehand"` and store the path in the `points` array as a sequence of `{x, y}` objects in image coordinates. The freehand line SHALL respect the current `strokeColor`, `strokeWidth`, and `opacity`.

#### Scenario: Draw a freehand path
- **WHEN** the Freehand tool is active and the user presses, moves through multiple positions, and releases
- **THEN** an annotation with `type: "freehand"` SHALL be added with a `points` array containing all sampled positions along the mouse path in image coordinates, rendered as a continuous polyline

#### Scenario: Live preview during drawing
- **WHEN** the user is drawing with the Freehand tool (mouse button held)
- **THEN** the active-tool canvas layer SHALL render the path accumulated so far in real time

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **F** key and no text input is focused
- **THEN** the active tool SHALL switch to the Freehand tool

---

### Requirement: Highlight Tool

The system SHALL provide a highlight annotation tool, activated by the keyboard shortcut **H**, that draws a semi-transparent colored overlay over a rectangular region.

The user clicks to set one corner and drags to the opposite corner. The resulting annotation object SHALL have `type: "highlight"` with `x`, `y`, `width`, and `height` defining the region in image coordinates, and a `highlightColor` property (default `#FFFF00`). The highlight MUST always be rendered at `0.4` opacity regardless of the annotation's `opacity` setting -- the visual effect SHALL resemble a highlighter pen on paper.

#### Scenario: Highlight a region
- **WHEN** the Highlight tool is active and the user drags from (50, 300) to (400, 340)
- **THEN** an annotation with `type: "highlight"`, `x: 50`, `y: 300`, `width: 350`, `height: 40`, and `highlightColor: "#FFFF00"` SHALL be added, rendered as a semi-transparent colored overlay

#### Scenario: Highlight is always semi-transparent
- **WHEN** a highlight annotation is rendered
- **THEN** the overlay MUST be rendered at `0.4` opacity so that the underlying image content remains visible through the highlight, regardless of the annotation's `opacity` property

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **H** key and no text input is focused
- **THEN** the active tool SHALL switch to the Highlight tool

---

### Requirement: Crop Tool

The system SHALL provide a crop tool, activated by the keyboard shortcut **C**, that crops the image to a user-selected rectangular region.

The user clicks to set one corner of the crop region and drags to the opposite corner. While dragging, the system SHALL display a crop overlay indicating the selected region and dimming the area outside it. On confirmation (e.g., pressing Enter or double-clicking), the image SHALL be cropped to the selected region. The crop operation affects the base image and all existing annotations -- annotations fully outside the crop region SHALL be removed, and coordinates of remaining annotations SHALL be adjusted relative to the new image origin.

The crop operation SHALL be recorded as a single compound command in the undo/redo history. The command SHALL store: the original image dimensions, the crop rectangle, the full pre-crop annotations array, and the removed annotations. Undoing a crop SHALL restore the original image, reinstate all removed annotations, and reverse the coordinate adjustments on remaining annotations.

The resulting annotation object SHALL have `type: "crop"` with `x`, `y`, `width`, and `height` defining the crop region in image coordinates. Unlike other annotation types, a crop annotation is NOT added to the annotations array -- it is consumed by the crop operation and recorded only in the undo history.

#### Scenario: Crop the image to a selected region
- **WHEN** the Crop tool is active and the user selects a region from (100, 100) to (500, 400) and confirms the crop
- **THEN** the base image SHALL be cropped to that 400x300 region, existing annotation coordinates SHALL be adjusted by subtracting the crop origin (100, 100), and annotations entirely outside the crop region SHALL be removed

#### Scenario: Cancel crop operation
- **WHEN** the user initiates a crop selection and presses **Escape**
- **THEN** the crop operation SHALL be cancelled and the image SHALL remain unchanged

#### Scenario: Keyboard shortcut activation
- **WHEN** the user presses the **C** key and no text input is focused
- **THEN** the active tool SHALL switch to the Crop tool

---

### Requirement: Annotation Data Model

Every annotation object SHALL be a plain, serializable JavaScript object (no class instances, no methods) suitable for JSON serialization, storage, transmission to the Rust backend, and use by the MCP server.

Each annotation tool type SHALL register itself with a renderer function that the canvas engine dispatches to when drawing annotations. The rendering dispatch SHALL use the annotation's `type` field to select the appropriate renderer. This registry-based design allows adding new annotation types without modifying the core rendering loop.

Each annotation object MUST contain the following common fields:

| Field | Type | Description |
|---|---|---|
| `id` | string (UUID v4) | Unique identifier, generated by the frontend via `crypto.randomUUID()` at annotation creation time |
| `type` | string enum | One of: `arrow`, `rect`, `ellipse`, `text`, `blur`, `step`, `freehand`, `highlight` |
| `x` | number | Top-left x in image coordinates |
| `y` | number | Top-left y in image coordinates |
| `width` | number | Bounding box width (for `rect`, `ellipse`, `blur`, `highlight`) |
| `height` | number | Bounding box height (for `rect`, `ellipse`, `blur`, `highlight`) |
| `points` | array of `{x, y}` | Geometry points (for `arrow`: 2 points; for `freehand`: N points) |
| `strokeColor` | string | Stroke/outline color (CSS color) |
| `fillColor` | string | Fill color (CSS color or `"transparent"`) |
| `strokeWidth` | number | Stroke width in image pixels |
| `opacity` | number | Opacity from 0.0 to 1.0 |
| `createdAt` | string | ISO 8601 timestamp |
| `locked` | boolean | Whether the annotation is locked from editing |

In addition, the following type-specific fields MUST be present when applicable:

| Field | Applicable Types | Type | Description |
|---|---|---|---|
| `text` | `text`, `step` | string | Text content |
| `fontSize` | `text` | number | Font size in image pixels |
| `fontFamily` | `text` | string | CSS font family |
| `stepNumber` | `step` | number | Displayed step number |
| `blurRadius` | `blur` | number | Blur/pixelate intensity |
| `highlightColor` | `highlight` | string | Highlight overlay color |

All geometry values (x, y, width, height, points) MUST be in image coordinates, not screen coordinates. The canvas engine is responsible for applying the current zoom/pan transform when rendering.

> **Note**: The `crop` tool operation (defined above) does NOT produce a persisted annotation in this data model. Crop geometry is recorded only in the undo/redo history. The `type` enum therefore does not include `crop`.

#### Scenario: UUID generation
- **WHEN** the frontend creates a new annotation object
- **THEN** it MUST assign a UUID v4 string to the `id` field using `crypto.randomUUID()` (or equivalent)
- **THEN** the `id` MUST be unique across all annotations in the current session

#### Scenario: Deserialization validation
- **WHEN** the Rust backend or MCP server receives an annotation array via IPC or JSON-RPC
- **THEN** it MUST validate that each annotation has a non-empty `id`, a recognized `type`, and numeric geometry values
- **THEN** annotations failing validation MUST be rejected with a descriptive error (not silently dropped)

#### Scenario: Serialize and deserialize an annotation
- **WHEN** an annotation object is passed to `JSON.stringify()` and the result is passed to `JSON.parse()`
- **THEN** the resulting object MUST be identical in structure and values to the original (no data loss, no class methods required)

#### Scenario: Annotation sent to Rust backend
- **WHEN** the frontend invokes `save_image` or `copy_to_clipboard` with an annotations array
- **THEN** each annotation in the array MUST conform to this data model so the Rust backend can deserialize it via `serde_json`

#### Scenario: Coordinates are in image space
- **WHEN** the user draws an annotation on a zoomed or panned canvas
- **THEN** the stored `x`, `y`, `width`, `height`, and `points` values MUST be in the original image coordinate system, independent of the current zoom level or pan offset

---

### Requirement: Annotation Lock and Unlock

The system SHALL allow the user to lock an annotation, preventing it from being moved, resized, or deleted. The `locked` field on the annotation object controls this behavior. Locking SHALL be toggled via right-click context menu ("Lock" / "Unlock") on a selected annotation.

#### Scenario: Lock a selected annotation
- **WHEN** the user right-clicks a selected annotation and chooses "Lock"
- **THEN** the annotation's `locked` field SHALL be set to `true`
- **THEN** the selection handles SHALL change to indicate the locked state (e.g., dashed handles or a lock icon)

#### Scenario: Locked annotation cannot be moved or resized
- **WHEN** the user attempts to drag or resize a locked annotation
- **THEN** the move or resize operation SHALL be ignored and the annotation SHALL remain in place

#### Scenario: Locked annotation cannot be deleted via Delete key
- **WHEN** the user selects a locked annotation and presses `Delete`
- **THEN** the annotation SHALL NOT be removed from the annotations array

#### Scenario: Unlock a locked annotation
- **WHEN** the user right-clicks a locked annotation and chooses "Unlock"
- **THEN** the annotation's `locked` field SHALL be set to `false`
- **THEN** normal move, resize, and delete operations SHALL be re-enabled

---

### Requirement: Annotation Z-Ordering

Annotations SHALL be rendered in the order they appear in the annotations array (index 0 is bottommost, last index is topmost). The system SHALL provide z-ordering operations accessible via right-click context menu on a selected annotation:

- **Bring to Front** -- move to last position in the array
- **Send to Back** -- move to first position in the array
- **Bring Forward** -- move one position toward the end
- **Send Backward** -- move one position toward the start

Z-order changes SHALL be recorded as commands in the undo/redo history.

#### Scenario: Bring annotation to front
- **WHEN** the user right-clicks a selected annotation and chooses "Bring to Front"
- **THEN** the annotation SHALL be moved to the last position in the annotations array
- **THEN** the annotation layer SHALL be re-rendered with the new ordering

#### Scenario: Send annotation to back
- **WHEN** the user right-clicks a selected annotation and chooses "Send to Back"
- **THEN** the annotation SHALL be moved to the first position in the annotations array

#### Scenario: Z-order change is undoable
- **WHEN** the user changes an annotation's z-order and then presses `Ctrl+Z`
- **THEN** the annotation SHALL return to its previous position in the array
