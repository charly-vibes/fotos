# Screenshot Capture

## Purpose

Capability spec for the screenshot capture layer of Fotos (`io.github.charly.fotos`). Covers platform detection, capture backends, capture modes, result handling, and monitor enumeration.

## Requirements

### Requirement: Runtime Platform Detection

The system SHALL detect the current platform and compositor at runtime by inspecting environment variables (`XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`, `WAYLAND_DISPLAY`) and OS identity. The detection MUST resolve to exactly one of the following variants:

- `LinuxWaylandGnome` -- Wayland session with GNOME desktop
- `LinuxWaylandKde` -- Wayland session with KDE Plasma desktop
- `LinuxWaylandOther` -- Wayland session with another compositor (Hyprland, Sway, etc.)
- `LinuxX11` -- X11 session on Linux
- `Windows` -- Windows 10/11

#### Scenario: GNOME Wayland detected
- **WHEN** `XDG_SESSION_TYPE` is `wayland` and `XDG_CURRENT_DESKTOP` contains `GNOME`
- **THEN** the platform MUST resolve to `LinuxWaylandGnome`

#### Scenario: KDE Wayland detected
- **WHEN** `XDG_SESSION_TYPE` is `wayland` and `XDG_CURRENT_DESKTOP` contains `KDE`
- **THEN** the platform MUST resolve to `LinuxWaylandKde`

#### Scenario: Other Wayland compositor detected
- **WHEN** `XDG_SESSION_TYPE` is `wayland` and `XDG_CURRENT_DESKTOP` is neither `GNOME` nor `KDE`
- **THEN** the platform MUST resolve to `LinuxWaylandOther`

#### Scenario: X11 session detected
- **WHEN** `XDG_SESSION_TYPE` is `x11` or `WAYLAND_DISPLAY` is unset on Linux
- **THEN** the platform MUST resolve to `LinuxX11`

#### Scenario: Windows detected
- **WHEN** the operating system is Windows
- **THEN** the platform MUST resolve to `Windows`

### Requirement: Backend Selection from Platform

The system SHALL select the appropriate capture backend based on the detected platform. All Wayland variants (`LinuxWaylandGnome`, `LinuxWaylandKde`, `LinuxWaylandOther`) MUST use the xdg-desktop-portal backend. `LinuxX11` and `Windows` MUST use the xcap backend.

#### Scenario: Wayland platform selects portal backend
- **WHEN** the detected platform is any Wayland variant
- **THEN** the system MUST use the xdg-desktop-portal (ashpd) capture backend

#### Scenario: X11 platform selects xcap backend
- **WHEN** the detected platform is `LinuxX11`
- **THEN** the system MUST use the xcap capture backend

#### Scenario: Windows platform selects xcap backend
- **WHEN** the detected platform is `Windows`
- **THEN** the system MUST use the xcap capture backend

---

### Requirement: xcap Backend for X11 and Windows

The system SHALL use the `xcap` crate (version 0.8+) to perform screenshot capture on X11 and Windows platforms. The xcap backend MUST support fullscreen, monitor, region, and window capture modes.

#### Scenario: xcap captures fullscreen on X11
- **WHEN** the platform is `LinuxX11` and a fullscreen capture is requested
- **THEN** the xcap backend MUST composite all monitors into a single image and return it as a `CaptureResult`

#### Scenario: xcap captures a specific window on Windows
- **WHEN** the platform is `Windows` and a window capture is requested with a valid `WindowId`
- **THEN** the xcap backend MUST capture only that window's content and return it as a `CaptureResult`

### Requirement: xdg-desktop-portal Backend for Wayland

The system SHALL use the `ashpd` crate (version 0.11+) to perform screenshot capture on Wayland platforms via the xdg-desktop-portal Screenshot interface. The portal backend MUST communicate over D-Bus using `zbus`.

#### Scenario: Portal backend invokes ashpd on Wayland
- **WHEN** a capture is requested on a Wayland platform
- **THEN** the system MUST call `ashpd::desktop::screenshot::Screenshot` to initiate the capture through the portal

#### Scenario: Portal backend falls back gracefully when portal is unavailable
- **WHEN** the xdg-desktop-portal service is not running or does not support the Screenshot interface
- **THEN** the system MUST return a descriptive error indicating that the portal is unavailable

### Requirement: CaptureBackend Trait

All capture backends MUST implement the `CaptureBackend` async trait, which defines:

- `capture(mode: CaptureMode) -> Result<CaptureResult>` -- perform a screenshot capture
- `list_monitors() -> Result<Vec<MonitorInfo>>` -- enumerate available monitors
- `list_windows() -> Result<Vec<WindowInfo>>` -- enumerate visible windows

#### Scenario: Backend implements full trait contract
- **WHEN** a new capture backend is registered
- **THEN** it MUST implement all three methods of the `CaptureBackend` trait

---

### Requirement: Fullscreen Capture Mode

The system SHALL support a `Fullscreen` capture mode that composites all connected monitors into a single image.

#### Scenario: Fullscreen capture across multiple monitors
- **WHEN** a fullscreen capture is requested and the system has two or more monitors
- **THEN** the result image MUST contain the content of all monitors composited together

#### Scenario: Fullscreen capture with a single monitor
- **WHEN** a fullscreen capture is requested and the system has exactly one monitor
- **THEN** the result image MUST contain the full content of that monitor

### Requirement: Monitor Capture Mode

The system SHALL support a `Monitor(u32)` capture mode that captures a single monitor identified by its index.

#### Scenario: Valid monitor index
- **WHEN** a monitor capture is requested with an index that corresponds to an existing monitor
- **THEN** the system MUST capture only that monitor and return it as a `CaptureResult`

#### Scenario: Invalid monitor index
- **WHEN** a monitor capture is requested with an index that does not correspond to any connected monitor
- **THEN** the system MUST return an error indicating an invalid monitor index

### Requirement: Region Capture Mode

The system SHALL support a `Region(Rect)` capture mode that captures a user-selected rectangular area of the screen.

#### Scenario: Region capture on X11 or Windows
- **WHEN** a region capture is requested on X11 or Windows
- **THEN** the frontend MUST display a transparent overlay for the user to draw a selection rectangle, and the backend MUST crop the captured image to the selected region

#### Scenario: Region capture on Wayland
- **WHEN** a region capture is requested on a Wayland platform
- **THEN** the system MUST invoke the portal with `interactive(true)` so the compositor's native area selector is shown to the user

### Requirement: Window Capture Mode

The system SHALL support a `Window(WindowId)` capture mode that captures a single application window identified by its window ID.

#### Scenario: Valid window capture
- **WHEN** a window capture is requested with a valid `WindowId` for a visible window
- **THEN** the system MUST capture only that window's content and return it as a `CaptureResult`

#### Scenario: Invalid or closed window
- **WHEN** a window capture is requested with a `WindowId` that no longer corresponds to a visible window
- **THEN** the system MUST return an error indicating the window was not found

---

### Requirement: UUID Assignment

Every successful capture MUST be assigned a unique identifier using UUID v4 (via the `uuid` crate). The ID SHALL be assigned immediately upon successful image acquisition, before any post-processing.

#### Scenario: UUID assigned on capture
- **WHEN** a screenshot is successfully captured by any backend
- **THEN** the resulting `CaptureResult` MUST contain a non-nil UUID v4 in its `id` field

#### Scenario: UUIDs are unique across captures
- **WHEN** two screenshots are captured in the same session
- **THEN** each `CaptureResult` MUST have a distinct `id` value

### Requirement: Capture Metadata

Every `CaptureResult` MUST include a `CaptureMetadata` struct containing:

- `timestamp` -- a `DateTime<Utc>` recording when the capture occurred
- `mode` -- the `CaptureMode` that was used
- `monitor` -- an `Option<String>` with the monitor name (populated for monitor and fullscreen modes)
- `window_title` -- an `Option<String>` with the captured window's title (populated for window mode)
- `dimensions` -- a `(u32, u32)` tuple of `(width, height)` in pixels

#### Scenario: Metadata populated for fullscreen capture
- **WHEN** a fullscreen capture completes
- **THEN** `metadata.mode` MUST be `Fullscreen`, `metadata.dimensions` MUST match the composited image size, and `metadata.timestamp` MUST reflect the time of capture

#### Scenario: Metadata populated for window capture
- **WHEN** a window capture completes for a window titled "Firefox"
- **THEN** `metadata.window_title` MUST be `Some("Firefox")` and `metadata.mode` MUST be `Window(..)`

#### Scenario: Metadata populated for region capture
- **WHEN** a region capture completes for a 400x300 rectangle
- **THEN** `metadata.dimensions` MUST be `(400, 300)` and `metadata.mode` MUST be `Region(..)`

### Requirement: In-Memory Image Storage

The captured image MUST be held in memory as a `DynamicImage` (from the `image` crate) within the `CaptureResult`. The system SHALL NOT write the capture to disk unless explicitly requested by a save or export operation.

#### Scenario: Image available in memory after capture
- **WHEN** a capture completes successfully
- **THEN** `CaptureResult.image` MUST contain a valid `DynamicImage` that can be encoded to PNG, JPEG, or WebP

---

### Requirement: Interactive Portal Screenshot

On Wayland platforms, the system SHALL request an interactive screenshot through the xdg-desktop-portal by calling `ashpd::desktop::screenshot::Screenshot::request()` with `interactive(true)`. This delegates region/area selection to the compositor's native overlay (GNOME Shell, KDE Plasma, etc.).

#### Scenario: GNOME interactive screenshot flow
- **WHEN** a capture is initiated on `LinuxWaylandGnome`
- **THEN** the system MUST call the portal with `interactive(true)`, GNOME Shell MUST display its native area selector, and the user's selection MUST be respected

#### Scenario: KDE interactive screenshot flow
- **WHEN** a capture is initiated on `LinuxWaylandKde`
- **THEN** the system MUST call the portal with `interactive(true)`, and KDE Spectacle's portal integration MUST handle the selection

### Requirement: Portal Temporary File Handling

The xdg-desktop-portal returns the screenshot as a temporary file URI (e.g., `file:///tmp/screenshot-XXXX.png`). The system MUST load this file into memory as a `DynamicImage`, assign a UUID and metadata, and then clean up or release the temporary file reference.

#### Scenario: Temp file loaded into memory
- **WHEN** the portal returns a file URI for the captured screenshot
- **THEN** the system MUST read the file, decode it into a `DynamicImage`, and populate a `CaptureResult` with UUID and metadata

#### Scenario: Temp file URI is invalid
- **WHEN** the portal returns a file URI that cannot be read or decoded
- **THEN** the system MUST return an error describing the failure (file not found, decode error, or permission denied)

### Requirement: Portal Screenshot Event Emission

After loading the portal screenshot into memory, the Rust backend MUST emit a `screenshot-ready` Tauri event to the frontend. The event payload SHALL include the screenshot ID and dimensions so the frontend can request and display the image.

#### Scenario: Frontend receives screenshot-ready event
- **WHEN** a portal screenshot is successfully loaded into memory
- **THEN** the backend MUST emit a `screenshot-ready` event containing the `CaptureResult.id` and `metadata.dimensions`

---

### Requirement: xcap Direct Capture

On X11 and Windows, the system SHALL use xcap's `Monitor::all()` or `Window::all()` APIs to enumerate targets, then capture the selected target directly into an image buffer. The buffer MUST be converted to a `DynamicImage` and wrapped in a `CaptureResult`.

#### Scenario: Full capture on X11 via xcap
- **WHEN** a fullscreen capture is requested on `LinuxX11`
- **THEN** the system MUST call `xcap::Monitor::all()`, capture each monitor, composite them, assign a UUID, and populate metadata

#### Scenario: Window capture on Windows via xcap
- **WHEN** a window capture is requested on `Windows` with a specific `WindowId`
- **THEN** the system MUST locate the window via xcap, capture its content, and return a `CaptureResult`

### Requirement: Region Selection Overlay for X11/Windows

For region capture on X11 and Windows, the frontend MUST display a transparent fullscreen overlay window that allows the user to draw a selection rectangle. The selected rectangle coordinates MUST be sent to the Rust backend, which crops the captured image to that region.

#### Scenario: User draws selection rectangle
- **WHEN** a region capture is initiated on X11 or Windows
- **THEN** a transparent fullscreen overlay MUST appear, the user MUST be able to click and drag to define a rectangle, and upon release the selected coordinates MUST be sent to the backend

#### Scenario: User cancels region selection
- **WHEN** the user presses Escape during region selection on the overlay
- **THEN** the overlay MUST close and the capture MUST be cancelled without producing a result

### Requirement: X11/Windows Screenshot Event Emission

After capturing via xcap, the Rust backend MUST emit a `screenshot-ready` Tauri event to the frontend with the screenshot ID and dimensions, consistent with the Wayland portal flow.

#### Scenario: Frontend receives screenshot-ready event from xcap
- **WHEN** an xcap capture completes successfully
- **THEN** the backend MUST emit a `screenshot-ready` event with the same payload shape as the portal flow

---

### Requirement: List Available Monitors

The system SHALL provide a `list_monitors()` function that returns a `Vec<MonitorInfo>` describing all connected monitors. Each `MonitorInfo` MUST include:

- `index` -- zero-based monitor index (`u32`)
- `name` -- human-readable monitor name or model identifier (`String`)
- `width` -- horizontal resolution in pixels (`u32`)
- `height` -- vertical resolution in pixels (`u32`)
- `x` -- horizontal position in the virtual desktop (`i32`)
- `y` -- vertical position in the virtual desktop (`i32`)
- `is_primary` -- whether this is the primary monitor (`bool`)

#### Scenario: Single monitor enumeration
- **WHEN** `list_monitors()` is called on a system with one connected display
- **THEN** the result MUST contain exactly one `MonitorInfo` entry with `index` of `0` and `is_primary` set to `true`

#### Scenario: Multi-monitor enumeration
- **WHEN** `list_monitors()` is called on a system with three connected displays
- **THEN** the result MUST contain three `MonitorInfo` entries, each with a unique `index` from `0` to `2`, and exactly one entry MUST have `is_primary` set to `true`

### Requirement: Monitor Enumeration on Wayland

On Wayland platforms where direct monitor enumeration may be restricted, the system SHALL use the xdg-desktop-portal or compositor-specific APIs to obtain monitor information. If enumeration is not supported, the system MUST return at least one monitor entry representing the default display.

#### Scenario: Wayland monitor enumeration via portal
- **WHEN** `list_monitors()` is called on a Wayland platform that supports monitor enumeration through the portal
- **THEN** the result MUST contain accurate monitor metadata matching the connected displays

#### Scenario: Wayland fallback for restricted enumeration
- **WHEN** `list_monitors()` is called on a Wayland platform that does not expose monitor details
- **THEN** the result MUST contain at least one `MonitorInfo` entry with best-effort metadata (e.g., dimensions from the primary display)
