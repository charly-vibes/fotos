# Canvas Engine

Capability spec for the HTML5 Canvas 2D rendering engine used by Fotos (`io.github.charly.fotos`).

## Purpose

The canvas engine manages triple-layer rendering, zoom/pan navigation, coordinate transforms, image loading, an efficient render loop, and composite export. It is implemented in `src-ui/js/canvas/engine.js` as the `CanvasEngine` class.

## Requirements

### Requirement: Triple-Layer Rendering

The canvas engine SHALL maintain three stacked `<canvas>` elements for rendering separation and performance:

- **Layer 0** (`canvas-base`): The screenshot image. Redrawn only on load, zoom, or pan.
- **Layer 1** (`canvas-annotations`): Committed annotations. Redrawn only when the annotations array changes.
- **Layer 2** (`canvas-active`): Active tool preview. Redrawn on every mouse move during tool interaction.

Each layer MUST use its own `CanvasRenderingContext2D` obtained via `getContext('2d')`. All three canvases MUST be sized identically and stacked within the `#canvas-container` element so they overlay precisely.

#### Scenario: Engine initializes three layers
- **WHEN** a `CanvasEngine` is constructed with three canvas elements (`baseCanvas`, `annoCanvas`, `activeCanvas`)
- **THEN** each canvas SHALL have a dedicated 2D rendering context stored as `#baseCtx`, `#annoCtx`, and `#activeCtx`
- **THEN** all three canvases SHALL be sized to match the container dimensions

#### Scenario: Layers remain visually aligned
- **WHEN** the container is resized (e.g., via a `ResizeObserver` callback)
- **THEN** all three canvases MUST be resized to the same width and height
- **THEN** the current viewport (zoom and pan) SHALL be preserved after resize

#### Scenario: Layer isolation prevents unnecessary redraws
- **WHEN** the user moves the mouse during an active tool interaction
- **THEN** only the active layer (`canvas-active`) SHALL be cleared and redrawn
- **THEN** the base layer and annotations layer MUST NOT be redrawn

---

### Requirement: Zoom and Pan

The canvas engine SHALL support zoom in, zoom out, and panning so the user can navigate large or detailed screenshots.

- Zoom in/out SHALL be triggered by the `+` and `-` keys.
- Pan SHALL be triggered by holding `Space` and dragging the mouse.
- Reset zoom SHALL be triggered by `Ctrl+0`, restoring zoom to `1.0` and pan offsets to `(0, 0)`.
- The current zoom level SHALL be stored in `state.zoom` and pan offsets in `state.panX` / `state.panY`.

#### Scenario: Zoom in
- **WHEN** the user presses the `+` key
- **THEN** the zoom level SHALL increase
- **THEN** all three layers MUST be redrawn with the updated transform

#### Scenario: Zoom out
- **WHEN** the user presses the `-` key
- **THEN** the zoom level SHALL decrease
- **THEN** all three layers MUST be redrawn with the updated transform

#### Scenario: Pan with Space and drag
- **WHEN** the user holds the `Space` key and drags the mouse
- **THEN** the canvas viewport SHALL translate by the drag delta, updating `panX` and `panY`
- **THEN** all three layers MUST be redrawn with the updated transform

#### Scenario: Reset zoom
- **WHEN** the user presses `Ctrl+0`
- **THEN** the zoom level SHALL be reset to `1.0`
- **THEN** the pan offsets SHALL be reset to `(0, 0)`
- **THEN** all three layers MUST be redrawn at the default transform

---

### Requirement: Coordinate Transforms

The canvas engine SHALL provide a `screenToImage(screenX, screenY)` method that converts screen coordinates (from mouse events) to image coordinates by applying the inverse of the current zoom and pan transform.

All annotation geometry MUST be stored in image coordinates, not screen coordinates, so that annotations remain correctly positioned regardless of the current zoom/pan state.

#### Scenario: Convert screen coordinates to image coordinates
- **WHEN** the user clicks at screen position `(screenX, screenY)` while the canvas has zoom level `z`, pan offset `(panX, panY)`
- **THEN** `screenToImage(screenX, screenY)` SHALL return image coordinates computed by applying the inverse transform: `imageX = (screenX - panX) / z`, `imageY = (screenY - panY) / z`

#### Scenario: Annotations use image coordinates
- **WHEN** a tool creates an annotation from a mouse interaction
- **THEN** the tool MUST convert all screen coordinates to image coordinates via `screenToImage` before storing the annotation geometry
- **THEN** the stored annotation coordinates SHALL be independent of the current zoom and pan state

---

### Requirement: Image Loading

The canvas engine SHALL provide a `loadImage(imageData)` method that accepts screenshot image data (as an `ArrayBuffer` or equivalent), creates an `ImageBitmap`, stores it internally as `#image`, and triggers a base layer redraw.

#### Scenario: Load screenshot image
- **WHEN** `loadImage(imageData)` is called with valid image data
- **THEN** the engine SHALL create an `ImageBitmap` from the provided data
- **THEN** the `ImageBitmap` SHALL be stored as `#image`
- **THEN** the base layer SHALL be redrawn via `renderBase()`

#### Scenario: Replace existing image
- **WHEN** `loadImage(imageData)` is called while an image is already loaded
- **THEN** the previous `#image` SHALL be replaced with the new `ImageBitmap`
- **THEN** the base layer SHALL be redrawn to display the new image

---

### Requirement: Render Loop

The canvas engine SHALL implement an efficient render loop with separate redraw methods for each layer. Each layer MUST only be redrawn when its specific trigger condition is met:

- `renderBase()`: Clears the base canvas, applies the current zoom/pan transform, and draws the `#image`. Triggered on image load, zoom change, or pan change.
- `renderAnnotations(annotations)`: Clears the annotations canvas, applies the current zoom/pan transform, iterates over all committed annotations, and draws each one. Triggered when the annotations array changes (add, remove, modify, undo, redo).
- `renderActive(previewShape)`: Clears the active canvas, applies the current zoom/pan transform, and draws a single preview shape. Triggered on every `mousemove` during active tool interaction.

#### Scenario: Redraw base layer on image load
- **WHEN** a new screenshot image is loaded via `loadImage`
- **THEN** `renderBase()` SHALL clear the base canvas, apply the transform, and call `drawImage` with the stored `#image`

#### Scenario: Redraw base layer on zoom or pan
- **WHEN** the zoom level or pan offset changes
- **THEN** `renderBase()` SHALL be called to redraw the screenshot at the new transform
- **THEN** `renderAnnotations()` SHALL also be called so annotations remain aligned

#### Scenario: Redraw annotations on change
- **WHEN** an annotation is added, removed, modified, or restored via undo/redo
- **THEN** `renderAnnotations(annotations)` SHALL clear the annotations canvas and redraw all committed annotations
- **THEN** the base layer MUST NOT be redrawn

#### Scenario: Redraw active layer on mouse move
- **WHEN** the user moves the mouse while an annotation tool is active and a shape is being drawn
- **THEN** `renderActive(previewShape)` SHALL clear the active canvas and draw the current tool preview shape
- **THEN** neither the base layer nor the annotations layer MUST be redrawn

---

### Requirement: Export Composite

The canvas engine SHALL provide an `exportComposite(annotations, format)` method that composites the screenshot and all annotations into a single image and returns it as a `Blob`.

- The method SHALL create an offscreen canvas sized to the original image dimensions (not the viewport dimensions).
- The base screenshot image SHALL be drawn first, at original scale with no zoom/pan transform.
- All committed annotations SHALL be drawn on top, at original scale with no zoom/pan transform.
- The composited result SHALL be returned as a `Blob` in the requested format (defaulting to `'png'`).

#### Scenario: Export composited image as PNG
- **WHEN** `exportComposite(annotations, 'png')` is called
- **THEN** the engine SHALL create an offscreen canvas at the original image width and height
- **THEN** the engine SHALL draw the base `#image` at position `(0, 0)` with no transform
- **THEN** the engine SHALL iterate over all annotations and draw each at original image coordinates
- **THEN** the engine SHALL return the canvas contents as a `Blob` with MIME type `image/png`

#### Scenario: Export preserves original resolution
- **WHEN** the user has zoomed in to 200% and panned the viewport before exporting
- **THEN** the exported image MUST be at the original screenshot dimensions, not the zoomed viewport dimensions
- **THEN** annotations MUST be positioned using their stored image coordinates, not screen coordinates

#### Scenario: Export with no annotations
- **WHEN** `exportComposite([], 'png')` is called with an empty annotations array
- **THEN** the exported `Blob` SHALL contain only the original screenshot image at full resolution
