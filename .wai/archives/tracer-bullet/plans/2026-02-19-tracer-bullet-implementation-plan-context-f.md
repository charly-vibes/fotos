# Tracer Bullet Implementation Plan

## Context

Fotos is a Tauri 2 screenshot capture + annotation + AI analysis app. The codebase is well-scaffolded (~15-20% done) with 9 OpenSpec capability specs, but most code returns "Not yet implemented". A **tracer bullet** is the narrowest useful end-to-end slice through the entire system using production-quality code.

**The slice**: Capture screenshot -> Display on canvas -> Draw rectangle -> Save composited PNG

This proves every integration boundary: Tauri boot, Rust capture, Tauri IPC, JS canvas rendering, coordinate transforms, annotation model, Rust compositing, file I/O.

## What's Excluded

AI (OCR/PII/LLM), MCP server, IPC bridge, settings modal, export dialog, color picker, selection/resize, 9 of 10 annotation tools, clipboard ops, Wayland portal window enumeration, theme switching.

## What's Included (Minimally)

Empty canvas state, zoom (+/-/Ctrl+0/mouse wheel), pan (Space+drag), status bar, toolbar (capture + rect + save), HiDPI support, undo/redo for rectangle tool.

## Existing Scaffolding

The following files are **already fully implemented** and must be leveraged, not duplicated:

| File | What it provides |
|------|-----------------|
| `src-ui/js/state.js` | `StateStore` class with pub/sub pattern. Exported singleton `store` with keys: `activeTool`, `strokeColor`, `fillColor`, `strokeWidth`, `opacity`, `zoom`, `panX`, `panY`, `currentImageId`, `annotations`, `nextStepNumber`. |
| `src-ui/js/canvas/history.js` | `History` class with command pattern. Already handles: 100-command limit with FIFO eviction, redo stack clearing on new action, `canUndo`/`canRedo` getters. |
| `src-ui/js/ui/toolbar.js` | `initToolbar(store)` — wires tool buttons (`data-tool` attributes) and keyboard shortcuts to `store.set('activeTool', ...)`. Already handles active tool highlighting. |
| `src-ui/js/app.js` | Partial init: creates `CanvasEngine`, calls `initToolbar(store)`. TODOs for keyboard shortcuts, Tauri events, settings. |
| `src-tauri/src/lib.rs` | Tauri builder with all plugins registered and all commands in `invoke_handler`. No `.setup()` hook or `.manage()` state yet — these need adding. |
| `src-tauri/src/capture/mod.rs` | Data model: `CaptureMode`, `CaptureMetadata`, `CaptureResult` structs. `CaptureResult.image` is `image::DynamicImage`. |
| `src-tauri/src/commands/capture.rs` | Stubbed commands. `ScreenshotResponse { id, width, height, data_url }` struct already defined. |
| `src-tauri/src/commands/files.rs` | Stubbed commands. `Annotation` struct defined with `#[serde(rename = "type")]` on `annotation_type` field; other fields use snake_case (needs `rename_all`). |
| `src-ui/index.html` | Complete HTML shell with triple-layer canvas, toolbar groups, status bar spans (`#status-dimensions`, `#status-zoom`, `#status-tool`, `#status-message`). |
| `src-ui/css/main.css` | Full layout grid, toolbar styling, canvas stacking, AI panel, status bar. |

## Spec Deviations (Intentional for Tracer Bullet)

| Deviation | Spec requirement | Tracer bullet approach | Why |
|-----------|-----------------|----------------------|-----|
| D1 | `CaptureBackend` trait with 3 methods | Direct functions, no trait | Trait adds indirection for one backend; will add when second backend needed |
| D2 | Backend emits `screenshot-ready` event | Command returns `ScreenshotResponse` directly | Simpler IPC for tracer; event pattern needed later for async portal flows |
| D3 | `loadImage(ArrayBuffer)` signature | `loadImage(dataUrl)` accepting data URL string | Base64 data URL is what the command returns; optimize to ArrayBuffer later |
| D4 | `modal(true)` not in spec | Portal call includes `.modal(true)` | Prevents portal dialog from going behind app window; harmless addition |

---

## Phase 0: App Boot

**Goal**: App compiles, launches, shows window with existing UI shell.

### 0.1 Create Tauri 2 capability file
**Create** `src-tauri/capabilities/default.json` — Tauri 2 requires explicit plugin permissions. Grant `core:default`, `dialog:default`, `fs:default`, `clipboard-manager:default`, `global-shortcut:default`, `os:default`, `store:default`, `core:event:default`, `shell:allow-open`.

### 0.2 Add setup hook and managed state
**Modify** `src-tauri/src/lib.rs` — The file currently has the Tauri builder with plugins and command registrations but no `.setup()` or `.manage()`. Add:
- `.setup(|app| { app.get_webview_window("main").unwrap().show().unwrap(); Ok(()) })` before `.run()` — window starts hidden per tauri.conf.json
- `tracing_subscriber::fmt::init()` at the start of `run()` for logging

> **Note**: `.manage()` for ImageStore is added in Phase 1.3.

### 0.3 Verify
`just dev` — window opens, toolbar/canvas/status bar visible, buttons do nothing yet.

---

## Phase 1: Image Store + Backend Routing

**Goal**: In-memory image registry + platform-aware capture routing.

### 1.1 Create image store
**Create** `src-tauri/src/capture/store.rs` — `ImageStore` wrapping `Mutex<HashMap<Uuid, CaptureResult>>` with `insert(&self, result: CaptureResult) -> Uuid`, `get(&self, id: &Uuid) -> Option<...>`, `remove(&self, id: &Uuid)`.

### 1.2 Wrap image in Arc and add routing
**Modify** `src-tauri/src/capture/mod.rs`:
- Change `CaptureResult.image` from `image::DynamicImage` to `Arc<image::DynamicImage>` — needed for shared ownership between ImageStore and compositing. This is an implementation detail transparent to the spec's `DynamicImage` requirement.
- Add `pub mod store;`
- Add async routing function `pub async fn capture(mode: CaptureMode) -> Result<CaptureResult>` that calls `detect_platform()` and dispatches:
  - Any `LinuxWayland*` variant -> `portal::capture_via_portal(mode)`
  - `LinuxX11` | `Windows` -> `xcap_backend::capture_xcap(mode)`

### 1.3 Register managed state
**Modify** `src-tauri/src/lib.rs` — Add `.manage(capture::store::ImageStore::new())` to the builder chain (before `.invoke_handler()`).

### 1.4 Fix platform detection
**Modify** `src-tauri/src/capture/detect.rs` — The current code falls through to `LinuxX11` whenever `XDG_SESSION_TYPE` is not `"wayland"`, without checking `WAYLAND_DISPLAY`. Per spec (Scenario: "Fallback to X11 when session type unknown"), add: when `XDG_SESSION_TYPE` is empty/unset, check `WAYLAND_DISPLAY`. If set, resolve to `LinuxWaylandOther`. If unset, resolve to `LinuxX11`.

### 1.5 Verify
`just check` passes (compile-only — no runtime behavior to test yet).

### 1.6 Unit test: ImageStore
**Create** `src-tauri/src/capture/store.rs` (in `#[cfg(test)] mod tests`) — Test `insert` returns UUID, `get` retrieves by UUID, `remove` clears entry, `get` after remove returns `None`.

---

## Phase 2: Capture Backends

**Goal**: Working portal (Wayland) and xcap (X11) capture.

### 2.1 Implement portal backend
**Modify** `src-tauri/src/capture/portal.rs` — Change signature to `pub async fn capture_via_portal(mode: CaptureMode) -> Result<CaptureResult>`. Implementation:
1. Call `ashpd::desktop::screenshot::Screenshot::request().interactive(true).modal(true).send().await`
2. Get URI from response
3. Validate URI scheme is `file://` — if not (e.g. `fd://`), return error: `"Unsupported portal URI scheme: {scheme}"`
4. Parse `file://` path, load image via `image::open()`
5. Assign `Uuid::new_v4()`, populate `CaptureMetadata` with `Utc::now()` timestamp, mode, dimensions from image
6. Clean up temp file (best-effort `std::fs::remove_file`)
7. Wrap image in `Arc::new()`
8. If portal D-Bus call fails, return descriptive error: `"Portal unavailable: {err}"`

### 2.2 Implement xcap fullscreen backend
**Modify** `src-tauri/src/capture/xcap_backend.rs` — Change signature to `pub fn capture_xcap(mode: CaptureMode) -> Result<CaptureResult>`. For `Fullscreen` mode:
1. Call `xcap::Monitor::all()?` to enumerate all monitors
2. Capture each monitor's image
3. **Composite all monitors** into a single image by computing the bounding box across all monitor positions/dimensions, creating a canvas of that size, and drawing each monitor's image at its `(x, y)` offset — per spec, fullscreen MUST include all monitors
4. Wrap in `DynamicImage::ImageRgba8`, assign UUID, populate metadata with composited dimensions
5. For single-monitor systems this naturally captures just that monitor

### 2.3 Verify
`just check` passes (compile-only).

### 2.4 Error path verification
Add `#[cfg(test)]` tests in `portal.rs` and `xcap_backend.rs`:
- Portal: test that a non-`file://` URI returns the expected error message
- xcap: test metadata population (mock a small image, verify UUID is set, dimensions match)

---

## Phase 3: Capture Command

**Goal**: Frontend can trigger capture and receive image as base64 data URL.

### 3.1 Implement take_screenshot command
**Modify** `src-tauri/src/commands/capture.rs` — Add `State<'_, ImageStore>` parameter to `take_screenshot`. Implementation:
1. Parse `mode` string to `CaptureMode` (for tracer bullet, support `"fullscreen"` and `"monitor"` with the `monitor: Option<u32>` param)
2. Call `capture::capture(mode).await`
3. On error, return descriptive `Err(format!("Capture failed: {err}"))`
4. Encode image as base64 PNG data URL: `format!("data:image/png;base64,{}", base64::encode(png_bytes))`
5. Store `CaptureResult` in `ImageStore` via `store.insert(result)`
6. Return `ScreenshotResponse { id: uuid.to_string(), width, height, data_url }`

> **Spec deviation D2**: The spec requires emitting a `screenshot-ready` event. The tracer bullet returns data directly through the command for simplicity. The event pattern will be needed when portal captures run asynchronously in the background.

### 3.2 Verify
`just dev` — open webview devtools (F12), run:
```js
await window.__TAURI__.core.invoke('take_screenshot', { mode: 'fullscreen', monitor: null })
```
GNOME screenshot UI appears, result has valid `data_url`, `id`, `width`, `height`.

> **Note**: `window.__TAURI__` is available only in Tauri dev mode. This is a manual smoke test, not an automated test.

### 3.3 Error path verification
Test in devtools: `invoke('take_screenshot', { mode: 'invalid', monitor: null })` — should return descriptive error, not crash.

---

## Phase 4: Canvas Engine

**Goal**: Triple-layer canvas with image loading, rendering, coordinate transforms, HiDPI, zoom, pan, empty state.

### 4.1 Implement CanvasEngine
**Modify** `src-ui/js/canvas/engine.js` — Full implementation. The class already has private field declarations and stub methods. Implement:

**Constructor + resize:**
- Store references to all 3 canvases + contexts + container (`#canvas-container`)
- `ResizeObserver` on container → calls `#resize()`
- `matchMedia('(resolution: Xdppx)')` listener for DPR changes → calls `#resize()`
- `#resize()`: set each canvas backing store to `Math.floor(containerWidth * DPR)` x `Math.floor(containerHeight * DPR)`, CSS size to container dimensions. Call `#renderAll()`.

**Image loading:**
- `loadImage(dataUrl)`: create `new Image()`, set `src = dataUrl`, on load → `createImageBitmap(img)` → store as `#image` → call `#renderAll()`

**Coordinate transform — pan is in CSS pixels:**
- `screenToImage(sx, sy)`: `{ x: (sx - panX) / zoom, y: (sy - panY) / zoom }` — matches spec formula. DPR is transparent because the context is pre-scaled by `ctx.scale(dpr, dpr)`, so all drawing and mouse coordinates are in CSS-pixel space.
- `#applyTransform(ctx)`: `ctx.setTransform(dpr, 0, 0, dpr, 0, 0)` (reset to DPR scale), then `ctx.translate(panX, panY)`, then `ctx.scale(zoom, zoom)`

**Rendering:**
- `renderBase()`: clear, if no `#image` draw centered placeholder text ("Capture or open a screenshot to begin" in `--text-secondary` color), else apply transform + `drawImage(#image, 0, 0)`
- `renderAnnotations(anns)`: clear, apply transform, draw each via `#drawAnnotation(ctx, ann)`
- `renderActive(preview)`: clear, apply transform, draw single shape if preview is not null
- `#drawAnnotation(ctx, ann)`: switch on `ann.type`:
  - `rect`: set `ctx.strokeStyle = ann.strokeColor`, `ctx.lineWidth = ann.strokeWidth`, `ctx.globalAlpha = ann.opacity`. Call `ctx.strokeRect(ann.x, ann.y, ann.width, ann.height)`. If `ann.fillColor !== 'transparent'`, set `ctx.fillStyle = ann.fillColor` and `ctx.fillRect(...)`.
- `#renderAll()`: calls `renderBase()`, then `renderAnnotations(currentAnnotations)` if any

**Zoom — clamped 0.1 to 10.0:**
- `setZoom(z)`: clamp `z` to `[0.1, 10.0]`, store, call `#renderAll()`
- `getZoom()`: return current zoom
- `zoomBy(factor)`: `setZoom(zoom * factor)` — used by mouse wheel

**Pan:**
- `setPan(x, y)`: store CSS-pixel pan offsets, call `#renderAll()`
- `getPan()`: return `{ x: panX, y: panY }`

**Getters:**
- `hasImage`, `imageWidth`, `imageHeight`, `dpr`

### 4.2 Verify
`just dev` — canvas shows centered "Capture or open a screenshot to begin". Resize window, text stays centered. DevTools: `engine.setZoom(0.05)` clamps to 0.1, `engine.setZoom(20)` clamps to 10.0.

---

## Phase 5: Frontend Capture Flow + Navigation

**Goal**: Capture button triggers screenshot, image displays on canvas, toolbar enables, zoom + pan work.

### 5.1 Wire capture, zoom, pan, and status
**Modify** `src-ui/js/app.js` — The file currently imports `store`, `CanvasEngine`, `initToolbar`. Add imports:
- `{ invoke }` from `@tauri-apps/api/core` (Tauri 2 JS API for invoking commands)
- `{ save }` from `@tauri-apps/plugin-dialog` (for Phase 8)
- `{ History }` from `./canvas/history.js`

In `init()`, after existing `initToolbar(store)` call, add:

**Capture wiring:**
- Query `[data-action="capture-fullscreen"]` button. On click: call `invoke('take_screenshot', { mode: 'fullscreen', monitor: null })`, store result's `id` in `store.set('currentImageId', result.id)`, call `engine.loadImage(result.data_url)`, update status bar dimensions (`result.width x result.height`), enable annotation tool buttons (remove `disabled` attribute).
- On init: set `disabled` on all `#annotation-tools button` elements until an image is loaded.

**Zoom keyboard handlers:**
- `+` / `=` key: `engine.setZoom(engine.getZoom() * 1.25)`, update `#status-zoom`
- `-` key: `engine.setZoom(engine.getZoom() / 1.25)`, update `#status-zoom`
- `Ctrl+0`: `engine.setZoom(1.0); engine.setPan(0, 0)`, update `#status-zoom`

**Mouse wheel zoom (centered on cursor):**
- `wheel` event on canvas container: `e.preventDefault()`, compute zoom factor from `e.deltaY` (up = zoom in by 1.1, down = zoom out by 1/1.1). Adjust pan so zoom centers on cursor position: `newPan = cursor - (cursor - oldPan) * (newZoom / oldZoom)`. Call `engine.setZoom()` and `engine.setPan()`.
- Handle `ctrlKey` wheel events for trackpad pinch-to-zoom.

**Space+drag pan:**
- `keydown` Space: set `isPanning = true`, change cursor to `grab` on canvas container
- `keyup` Space: set `isPanning = false`, restore cursor
- `mousedown` while panning: record start position, change cursor to `grabbing`
- `mousemove` while panning and dragging: compute delta in CSS pixels, call `engine.setPan(pan.x + dx, pan.y + dy)`
- `mouseup`: stop drag

**Status bar update helper:**
- `updateStatus({ dimensions, zoom, tool, message })`: set `textContent` on `#status-dimensions`, `#status-zoom` (format as `Math.round(zoom * 100) + '%'`), `#status-tool`, `#status-message`. Messages auto-clear after 4 seconds via `setTimeout`.

**Tool name sync:**
- `store.on('activeTool', tool => updateStatus({ tool: toolDisplayName(tool) }))` — map `'rect'` to `'Rectangle'`, etc.

### 5.2 Add disabled button style
**Modify** `src-ui/css/main.css` — Add `#toolbar button:disabled { opacity: 0.3; cursor: not-allowed; pointer-events: none; }`.

### 5.3 Verify
`just dev`:
1. App opens, annotation tools disabled, placeholder text visible
2. Click capture fullscreen — GNOME overlay appears, select area, image shows on canvas
3. Status bar: "1920 x 1080" "100%", annotation buttons enabled
4. Press `+` — zoom increases, status shows "125%"
5. Press `-` — zoom decreases
6. `Ctrl+0` — resets to 100%
7. Mouse wheel over canvas — zooms centered on cursor
8. Hold Space + drag — canvas pans, cursor shows grab hand

---

## Phase 6: Rectangle Annotation Tool

**Goal**: Click-drag draws rectangle with preview, commits via command pattern, undo/redo works.

### 6.1 Create AddAnnotationCommand
**Create** `src-ui/js/canvas/commands.js`:
```js
export class AddAnnotationCommand {
  #annotation;
  constructor(annotation) { this.#annotation = annotation; }
  execute(annotations) { return [...annotations, this.#annotation]; }
  undo(annotations) { return annotations.filter(a => a.id !== this.#annotation.id); }
}
```
This is consumed by the existing `History` class from `history.js` (which already handles the 100-command limit, FIFO eviction, and redo-stack clearing on new actions).

### 6.2 Wire rectangle tool mouse events
**Modify** `src-ui/js/app.js` — Import `AddAnnotationCommand` from `./canvas/commands.js`. Create `const history = new History()`.

On `mousedown` (active canvas, `canvas-active`):
- If `store.get('activeTool') !== 'rect'` or `!engine.hasImage`, return
- Record start point: `startImg = engine.screenToImage(e.offsetX, e.offsetY)`
- Set `isDrawing = true`

On `mousemove`:
- If not drawing, return
- Compute current image point: `curImg = engine.screenToImage(e.offsetX, e.offsetY)`
- Build preview rect: `normalizeRect(startImg, curImg)` — ensure positive width/height
- Call `engine.renderActive(previewRect)` with style from `StateStore`:
  ```js
  { type: 'rect', x, y, width, height,
    strokeColor: store.get('strokeColor'),
    fillColor: store.get('fillColor'),
    strokeWidth: store.get('strokeWidth'),
    opacity: store.get('opacity') }
  ```

On `mouseup`:
- If not drawing, return. Set `isDrawing = false`.
- Normalize rect. Skip if width < 2 or height < 2 (prevents accidental clicks)
- Create annotation object with **all required data model fields**:
  ```js
  { id: crypto.randomUUID(),
    type: 'rect',
    x, y, width, height,
    points: [],
    strokeColor: store.get('strokeColor'),
    fillColor: store.get('fillColor'),
    strokeWidth: store.get('strokeWidth'),
    opacity: store.get('opacity'),
    createdAt: new Date().toISOString(),
    locked: false }
  ```
- Execute through history: `const newAnns = history.execute(new AddAnnotationCommand(ann), store.get('annotations'))`
- Update store: `store.set('annotations', newAnns)`
- Re-render: `engine.renderAnnotations(newAnns)`, `engine.renderActive(null)` (clear preview)

**`normalizeRect(p1, p2)` helper** (inline or in a utils file):
```js
function normalizeRect(p1, p2) {
  return { x: Math.min(p1.x, p2.x), y: Math.min(p1.y, p2.y),
           width: Math.abs(p2.x - p1.x), height: Math.abs(p2.y - p1.y) };
}
```

### 6.3 Wire undo/redo
**Modify** `src-ui/js/app.js`:
- Query `[data-action="undo"]` and `[data-action="redo"]` buttons
- Undo button click + `Ctrl+Z`: `const anns = history.undo(store.get('annotations')); store.set('annotations', anns); engine.renderAnnotations(anns);`
- Redo button click + `Ctrl+Shift+Z`: `const anns = history.redo(store.get('annotations')); store.set('annotations', anns); engine.renderAnnotations(anns);`
- Keyboard handler: check `e.ctrlKey && e.key === 'z'` (undo) and `e.ctrlKey && e.shiftKey && e.key === 'Z'` (redo)

### 6.4 Sync active tool to canvas cursor
- `store.on('activeTool', tool => { activeCanvas.style.cursor = tool === 'rect' ? 'crosshair' : 'default'; })`
- Update `#status-tool` via the status helper from Phase 5.1.

### 6.5 Verify
`just dev`:
1. Capture screenshot
2. Press R — crosshair cursor, status bar shows "Rectangle"
3. Click-drag: red rectangle preview appears during drag
4. Release: rectangle commits (solid outline on image)
5. Draw second rectangle
6. `Ctrl+Z` — second rect disappears. `Ctrl+Shift+Z` — reappears
7. Draw >100 rects — first ones become non-undoable (FIFO eviction, handled by existing History class)
8. Zoom in/out — rectangles stay correctly positioned in image space

---

## Phase 7: Backend Compositing + Save

**Goal**: Rust composites annotations onto image and writes PNG. Expose both `composite_image` and `save_image` commands.

### 7.1 Fix Annotation serde
**Modify** `src-tauri/src/commands/files.rs`:
- Add `#[serde(rename_all = "camelCase")]` to `Annotation` struct. This renames `stroke_color` -> `strokeColor`, `fill_color` -> `fillColor`, etc. The existing `#[serde(rename = "type")]` on `annotation_type` takes precedence over `rename_all`, so the `type` field still deserializes correctly.
- Add missing fields per annotation data model: `points: Option<Vec<Point>>` (where `Point { x: f64, y: f64 }`), `created_at: Option<String>`, `locked: Option<bool>`, `step_number: Option<u32>`, `blur_radius: Option<f64>`, `highlight_color: Option<String>`, `font_family: Option<String>`.

### 7.2 Implement compositing
**Create** `src-tauri/src/commands/composite.rs`:
- `pub fn composite(base: &DynamicImage, annotations: &[Annotation]) -> RgbaImage`
- Copy base to `rgba8` buffer
- For each annotation, dispatch by `annotation_type`:
  - `"rect"`: Parse `stroke_color` CSS hex to `Rgba<u8>`. Use `imageproc::drawing::draw_hollow_rect_mut` for stroke. For `stroke_width > 1`, draw multiple offset rects (inset by 1px each iteration) to approximate thick strokes. If `fill_color` is present and not `"transparent"`, parse color and use `imageproc::drawing::draw_filled_rect_mut` for the interior (drawn before stroke so stroke overlays fill).
  - Other types: skip with a debug log (tracer bullet only handles `rect`)

> **Visual fidelity note**: Concentric rect drawing approximates but doesn't pixel-match canvas `lineWidth` rendering. This is acceptable for the tracer bullet. A production fix would use `imageproc::drawing::draw_antialiased_line_segment_mut` or manual pixel iteration for precise stroke width.

### 7.3 Expose composite_image command
**Modify** `src-tauri/src/commands/files.rs` — Add a new Tauri command:
```rust
#[tauri::command]
pub async fn composite_image(
    image_id: String,
    annotations: Vec<Annotation>,
    format: String,
    state: State<'_, ImageStore>,
) -> Result<Vec<u8>, String> { ... }
```
This is the **single compositing authority** per the file-operations spec. Both `save_image` and future `copy_to_clipboard` call through this.

### 7.4 Implement save_image command
**Modify** `src-tauri/src/commands/files.rs` — Add `State<'_, ImageStore>` parameter. Implementation:
1. Parse `image_id` as UUID. If invalid: `return Err(format!("Invalid image ID: {image_id}"))`
2. Get image from store. If not found: `return Err(format!("No image found for ID: {image_id}"))`
3. Call `composite::composite(&image, &annotations)` to get composited `RgbaImage`
4. Create parent dirs: `std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create directory: {e}"))?`
5. Encode and save based on `format` string:
   - `"png"`: `image.save_with_format(path, ImageFormat::Png)`
   - `"jpg"` | `"jpeg"`: `image.save_with_format(path, ImageFormat::Jpeg)` (use default quality for tracer bullet; configurable quality comes with settings)
   - `"webp"`: `image.save_with_format(path, ImageFormat::WebP)`
   - Other: `return Err(format!("Unsupported format: {format}"))`
6. On write error: `return Err(format!("Failed to save: {e}"))`
7. Return absolute path as `Ok(path)`

### 7.5 Register composite module and command
**Modify** `src-tauri/src/commands/mod.rs` — Add `pub mod composite;`.
**Modify** `src-tauri/src/lib.rs` — Add `commands::files::composite_image` to `invoke_handler`.

### 7.6 Verify
`just check` passes (compile-only).

### 7.7 Unit test: compositing
Add `#[cfg(test)]` in `composite.rs`:
- Create a 100x100 red image, add a rect annotation at (10, 10, 50, 50) with blue stroke
- Call `composite()`, verify pixel at (10, 10) is blue (stroke), pixel at (25, 25) is red (inside, no fill), pixel at (0, 0) is red (outside)

---

## Phase 8: Frontend Save Flow

**Goal**: Save button opens native file dialog with format options and writes composited image.

### 8.1 Wire save button
**Modify** `src-ui/js/app.js` — Query `[data-action="save"]` button. On click:
1. Check `store.get('currentImageId')` exists, else show status message "No image to save" and return
2. Open Tauri save dialog with format filters and default filename:
   ```js
   const path = await save({
     defaultPath: `fotos-${timestamp()}.png`,
     filters: [
       { name: 'PNG', extensions: ['png'] },
       { name: 'JPEG', extensions: ['jpg', 'jpeg'] },
       { name: 'WebP', extensions: ['webp'] },
     ]
   });
   ```
3. If user cancelled (`path === null`), return silently
4. Infer format from file extension: `path.split('.').pop().toLowerCase()` → map `'jpg'`/`'jpeg'` to `'jpg'`, `'webp'` to `'webp'`, default to `'png'`
5. Invoke save: `await invoke('save_image', { imageId, annotations: store.get('annotations'), format, path })`
6. Show success: `updateStatus({ message: \`Saved to ${path}\` })`
7. On error: `updateStatus({ message: \`Save failed: ${err}\` })`

**`timestamp()` helper**: returns `YYYY-MM-DD-HHmmss` from `new Date()`.

### 8.2 Verify (End-to-End)
1. `just dev` — app opens, empty canvas placeholder, annotation tools disabled
2. Click capture fullscreen — GNOME overlay, select area, image appears, tools enable
3. Status bar shows dimensions and "100%"
4. Mouse wheel zoom — zooms centered on cursor, percentage updates
5. Hold Space + drag — canvas pans
6. `Ctrl+0` — resets zoom and pan
7. Press R — crosshair cursor, status shows "Rectangle"
8. Click-drag: red rectangle preview. Release: rectangle commits
9. Draw second rectangle
10. `Ctrl+Z` — second rect disappears. `Ctrl+Shift+Z` — reappears
11. Zoom in — image and rects scale together, rects stay positioned correctly
12. Click Save — native dialog with PNG/JPEG/WebP filters, default filename `fotos-2026-02-19-143022.png`
13. Pick path, confirm — PNG saved, status bar shows "Saved to ..."
14. Open saved PNG in viewer — screenshot with both rectangles composited at correct positions

### 8.3 Error path verification
- Try saving with no image loaded — status shows "No image to save"
- Cancel the save dialog — nothing happens, no error
- Save to a read-only path — status shows "Save failed: ..." with descriptive error

---

## Files Summary

| Action | File | Changes |
|--------|------|---------|
| **Create** | `src-tauri/capabilities/default.json` | Tauri 2 plugin permissions |
| **Create** | `src-tauri/src/capture/store.rs` | `ImageStore` with Mutex + HashMap + unit tests |
| **Create** | `src-ui/js/canvas/commands.js` | `AddAnnotationCommand` (consumed by existing `History`) |
| **Create** | `src-tauri/src/commands/composite.rs` | `composite()` function + unit tests |
| **Modify** | `src-tauri/src/lib.rs` | Add `.setup()` hook, `.manage(ImageStore)`, `composite_image` to handler, tracing init |
| **Modify** | `src-tauri/src/capture/mod.rs` | `Arc<DynamicImage>`, `pub mod store`, `capture()` routing fn |
| **Modify** | `src-tauri/src/capture/detect.rs` | `WAYLAND_DISPLAY` fallback when `XDG_SESSION_TYPE` is unset |
| **Modify** | `src-tauri/src/capture/portal.rs` | Implement portal capture with URI validation + error handling |
| **Modify** | `src-tauri/src/capture/xcap_backend.rs` | Implement fullscreen compositing ALL monitors |
| **Modify** | `src-tauri/src/commands/capture.rs` | Implement `take_screenshot` with `State<ImageStore>` |
| **Modify** | `src-tauri/src/commands/files.rs` | `serde rename_all`, add missing Annotation fields, implement `save_image` + `composite_image` command, error handling |
| **Modify** | `src-tauri/src/commands/mod.rs` | Add `pub mod composite` |
| **Modify** | `src-ui/js/canvas/engine.js` | Full implementation: triple-layer render, zoom clamp, pan, HiDPI, empty state |
| **Modify** | `src-ui/js/app.js` | Capture flow, zoom (+/-/Ctrl+0/wheel), Space+drag pan, rect tool, undo/redo, save flow, status bar |
| **Modify** | `src-ui/css/main.css` | Disabled button style |

**Existing files leveraged (not modified):**
- `src-ui/js/canvas/history.js` — command pattern undo/redo (100-limit, FIFO, redo clearing)
- `src-ui/js/state.js` — `StateStore` singleton with all state keys
- `src-ui/js/ui/toolbar.js` — tool button wiring and keyboard shortcuts
- `src-ui/index.html` — complete HTML shell
- `src-ui/css/main.css` — existing layout (only adding disabled style)

## Risks

1. **ashpd inside Tauri process**: Portal D-Bus call may need `WindowIdentifier`. Mitigation: test Phase 2 early; if needed, pass Wayland handle.
2. **Base64 for large images**: 4K screenshot = ~13MB base64 string through IPC. Acceptable for tracer bullet; optimize later with `convertFileSrc` or `tauri://localhost` asset protocol.
3. **imageproc 0.25 API**: `draw_hollow_rect_mut` signature may differ from docs. Fallback: manual pixel rect drawing with `image::GenericImage::put_pixel`.
4. **Multi-monitor compositing**: xcap monitor positions may use negative coordinates or non-zero origins. Need to compute bounding box carefully with signed offsets.
