Implemented all 8 remaining annotation tools (arrow, ellipse, text, blur, step, freehand, highlight, crop).

## Frontend (JS)
- engine.js: Added drawShape cases for all types. Arrow uses atan2 arrowhead. Blur uses OffscreenCanvas pixelation. Step renders numbered circle. Freehand renders polyline. Highlight forced 0.4 opacity. Added renderCropOverlay() with destination-out composite for the dimming effect. Added #getShapeBBox() for per-type selection indicators.
- history.js: Added record(cmd) method for pre-executed commands (used by crop).
- commands.js: Added CropCommand with async onUndo/onExecute callbacks. Uses Promise.resolve().then() to fire image-reload side effects asynchronously without blocking History's synchronous undo/redo interface.
- app.js: Complete rewrite of mouse handlers. Dispatch by activeTool. DRAG_DRAW_TOOLS set covers rect/arrow/ellipse/blur/highlight. Step places on mousedown. Text shows floating textarea, commits on blur/Escape. Crop uses isCropDragging state, Enter to confirm, Escape to cancel. applyCrop() calls backend and records CropCommand in history. buildPreviewShape() and buildCommittedShape() helpers factor out per-tool annotation construction.

## Backend (Rust)
- files.rs: Added Point struct. Updated Annotation with stepNumber, blurRadius, highlightColor, fontFamily fields, changed points to Vec<Point>. Added composite_annotation dispatcher. Added composite_arrow (shaft + two-line arrowhead, draw_thick_line helper), composite_ellipse (draw_filled_ellipse_mut + draw_hollow_ellipse_mut), composite_freehand (polyline via draw_thick_line), composite_highlight (blend_pixel at 0.4 opacity), composite_blur (block-average pixelation), composite_step (draw_filled_circle_mut, text skipped — needs font embedding). 4 new tests covering highlight, blur, arrow, and new serde fields.

## Known limitations
- Step circle text (the number) not composited in exported PNG — needs font embedding (ab_glyph + bundled font file)
- Text annotation not composited in exported PNG — same reason
- Ellipse stroke/fill uses imageproc's direct pixel write (no alpha compositing) — acceptable for opaque colors
- Thick line approximation uses parallel offset lines — looks correct for reasonable stroke widths
