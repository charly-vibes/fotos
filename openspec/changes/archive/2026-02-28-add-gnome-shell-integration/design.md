# Design: GNOME Shell Extension Integration

## Context

Fotos is cross-platform (Linux Flatpak, Windows MSI/NSIS, macOS). The GNOME integration must be
additive — a Linux-only enhancement that degrades gracefully to no-op on other platforms. The
constraint is: **do not couple the Tauri app to GNOME** — the extension is the GNOME-specific
component, not Fotos itself.

## Goals / Non-Goals

**Goals:**
- Panel indicator with capture actions accessible from the GNOME top bar
- GNOME-native global shortcuts (work even when Fotos window is not focused, via GNOME Shell)
- One-click cold launch: extension spawns Fotos if not running
- No user configuration required beyond installing + enabling the extension

**Non-Goals:**
- Replacing the Tauri app with a GNOME extension (Option C)
- Annotation UI in GNOME Shell / Clutter
- GNOME Quick Settings integration (overkill for a screenshot tool)
- KDE Plasma panel integration (separate effort)
- Automatic extension installation (user installs via GNOME Extension Manager or EGO)

## Architecture

```
GNOME Shell process (GJS)         Session D-Bus
──────────────────────────        ─────────────
FotosExtension                    io.github.charly.Fotos
  PanelMenu.Button ──────────────→ Activate()
  menu item "Region" ────────────→ TakeScreenshot("region")
  menu item "Fullscreen" ─────────→ TakeScreenshot("fullscreen")
  Ctrl+Shift+S keybinding ────────→ TakeScreenshot("region")
  Ctrl+Shift+A keybinding ────────→ TakeScreenshot("fullscreen")

  NameOwnerChanged listener ──────→ enable/disable menu items
  Gio.DesktopAppInfo.launch() ────→ spawn Fotos when not running

Fotos process (Tauri/Rust)
────────────────────────────
dbus.rs (Linux only)
  io.github.charly.Fotos service
    Activate() ─────────────────→ window.show() + set_focus()
    TakeScreenshot(mode) ───────→ app.emit("global-capture-{mode}", ())
    Version property ───────────→ env!("CARGO_PKG_VERSION")
```

## Decisions

### Decision: D-Bus over Unix socket for extension communication

The extension runs inside GNOME Shell's process (GJS). GJS has first-class D-Bus support via
`Gio.DBusProxy` — well-documented, with native service-presence detection via `NameOwnerChanged`.
The existing Unix socket IPC is designed for the fotos-mcp binary (length-framed JSON), not for
GJS callers. D-Bus is the correct IPC mechanism here.

Alternatives considered:
- **Unix socket from GJS**: Possible via `Gio.UnixSocketAddress` but requires manual
  protocol framing, no built-in service presence detection, awkward error handling in GJS.
- **HTTP REST**: Would require running an HTTP server in Fotos — overkill, security surface.

**Critical implementation note — async-only D-Bus calls**: GNOME Shell extensions run on the
main loop. All D-Bus proxy calls MUST use the async/callback pattern to avoid blocking the
compositor (which would freeze the entire desktop). Use `Gio.DBusProxy.makeProxyWrapper` and
call the generated methods with a trailing callback argument — never call the `*Sync` variants.
The interface introspection XML MUST be defined as an inline constant string in `extension.js`
(GNOME extensions cannot read files at runtime).

Example correct pattern:
```javascript
const FotosProxy = Gio.DBusProxy.makeProxyWrapper(FOTOS_IFACE_XML); // XML inline const
const proxy = new FotosProxy(Gio.DBus.session, 'io.github.charly.Fotos', '/io/github/charly/Fotos');
// Async call — OK
proxy.TakeScreenshotRemote('region', (_proxy, error) => { if (error) log(error); });
// NEVER: proxy.TakeScreenshotSync('region')  ← blocks Shell main loop
```

### Decision: zbus for Rust D-Bus service

`zbus` is already in `Cargo.toml` (unused). It supports async/await with Tokio, has a clean
`#[interface]` macro, and handles connection lifecycle. No new dependency.

### Decision: Service startup is non-fatal

The D-Bus daemon may not be running (e.g., non-desktop environments, CI). Service start failure
is logged as a warning and ignored — Fotos continues to function normally without the extension.

### Decision: Extension in-repo under `gnome-extension/`

Keeps the extension versioned alongside the app. Users install it separately via GNOME Extension
Manager (or EGO), but the source lives here. Alternative (separate repo) adds maintenance
overhead with no clear benefit at this stage.

### Decision: GSettings schema for keybindings

GNOME Shell's `Main.wm.addKeybinding()` requires keybindings to be backed by a GSettings schema.
This is non-negotiable — it's how GNOME manages shortcut conflicts and allows users to change
bindings via the Keyboard settings panel. The schema compiles to a binary (`gschemas.compiled`)
that must ship alongside the extension JS files.

### Decision: Target GNOME Shell 45–48

GNOME 45 introduced the ES module extension format (`export default class ... extends Extension`).
Targeting 45+ avoids maintaining a legacy compatibility shim. GNOME 47 and 48 are current LTS
releases on Fedora and Ubuntu respectively.

## Risks / Trade-offs

- **Shortcut conflict — two layers**: On Wayland, GNOME Shell global keybindings fire before
  the app window receives the key event. This means:
  1. `Ctrl+Shift+S` (extension) vs Tauri's global shortcut plugin: both may register; GNOME
     Shell takes priority, extension fires, Tauri registration either silently fails or is a
     no-op. The `AtomicBool` capture mutex prevents double-capture if both fire.
  2. `Ctrl+Shift+S` (extension) vs in-app save-as (`ui-shell` spec): the extension intercepts
     the key even when Fotos is focused, permanently shadowing the in-app save-as action. Users
     must remap `capture-region-shortcut` in GSettings to restore save-as. Future fix: only
     activate the shortcut when the Fotos window is NOT focused (deferred).

- **Flatpak D-Bus**: Extensions run outside Flatpak; Fotos runs inside. Flatpak apps can expose
  D-Bus services on the session bus (the `--own-name` permission grants this). This is the
  standard pattern for Flatpak ↔ desktop integration.

- **Extension review lag**: Extensions.gnome.org has a review queue. Distribution is manual
  until reviewed. Users can install from zip in the meantime.

## Open Questions

- Should the extension suppress Tauri's global shortcuts on GNOME to avoid the double-registration?
  (Deferred — requires Fotos detecting GNOME Shell extension presence, adds complexity.)
- Should the extension be submitted to extensions.gnome.org? (Out of scope for this change.)
