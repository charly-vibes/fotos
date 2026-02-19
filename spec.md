# SnapLens — AI-Powered Screenshot & Annotation Tool

## Project Specification v1.0

---

## 1. Project Overview

**SnapLens** is a cross-platform screenshot capture, annotation, and AI analysis tool built with Tauri 2 and vanilla JavaScript. It targets Linux (GNOME Wayland, KDE Wayland/X11), and Windows. It exposes an MCP (Model Context Protocol) server so AI agents (Claude Desktop, Cursor, etc.) can programmatically capture, annotate, OCR, and analyze screenshots.

### Core Principles

- **No web framework** — vanilla JS/HTML/CSS with ES modules, HTML5 Canvas for annotation
- **Rust does the heavy lifting** — capture, OCR, PII detection, LLM calls, file I/O, MCP server
- **Cross-platform via Tauri 2** — single codebase, native webview per OS
- **AI-first** — OCR, auto-blur PII, LLM vision analysis are core features, not plugins
- **MCP server** — AI agents can take screenshots, annotate, extract text, and analyze images

### Target Platforms

| Platform | Compositor | Capture Method | Package Format |
|---|---|---|---|
| Bluefin / Fedora GNOME | Wayland | xdg-desktop-portal (GNOME) | Flatpak |
| KDE Plasma 6 | Wayland | xdg-desktop-portal (KDE) | Flatpak / AppImage |
| KDE / Generic Linux | X11 | XShm / XGetImage via xcap | Flatpak / AppImage |
| Windows 10/11 | DWM | Windows.Graphics.Capture via xcap | MSI / NSIS |

---

## 2. Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Tauri 2 Shell                         │
│  ┌────────────────────────────────────────────────────┐  │
│  │              Frontend (Vanilla JS)                 │  │
│  │  ┌──────────┐ ┌──────────────┐ ┌───────────────┐  │  │
│  │  │ Toolbar  │ │ Canvas Engine│ │  AI Panel     │  │  │
│  │  │ & Tools  │ │ (HTML5 2D)   │ │  (OCR, LLM)  │  │  │
│  │  └──────────┘ └──────────────┘ └───────────────┘  │  │
│  └──────────────────┬─────────────────────────────────┘  │
│                     │ invoke() / events                   │
│  ┌──────────────────▼─────────────────────────────────┐  │
│  │              Rust Backend                          │  │
│  │  ┌──────────────┐  ┌────────────┐  ┌───────────┐  │  │
│  │  │ CaptureManager│  │ AIProcessor│  │ FileManager│ │  │
│  │  │ (xcap + portal)│ │ (OCR, LLM) │  │ (save/load)│ │  │
│  │  └──────────────┘  └────────────┘  └───────────┘  │  │
│  │  ┌──────────────┐  ┌────────────┐                  │  │
│  │  │ Credentials  │  │ SettingsStore│                 │  │
│  │  │ (OS keychain)│  │ (tauri-store)│                 │  │
│  │  └──────────────┘  └────────────┘                  │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│              MCP Server (separate binary)                 │
│  snaplens-mcp — communicates with main app via IPC/D-Bus │
│  Exposes: take_screenshot, ocr, annotate, analyze, list  │
│  Transport: stdio (JSON-RPC 2.0)                         │
└──────────────────────────────────────────────────────────┘
```

### Process Model

**Process 1 — Main Tauri App** (`snaplens`)
- GTK webview on Linux, WebView2 on Windows
- Handles UI, annotation canvas, user interaction
- Rust backend manages capture, AI, file operations
- Listens on a local IPC channel (Unix socket on Linux, named pipe on Windows)

**Process 2 — MCP Server** (`snaplens-mcp`)
- Standalone binary, invoked by MCP hosts via stdio
- Sends commands to Process 1 via IPC
- Stateless — delegates all work to the main app
- Can also operate standalone (capture + return, no UI)

---

## 3. Repository Structure

```
snaplens/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── tauri.conf.json               # Tauri configuration
├── README.md
├── LICENSE                       # MIT or Apache-2.0
│
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   ├── src/
│   │   ├── main.rs              # Tauri entry point
│   │   ├── lib.rs               # Shared library (used by both app and MCP)
│   │   ├── commands/            # Tauri invoke handlers
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs       # take_screenshot, list_monitors
│   │   │   ├── ai.rs            # run_ocr, analyze_llm, auto_blur_pii
│   │   │   ├── files.rs         # save_image, load_image, export
│   │   │   └── settings.rs      # get/set preferences, API keys
│   │   ├── capture/             # Platform capture abstraction
│   │   │   ├── mod.rs           # CaptureManager trait
│   │   │   ├── xcap_backend.rs  # xcap for X11 + Windows
│   │   │   ├── portal.rs        # xdg-desktop-portal for Wayland
│   │   │   └── detect.rs        # Runtime platform/compositor detection
│   │   ├── ai/                  # AI processing
│   │   │   ├── mod.rs
│   │   │   ├── ocr.rs           # Tesseract wrapper
│   │   │   ├── pii.rs           # PII detection + blur coordinates
│   │   │   ├── llm.rs           # Cloud LLM vision (Claude, GPT-4o, Gemini)
│   │   │   └── ollama.rs        # Local LLM via Ollama
│   │   ├── ipc/                 # Inter-process communication
│   │   │   ├── mod.rs
│   │   │   ├── server.rs        # IPC server (main app side)
│   │   │   └── client.rs        # IPC client (MCP server side)
│   │   └── credentials.rs       # OS keychain abstraction
│   │
│   └── resources/
│       └── tessdata/            # Tesseract language data (eng.traineddata)
│
├── src-mcp/                     # MCP server binary
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs              # MCP stdio entry point
│   │   ├── server.rs            # MCP tool/resource/prompt definitions
│   │   └── bridge.rs            # IPC client to main app
│   └── mcp-manifest.json        # MCP server manifest
│
├── src-ui/                      # Frontend (vanilla JS)
│   ├── index.html               # App shell
│   ├── css/
│   │   ├── main.css             # Layout, toolbar, panels
│   │   ├── canvas.css           # Canvas overlay styles
│   │   └── themes.css           # Light/dark via prefers-color-scheme
│   ├── js/
│   │   ├── app.js               # Entry: init, wire modules, global state
│   │   ├── state.js             # Central state object + event emitter
│   │   ├── tauri-bridge.js      # Typed wrappers around invoke()
│   │   ├── canvas/
│   │   │   ├── engine.js        # Render loop, layers, coordinate transforms
│   │   │   ├── tools.js         # Tool definitions (arrow, rect, text, etc.)
│   │   │   ├── tool-arrow.js    # Arrow tool implementation
│   │   │   ├── tool-rect.js     # Rectangle tool
│   │   │   ├── tool-ellipse.js  # Ellipse tool
│   │   │   ├── tool-text.js     # Text annotation tool
│   │   │   ├── tool-blur.js     # Blur/pixelate region tool
│   │   │   ├── tool-step.js     # Auto-incrementing step numbers
│   │   │   ├── tool-freehand.js # Freehand drawing
│   │   │   ├── tool-highlight.js# Semi-transparent highlight
│   │   │   ├── tool-crop.js     # Crop tool
│   │   │   ├── history.js       # Command pattern undo/redo
│   │   │   └── selection.js     # Object select, move, resize handles
│   │   ├── ui/
│   │   │   ├── toolbar.js       # Tool buttons, groups
│   │   │   ├── color-picker.js  # Color + opacity picker
│   │   │   ├── size-picker.js   # Stroke width / font size
│   │   │   ├── ai-panel.js      # OCR results, LLM response display
│   │   │   ├── export-dialog.js # Save / copy / upload options
│   │   │   └── settings.js      # Preferences modal
│   │   └── utils/
│   │       ├── dom.js           # DOM helpers, event delegation
│   │       ├── geometry.js      # Point, Rect, hit-testing math
│   │       └── debounce.js      # Input debouncing
│   └── assets/
│       ├── icons/               # Tool icons (SVG)
│       └── fonts/               # Optional bundled fonts
│
├── scripts/
│   ├── build-flatpak.sh         # Flatpak build script
│   └── dev.sh                   # Dev environment setup
│
├── flatpak/
│   ├── io.github.snaplens.SnapLens.yml  # Flatpak manifest
│   └── io.github.snaplens.SnapLens.desktop
│
└── tests/
    ├── rust/                    # Rust integration tests
    └── js/                      # JS unit tests (no framework, plain assertions)
```

---

## 4. Rust Dependencies

### src-tauri/Cargo.toml

```toml
[package]
name = "snaplens"
version = "0.1.0"
edition = "2021"

[dependencies]
# Tauri core
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-clipboard-manager = "2"
tauri-plugin-global-shortcut = "2"
tauri-plugin-os = "2"
tauri-plugin-store = "2"                  # Persistent settings

# Screenshot capture
xcap = "0.8"                              # Cross-platform capture (X11, Wayland, Windows)

# Wayland portal (Linux only)
ashpd = { version = "0.11", optional = true }  # xdg-desktop-portal async wrapper
zbus = { version = "5", optional = true }       # D-Bus IPC

# Image processing
image = "0.25"                            # Image loading, encoding, basic transforms
imageproc = "0.25"                        # Blur, filters, drawing primitives
ab_glyph = "0.2"                          # Font rasterization for text annotations

# AI / OCR
tesseract-rs = "0.3"                      # Tesseract OCR bindings
regex = "1"                               # PII pattern matching

# LLM API clients
reqwest = { version = "0.12", features = ["json", "multipart"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.22"

# IPC between main app and MCP server
interprocess = "2"                         # Cross-platform IPC (Unix sockets / named pipes)

# Credential storage
keyring = "3"                              # OS keychain (GNOME Keyring, KWallet, Windows Credential Manager)

# Utilities
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
directories = "6"                          # XDG / AppData paths

[features]
default = ["linux-portal"]
linux-portal = ["ashpd", "zbus"]

[target.'cfg(target_os = "linux")'.dependencies]
ashpd = "0.11"
zbus = "5"
```

### src-mcp/Cargo.toml

```toml
[package]
name = "snaplens-mcp"
version = "0.1.0"
edition = "2021"

[dependencies]
# MCP protocol
rmcp = "0.1"                              # Rust MCP SDK (or hand-roll JSON-RPC)
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# IPC to main app
interprocess = "2"

# Image handling
base64 = "0.22"
image = "0.25"

# Utilities
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## 5. Tauri Configuration

### tauri.conf.json

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "SnapLens",
  "identifier": "io.github.snaplens",
  "version": "0.1.0",
  "build": {
    "frontendDist": "../src-ui"
  },
  "app": {
    "windows": [
      {
        "title": "SnapLens",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "decorations": true,
        "visible": false
      }
    ],
    "trayIcon": {
      "iconPath": "icons/tray.png",
      "tooltip": "SnapLens"
    },
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: blob:; script-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  },
  "plugins": {
    "global-shortcut": {
      "shortcuts": ["PrintScreen", "Alt+PrintScreen", "Ctrl+Shift+S"]
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage", "msi", "nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": [
      "resources/tessdata/*"
    ],
    "externalBin": [
      "snaplens-mcp"
    ]
  }
}
```

---

## 6. Screenshot Capture Layer

### Platform Detection (src-tauri/src/capture/detect.rs)

```rust
pub enum Platform {
    LinuxWaylandGnome,
    LinuxWaylandKde,
    LinuxWaylandOther,   // Hyprland, Sway, etc.
    LinuxX11,
    Windows,
}

pub fn detect_platform() -> Platform {
    // Check XDG_SESSION_TYPE, XDG_CURRENT_DESKTOP, WAYLAND_DISPLAY
    // Falls back to xcap for X11 and Windows
}
```

### Capture Strategy

```rust
pub enum CaptureMode {
    Fullscreen,             // All monitors composited
    Monitor(u32),           // Specific monitor by index
    Region(Rect),           // User-selected rectangle
    Window(WindowId),       // Specific window
}

pub struct CaptureResult {
    pub id: Uuid,
    pub image: DynamicImage,
    pub metadata: CaptureMetadata,
}

pub struct CaptureMetadata {
    pub timestamp: DateTime<Utc>,
    pub mode: CaptureMode,
    pub monitor: Option<String>,
    pub window_title: Option<String>,
    pub dimensions: (u32, u32),
}

#[async_trait]
pub trait CaptureBackend {
    async fn capture(&self, mode: CaptureMode) -> Result<CaptureResult>;
    async fn list_monitors(&self) -> Result<Vec<MonitorInfo>>;
    async fn list_windows(&self) -> Result<Vec<WindowInfo>>;
}
```

### Wayland Portal Flow

```
User presses hotkey
    → Tauri global-shortcut fires
    → Rust: detect_platform() → LinuxWaylandGnome
    → Rust: ashpd Screenshot::request().interactive(true).send().await
    → GNOME Shell shows native area selector overlay
    → User selects region
    → Portal returns file:///tmp/screenshot-XXXX.png
    → Rust: load image, assign UUID, store in memory
    → Rust: emit "screenshot-ready" event to frontend
    → Frontend: display image on canvas, show annotation toolbar
```

### Windows / X11 Flow (via xcap)

```
User presses hotkey
    → Tauri global-shortcut fires
    → Rust: detect_platform() → Windows / LinuxX11
    → Rust: xcap Monitor::all() or Window::all()
    → For region mode: frontend shows transparent overlay for selection
    → xcap captures → returns image buffer
    → Rust: assign UUID, store
    → emit "screenshot-ready" → frontend canvas
```

---

## 7. Frontend Architecture

### index.html — App Shell

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>SnapLens</title>
  <link rel="stylesheet" href="css/main.css">
  <link rel="stylesheet" href="css/canvas.css">
  <link rel="stylesheet" href="css/themes.css">
</head>
<body>
  <!-- Top toolbar -->
  <header id="toolbar">
    <div class="tool-group" id="capture-tools">
      <button data-action="capture-region" title="Capture Region (Ctrl+Shift+S)">
        <!-- SVG icon -->
      </button>
      <button data-action="capture-fullscreen" title="Capture Fullscreen (PrtScn)">
      </button>
      <button data-action="capture-window" title="Capture Window (Alt+PrtScn)">
      </button>
    </div>

    <div class="separator"></div>

    <div class="tool-group" id="annotation-tools">
      <button data-tool="arrow" title="Arrow (A)"></button>
      <button data-tool="rect" title="Rectangle (R)"></button>
      <button data-tool="ellipse" title="Ellipse (E)"></button>
      <button data-tool="text" title="Text (T)"></button>
      <button data-tool="blur" title="Blur (B)"></button>
      <button data-tool="step" title="Step Number (N)"></button>
      <button data-tool="freehand" title="Freehand (F)"></button>
      <button data-tool="highlight" title="Highlight (H)"></button>
      <button data-tool="crop" title="Crop (C)"></button>
      <button data-tool="select" title="Select (V)"></button>
    </div>

    <div class="separator"></div>

    <div class="tool-group" id="style-controls">
      <div id="color-picker-trigger"></div>
      <div id="size-picker"></div>
    </div>

    <div class="separator"></div>

    <div class="tool-group" id="history-controls">
      <button data-action="undo" title="Undo (Ctrl+Z)"></button>
      <button data-action="redo" title="Redo (Ctrl+Shift+Z)"></button>
    </div>

    <div class="spacer"></div>

    <div class="tool-group" id="ai-tools">
      <button data-action="ocr" title="Extract Text (OCR)"></button>
      <button data-action="auto-blur" title="Auto-Blur PII"></button>
      <button data-action="ai-analyze" title="AI Analyze"></button>
    </div>

    <div class="separator"></div>

    <div class="tool-group" id="output-tools">
      <button data-action="copy-clipboard" title="Copy (Ctrl+C)"></button>
      <button data-action="save" title="Save (Ctrl+S)"></button>
      <button data-action="save-as" title="Save As (Ctrl+Shift+S)"></button>
    </div>
  </header>

  <!-- Main canvas area -->
  <main id="canvas-container">
    <canvas id="canvas-base"></canvas>     <!-- Screenshot image layer -->
    <canvas id="canvas-annotations"></canvas> <!-- Committed annotations -->
    <canvas id="canvas-active"></canvas>   <!-- Active tool preview -->
    <!-- Floating text input for text tool -->
    <textarea id="text-input" class="hidden"></textarea>
  </main>

  <!-- Collapsible AI results panel -->
  <aside id="ai-panel" class="collapsed">
    <div id="ai-panel-header">
      <span>AI Results</span>
      <button data-action="toggle-ai-panel"></button>
    </div>
    <div id="ai-panel-content">
      <div id="ocr-results" class="hidden"></div>
      <div id="llm-results" class="hidden"></div>
    </div>
  </aside>

  <!-- Status bar -->
  <footer id="statusbar">
    <span id="status-dimensions"></span>
    <span id="status-zoom"></span>
    <span id="status-tool"></span>
    <span id="status-message"></span>
  </footer>

  <script type="module" src="js/app.js"></script>
</body>
</html>
```

### State Management (js/state.js)

```javascript
// Simple event emitter + state store — no framework needed
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
    return () => this.#listeners.get(key).delete(fn); // unsubscribe
  }

  #emit(key, value, old) {
    this.#listeners.get(key)?.forEach(fn => fn(value, old));
  }
}

export const store = new StateStore({
  // Current tool
  activeTool: 'arrow',
  // Tool style
  strokeColor: '#FF0000',
  fillColor: 'transparent',
  strokeWidth: 2,
  fontSize: 16,
  opacity: 1.0,
  // Canvas state
  zoom: 1.0,
  panX: 0,
  panY: 0,
  // Screenshot
  currentImageId: null,
  // Annotations array — source of truth
  annotations: [],
  // History
  undoStack: [],
  redoStack: [],
  // Step counter
  nextStepNumber: 1,
  // AI
  ocrResults: null,
  llmResults: null,
  isProcessing: false,
});
```

### Canvas Engine (js/canvas/engine.js)

```javascript
// Triple-layer canvas for performance:
// Layer 0 (canvas-base):        Screenshot image — redrawn only on load/zoom/pan
// Layer 1 (canvas-annotations): Committed annotations — redrawn on annotation change
// Layer 2 (canvas-active):      Active tool preview — redrawn on every mouse move

export class CanvasEngine {
  #baseCtx;          // Screenshot layer
  #annoCtx;          // Annotation layer
  #activeCtx;        // Active tool layer
  #image = null;     // Current screenshot as ImageBitmap
  #transform;        // Current zoom + pan matrix

  constructor(baseCanvas, annoCanvas, activeCanvas) {
    this.#baseCtx = baseCanvas.getContext('2d');
    this.#annoCtx = annoCanvas.getContext('2d');
    this.#activeCtx = activeCanvas.getContext('2d');
    // ... size canvases, attach resize observer
  }

  loadImage(imageData) {
    // Create ImageBitmap from ArrayBuffer, store, trigger redraw
  }

  // Convert screen coords (mouse event) to image coords
  screenToImage(screenX, screenY) {
    // Apply inverse of current transform (zoom + pan)
  }

  // Redraw base layer (screenshot only)
  renderBase() {
    // Clear, apply transform, drawImage
  }

  // Redraw all committed annotations
  renderAnnotations(annotations) {
    // Clear, apply transform, iterate annotations, draw each
  }

  // Redraw active tool preview (called on mousemove)
  renderActive(previewShape) {
    // Clear, apply transform, draw single shape
  }

  // Export final composited image
  exportComposite(annotations, format = 'png') {
    // Create offscreen canvas at original image dimensions
    // Draw base image
    // Draw all annotations (at original scale, no transform)
    // Return as Blob
  }
}
```

### Command Pattern for Undo/Redo (js/canvas/history.js)

```javascript
// Each annotation action is a Command with execute() and undo()
// Only store deltas, not full canvas snapshots

class AddAnnotationCommand {
  constructor(annotation) {
    this.annotation = annotation;
  }
  execute(annotations) {
    annotations.push(this.annotation);
    return annotations;
  }
  undo(annotations) {
    return annotations.filter(a => a.id !== this.annotation.id);
  }
}

class MoveAnnotationCommand {
  constructor(id, fromPos, toPos) {
    this.id = id;
    this.fromPos = fromPos;
    this.toPos = toPos;
  }
  execute(annotations) { /* update position to toPos */ }
  undo(annotations) { /* update position to fromPos */ }
}

class DeleteAnnotationCommand { /* inverse of Add */ }
class ModifyStyleCommand { /* stores old + new style */ }

export class History {
  #undoStack = [];
  #redoStack = [];
  #maxSize = 100;

  execute(command, annotations) {
    const result = command.execute(annotations);
    this.#undoStack.push(command);
    this.#redoStack = []; // clear redo on new action
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
}
```

### Annotation Data Model

```javascript
// Every annotation is a plain object — serializable, no class instances
// This is what gets stored, sent to Rust, and used by MCP

const annotationSchema = {
  id: 'uuid-string',
  type: 'arrow|rect|ellipse|text|blur|step|freehand|highlight|crop',
  // Geometry (image coordinates, not screen coordinates)
  x: 0, y: 0,                    // Top-left origin
  width: 0, height: 0,           // Bounding box (for rect, ellipse, blur, crop)
  points: [],                     // For arrow: [{x,y},{x,y}], freehand: [{x,y},...]
  // Style
  strokeColor: '#FF0000',
  fillColor: 'transparent',
  strokeWidth: 2,
  opacity: 1.0,
  // Type-specific
  text: '',                       // For text and step annotations
  fontSize: 16,                   // For text
  fontFamily: 'sans-serif',
  stepNumber: 1,                  // For step tool
  blurRadius: 10,                 // For blur tool
  highlightColor: '#FFFF00',      // For highlight (always semi-transparent)
  // Metadata
  createdAt: 'ISO-timestamp',
  locked: false,
};
```

### Keyboard Shortcuts

| Key | Action |
|---|---|
| `V` | Select tool |
| `A` | Arrow tool |
| `R` | Rectangle tool |
| `E` | Ellipse tool |
| `T` | Text tool |
| `B` | Blur tool |
| `N` | Step number tool |
| `F` | Freehand tool |
| `H` | Highlight tool |
| `C` | Crop tool |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+C` | Copy to clipboard |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save as |
| `Delete` | Delete selected annotation |
| `Escape` | Deselect / cancel current tool |
| `Ctrl+A` | Select all annotations |
| `+` / `-` | Zoom in / out |
| `Ctrl+0` | Reset zoom |
| `Space` + drag | Pan canvas |

---

## 8. Tauri Commands (Rust ↔ JS Interface)

These are the `#[tauri::command]` functions the frontend calls via `invoke()`.

### Capture

```rust
#[tauri::command]
async fn take_screenshot(mode: String, monitor: Option<u32>) -> Result<ScreenshotResponse, String>;
// Returns: { id: String, width: u32, height: u32, dataUrl: String }

#[tauri::command]
async fn list_monitors() -> Result<Vec<MonitorInfo>, String>;

#[tauri::command]
async fn list_windows() -> Result<Vec<WindowInfo>, String>;
```

### AI Processing

```rust
#[tauri::command]
async fn run_ocr(image_id: String, lang: Option<String>) -> Result<OcrResult, String>;
// Returns: { text: String, regions: [{ text, x, y, w, h, confidence }] }

#[tauri::command]
async fn auto_blur_pii(image_id: String) -> Result<Vec<BlurRegion>, String>;
// Returns array of regions to blur: [{ x, y, w, h, pii_type: "email|phone|ssn|..." }]

#[tauri::command]
async fn analyze_llm(
    image_id: String,
    prompt: Option<String>,
    provider: String,         // "claude" | "openai" | "gemini" | "ollama"
) -> Result<LlmResponse, String>;
// Returns: { response: String, model: String, tokens_used: u32 }
```

### File Operations

```rust
#[tauri::command]
async fn save_image(
    image_id: String,
    annotations: Vec<Annotation>,
    format: String,            // "png" | "jpg" | "webp"
    path: String,
) -> Result<String, String>;

#[tauri::command]
async fn copy_to_clipboard(
    image_id: String,
    annotations: Vec<Annotation>,
) -> Result<(), String>;

#[tauri::command]
async fn export_annotations(
    image_id: String,
    annotations: Vec<Annotation>,
) -> Result<String, String>;    // Returns JSON of annotations for reimport
```

### Settings & Credentials

```rust
#[tauri::command]
async fn set_api_key(provider: String, key: String) -> Result<(), String>;
// Stores in OS keychain via keyring crate

#[tauri::command]
async fn get_settings() -> Result<Settings, String>;

#[tauri::command]
async fn set_settings(settings: Settings) -> Result<(), String>;
```

---

## 9. AI Processing Pipeline

### OCR (Tesseract)

```
Input: Screenshot image (PNG bytes)
    → Rust: load image, convert to grayscale
    → Rust: tesseract::TessApi::set_image()
    → Rust: get_text() → full extracted text
    → Rust: get_component_images(RIL_WORD) → bounding boxes per word
    → Return: { text, regions: [{ text, bbox, confidence }] }
```

**Configuration:**
- Default language: `eng`
- Bundled tessdata: `eng.traineddata` (~30MB)
- Additional languages: downloaded on demand to app data dir
- PSM mode: 3 (fully automatic page segmentation)

### PII Auto-Detection

Runs OCR first, then applies pattern matching on the extracted text with bounding boxes:

```rust
pub struct PiiDetector {
    patterns: Vec<PiiPattern>,
}

pub struct PiiPattern {
    name: &'static str,          // "email", "phone", "ssn", "credit_card", "api_key"
    regex: Regex,
}

// Built-in patterns:
// Email:       [a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}
// Phone (US):  \b(\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b
// SSN:         \b\d{3}-\d{2}-\d{4}\b
// Credit card: \b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b
// API keys:    \b(sk-[a-zA-Z0-9]{32,}|AKIA[A-Z0-9]{16}|ghp_[a-zA-Z0-9]{36})\b
// IP address:  \b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b
// AWS ARN:     \barn:aws:[a-z0-9-]+:[a-z0-9-]*:\d{12}:
```

**Pipeline:**
1. Run OCR with bounding boxes
2. For each detected text region, test against all PII patterns
3. Return list of `{ bbox, pii_type }` for matched regions
4. Frontend auto-creates blur annotations at those coordinates

### LLM Vision Analysis

```rust
pub struct LlmClient {
    provider: LlmProvider,
    api_key: String,
    model: String,
}

pub enum LlmProvider {
    Claude { model: String },        // claude-sonnet-4-20250514
    OpenAI { model: String },        // gpt-4o
    Gemini { model: String },        // gemini-2.0-flash
    Ollama { model: String, url: String },  // llava:7b @ localhost:11434
}
```

**Default prompts:**
- **Describe**: "Describe what you see in this screenshot in detail."
- **Extract code**: "Extract all code visible in this screenshot. Return only the code, properly formatted."
- **Bug report**: "Analyze this screenshot and generate a bug report with: summary, steps to reproduce (inferred), expected vs actual behavior, and severity assessment."
- **Accessibility**: "Generate alt-text for this screenshot suitable for screen readers."
- **Custom**: User-provided prompt

---

## 10. MCP Server Specification

### Server Identity

```json
{
  "name": "snaplens",
  "version": "0.1.0",
  "description": "AI-powered screenshot capture, annotation, and analysis"
}
```

### Tools

#### `take_screenshot`

```json
{
  "name": "take_screenshot",
  "description": "Capture a screenshot of the desktop, a specific monitor, or a window",
  "inputSchema": {
    "type": "object",
    "properties": {
      "mode": {
        "type": "string",
        "enum": ["fullscreen", "monitor", "window"],
        "description": "Capture mode",
        "default": "fullscreen"
      },
      "monitor_index": {
        "type": "integer",
        "description": "Monitor index (for monitor mode)"
      },
      "window_title": {
        "type": "string",
        "description": "Window title substring to match (for window mode)"
      },
      "delay_ms": {
        "type": "integer",
        "description": "Delay before capture in milliseconds",
        "default": 0
      }
    }
  }
}
```

**Returns:** Image content block (base64 PNG) + text metadata (dimensions, timestamp, screenshot ID).

#### `ocr_screenshot`

```json
{
  "name": "ocr_screenshot",
  "description": "Extract text from a screenshot using OCR",
  "inputSchema": {
    "type": "object",
    "properties": {
      "screenshot_id": {
        "type": "string",
        "description": "ID of a previously captured screenshot, or omit to capture a new one"
      },
      "language": {
        "type": "string",
        "description": "OCR language code (e.g., 'eng', 'deu', 'jpn')",
        "default": "eng"
      }
    }
  }
}
```

**Returns:** Text content block with extracted text + structured regions.

#### `annotate_screenshot`

```json
{
  "name": "annotate_screenshot",
  "description": "Add annotations to a screenshot",
  "inputSchema": {
    "type": "object",
    "properties": {
      "screenshot_id": { "type": "string" },
      "annotations": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "type": { "type": "string", "enum": ["arrow", "rect", "ellipse", "text", "blur"] },
            "x": { "type": "number" },
            "y": { "type": "number" },
            "width": { "type": "number" },
            "height": { "type": "number" },
            "points": { "type": "array", "items": { "type": "object" } },
            "text": { "type": "string" },
            "color": { "type": "string", "default": "#FF0000" },
            "stroke_width": { "type": "number", "default": 2 }
          },
          "required": ["type"]
        }
      }
    },
    "required": ["screenshot_id", "annotations"]
  }
}
```

**Returns:** Annotated image as base64 PNG.

#### `analyze_screenshot`

```json
{
  "name": "analyze_screenshot",
  "description": "Analyze a screenshot using an LLM vision model",
  "inputSchema": {
    "type": "object",
    "properties": {
      "screenshot_id": { "type": "string" },
      "prompt": {
        "type": "string",
        "description": "Analysis prompt",
        "default": "Describe what you see in this screenshot"
      },
      "provider": {
        "type": "string",
        "enum": ["claude", "openai", "gemini", "ollama"],
        "default": "claude"
      }
    }
  }
}
```

#### `auto_redact_pii`

```json
{
  "name": "auto_redact_pii",
  "description": "Detect and blur personally identifiable information in a screenshot",
  "inputSchema": {
    "type": "object",
    "properties": {
      "screenshot_id": { "type": "string" }
    },
    "required": ["screenshot_id"]
  }
}
```

**Returns:** Redacted image + list of detected PII types and locations.

#### `list_screenshots`

```json
{
  "name": "list_screenshots",
  "description": "List recent screenshots in the session",
  "inputSchema": {
    "type": "object",
    "properties": {
      "limit": { "type": "integer", "default": 10 }
    }
  }
}
```

### Resources

```
screenshots://recent          → List of recent screenshot metadata
screenshots://{id}            → Specific screenshot image + metadata
screenshots://{id}/ocr        → Cached OCR results for a screenshot
settings://current            → Current app settings
```

### Prompts

```json
[
  {
    "name": "describe_ui",
    "description": "Describe the UI elements visible in a screenshot",
    "arguments": [{ "name": "screenshot_id", "required": true }]
  },
  {
    "name": "extract_code",
    "description": "Extract all code visible in a screenshot",
    "arguments": [{ "name": "screenshot_id", "required": true }]
  },
  {
    "name": "generate_bug_report",
    "description": "Generate a bug report from a screenshot showing an error",
    "arguments": [
      { "name": "screenshot_id", "required": true },
      { "name": "context", "description": "Additional context about the bug" }
    ]
  },
  {
    "name": "accessibility_audit",
    "description": "Audit a UI screenshot for accessibility issues",
    "arguments": [{ "name": "screenshot_id", "required": true }]
  }
]
```

### MCP Host Configuration

**Claude Desktop / Claude Code:**
```json
{
  "mcpServers": {
    "snaplens": {
      "command": "snaplens-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

**Flatpak variant:**
```json
{
  "mcpServers": {
    "snaplens": {
      "command": "flatpak",
      "args": ["run", "--command=snaplens-mcp", "io.github.snaplens.SnapLens"]
    }
  }
}
```

---

## 11. Settings & Configuration

### User Settings Schema

```json
{
  "capture": {
    "defaultMode": "region",
    "includeMouseCursor": false,
    "delayMs": 0,
    "saveDirectory": "~/Pictures/SnapLens",
    "defaultFormat": "png",
    "jpegQuality": 90,
    "copyToClipboardAfterCapture": true
  },
  "annotation": {
    "defaultStrokeColor": "#FF0000",
    "defaultStrokeWidth": 2,
    "defaultFontSize": 16,
    "defaultFontFamily": "sans-serif",
    "stepNumberColor": "#FF0000",
    "stepNumberSize": 24,
    "blurRadius": 10
  },
  "ai": {
    "ocrLanguage": "eng",
    "defaultLlmProvider": "claude",
    "ollamaUrl": "http://localhost:11434",
    "ollamaModel": "llava:7b",
    "claudeModel": "claude-sonnet-4-20250514",
    "openaiModel": "gpt-4o",
    "geminiModel": "gemini-2.0-flash"
  },
  "ui": {
    "theme": "system",
    "showAiPanel": true,
    "showStatusBar": true
  }
}
```

### API Key Storage

API keys are stored in the OS keychain, never in config files:

| Provider | Keychain Service | Keychain Account |
|---|---|---|
| Anthropic | `snaplens` | `anthropic-api-key` |
| OpenAI | `snaplens` | `openai-api-key` |
| Google | `snaplens` | `google-api-key` |

---

## 12. Packaging & Distribution

### Flatpak (Linux Primary)

```yaml
# flatpak/io.github.snaplens.SnapLens.yml
app-id: io.github.snaplens.SnapLens
runtime: org.gnome.Platform
runtime-version: '47'
sdk: org.gnome.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
command: snaplens

finish-args:
  - --socket=wayland
  - --socket=fallback-x11
  - --socket=pulseaudio            # Notification sounds
  - --device=dri                   # GPU acceleration
  - --share=ipc                    # X11 shared memory
  - --share=network                # Cloud AI API calls
  - --filesystem=xdg-pictures:create
  - --talk-name=org.freedesktop.portal.Desktop
  - --talk-name=org.freedesktop.portal.Screenshot
  - --talk-name=org.freedesktop.secrets   # Keychain access

modules:
  - name: tesseract
    buildsystem: cmake-ninja
    sources:
      - type: archive
        url: https://github.com/tesseract-ocr/tesseract/archive/5.5.1.tar.gz

  - name: tessdata-eng
    buildsystem: simple
    build-commands:
      - install -Dm644 eng.traineddata /app/share/tessdata/eng.traineddata
    sources:
      - type: file
        url: https://github.com/tesseract-ocr/tessdata_fast/raw/main/eng.traineddata

  - name: snaplens
    buildsystem: simple
    build-commands:
      - cargo build --release
      - install -Dm755 target/release/snaplens /app/bin/snaplens
      - install -Dm755 target/release/snaplens-mcp /app/bin/snaplens-mcp
```

### Windows (MSI via Tauri)

Tauri generates MSI and NSIS installers automatically:

```bash
# Build for Windows
cargo tauri build --target x86_64-pc-windows-msvc
# Output: target/release/bundle/msi/SnapLens_0.1.0_x64.msi
```

Tesseract for Windows is bundled as a DLL + tessdata in the installer resources.

### AppImage (Linux Fallback)

```bash
cargo tauri build --bundles appimage
```

---

## 13. Development Phases

### Phase 1 — MVP (Capture + Annotate + Save)

**Goal:** Functional screenshot tool that replaces Greenshot basics.

- [ ] Tauri 2 project scaffolding
- [ ] Platform detection (Wayland GNOME/KDE, X11, Windows)
- [ ] Screenshot capture via xcap + xdg-desktop-portal
- [ ] Canvas engine with zoom, pan, triple-layer rendering
- [ ] Core annotation tools: arrow, rectangle, ellipse, text, freehand
- [ ] Undo/redo with command pattern
- [ ] Selection tool: move, resize, delete annotations
- [ ] Copy to clipboard, save to file (PNG/JPG)
- [ ] Basic toolbar and keyboard shortcuts
- [ ] Light/dark theme following OS preference

### Phase 2 — Advanced Annotations

- [ ] Blur/pixelate region tool
- [ ] Step-number tool (auto-incrementing)
- [ ] Highlight tool (semi-transparent overlay)
- [ ] Crop tool with aspect ratio lock
- [ ] Color picker with recent colors + opacity
- [ ] Stroke width and font size controls
- [ ] Export annotations as JSON for reimport

### Phase 3 — AI Features

- [ ] Tesseract OCR integration (extract text from region)
- [ ] PII auto-detection and blur suggestion
- [ ] AI panel UI for displaying results
- [ ] LLM vision integration (Claude, OpenAI, Gemini)
- [ ] Ollama local model support
- [ ] API key management via OS keychain
- [ ] Default prompt templates (describe, extract code, bug report)

### Phase 4 — MCP Server

- [ ] MCP server binary (`snaplens-mcp`)
- [ ] IPC bridge between MCP server and main app
- [ ] All 6 MCP tools implemented
- [ ] Resources for browsing screenshots
- [ ] Prompt templates for common workflows
- [ ] Documentation for MCP host configuration
- [ ] Standalone mode (capture without UI for headless/CI use)

### Phase 5 — Polish & Distribution

- [ ] Flatpak packaging and Flathub submission
- [ ] Windows MSI installer
- [ ] AppImage build
- [ ] System tray with quick capture
- [ ] Global shortcuts configuration
- [ ] Auto-update mechanism
- [ ] User onboarding / first-run experience
- [ ] Performance optimization (large screenshots, many annotations)

---

## 14. Design Guidelines

### Visual Style

- Follow platform conventions: respect system theme (light/dark)
- Toolbar: compact icon-only by default, tooltips on hover
- Canvas: checkerboard background behind transparent areas
- AI panel: collapsible sidebar, doesn't block annotation workspace
- Minimal chrome: the screenshot and annotations are the focus

### CSS Architecture

```css
/* themes.css */
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f5f5f5;
  --bg-toolbar: #e8e8e8;
  --text-primary: #1a1a1a;
  --text-secondary: #666666;
  --border: #d0d0d0;
  --accent: #2563eb;
  --accent-hover: #1d4ed8;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #1e1e1e;
    --bg-secondary: #2d2d2d;
    --bg-toolbar: #333333;
    --text-primary: #e0e0e0;
    --text-secondary: #999999;
    --border: #444444;
    --accent: #3b82f6;
    --accent-hover: #60a5fa;
  }
}
```

No CSS frameworks. Use CSS Grid for layout, CSS custom properties for theming, and minimal utility classes written by hand.

---

## 15. Security Considerations

- **API keys** stored in OS keychain only, never in config files or localStorage
- **MCP server** validates all file paths against path traversal
- **Screenshot storage** uses a sandboxed app-specific directory
- **Network calls** only to configured LLM API endpoints
- **CSP** restricts frontend to self-origin only
- **No eval()**, no inline scripts
- **Tauri permission model** restricts which commands the frontend can invoke
- **Flatpak sandbox** limits filesystem and D-Bus access

---

## 16. Open Questions for Development

1. **Region selection on X11/Windows:** Implement as a transparent fullscreen Tauri overlay window, or use xcap's built-in region selection? Overlay gives more control (crosshair cursor, magnifier, dimension display).

2. **Annotation persistence:** Should annotation layers be saved alongside images (as sidecar JSON files), embedded in PNG metadata, or saved as a custom `.snaplens` project file?

3. **Plugin system:** Should annotation tools be pluggable (dynamic loading of tool modules), or is a fixed set sufficient for v1?

4. **Video recording:** xcap has WIP video recording support. Should we plan for short GIF/video capture from the start, or defer entirely?

5. **Multi-language OCR:** Bundle only English by default and download additional languages on demand, or let users install language packs via settings?

6. **MCP image transport:** For large screenshots, should the MCP server return base64 inline (simple but large), or use a file URI and let the host read the file (more efficient but requires filesystem access)?
