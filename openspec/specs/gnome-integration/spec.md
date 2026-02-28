# gnome-integration Specification

## Purpose
TBD - created by archiving change add-gnome-shell-integration. Update Purpose after archive.
## Requirements
### Requirement: D-Bus Service

On Linux, Fotos SHALL expose a D-Bus service on the session bus under the well-known name
`io.github.charly.Fotos` at object path `/io/github/charly/Fotos` with interface
`io.github.charly.Fotos`. The service SHALL be started during app initialization and SHALL fail
non-fatally (log a warning and continue) if the session bus is unavailable.

The interface SHALL expose:
- Method `Activate()` — raise and focus the main Fotos window
- Method `TakeScreenshot(mode: String)` — trigger a capture; `mode` MUST be one of `"region"` or `"fullscreen"`
- Read-only property `Version: String` — the current app version string

#### Scenario: Activate raises the window
- **WHEN** the D-Bus client calls `Activate()`
- **THEN** the main Fotos window MUST become visible
- **THEN** the main Fotos window MUST receive input focus

#### Scenario: TakeScreenshot triggers region capture
- **WHEN** the D-Bus client calls `TakeScreenshot("region")`
- **THEN** the app MUST initiate a region capture (equivalent to pressing the capture-region toolbar button)

#### Scenario: TakeScreenshot triggers fullscreen capture
- **WHEN** the D-Bus client calls `TakeScreenshot("fullscreen")`
- **THEN** the app MUST initiate a fullscreen capture (equivalent to pressing the capture-fullscreen toolbar button)

#### Scenario: TakeScreenshot rejects unknown mode
- **WHEN** the D-Bus client calls `TakeScreenshot` with a value other than `"region"` or `"fullscreen"`
- **THEN** the method MUST return a D-Bus error `org.freedesktop.DBus.Error.InvalidArgs`

#### Scenario: Version property returns app version
- **WHEN** the D-Bus client reads the `Version` property
- **THEN** the returned string MUST equal the running app's semantic version (e.g., `"0.3.0"`)

#### Scenario: TakeScreenshot while capture is in progress
- **WHEN** the D-Bus client calls `TakeScreenshot` while a capture is already in progress
- **THEN** the method MUST return a D-Bus error `org.freedesktop.DBus.Error.Failed` with message `"capture in progress"`
- **THEN** the ongoing capture MUST NOT be interrupted

#### Scenario: Service unavailable does not crash app
- **WHEN** the D-Bus session bus is unavailable at app startup
- **THEN** Fotos MUST start normally and log a warning
- **THEN** all other app features MUST function without the D-Bus service

---

### Requirement: GNOME Shell Extension Panel Indicator

The GNOME Shell extension (`gnome-extension/`) SHALL add a `PanelMenu.Button` indicator to
the GNOME top panel (right box) whenever the extension is enabled. The indicator SHALL use the
`camera-photo-symbolic` icon. The popup menu SHALL contain:

1. "Open Fotos" — activates or launches the Fotos window
2. Separator
3. "Capture Region" — triggers a region capture
4. "Capture Fullscreen" — triggers a fullscreen capture

"Capture Region" and "Capture Fullscreen" SHALL be insensitive (greyed out) when the Fotos
D-Bus service is not present on the session bus.

#### Scenario: Indicator appears in GNOME panel
- **WHEN** the extension is enabled in GNOME Shell
- **THEN** a camera icon MUST appear in the GNOME top panel right box

#### Scenario: Clicking Open Fotos when running
- **WHEN** the Fotos D-Bus service is present and the user clicks "Open Fotos"
- **THEN** the extension MUST call `Activate()` on the D-Bus service
- **THEN** the Fotos window MUST become visible and focused

#### Scenario: Clicking Open Fotos when not running
- **WHEN** the Fotos D-Bus service is NOT present and the user clicks "Open Fotos"
- **THEN** the extension MUST launch Fotos via `Gio.DesktopAppInfo` using the `io.github.charly.fotos.desktop` entry
- **THEN** after Fotos registers on the bus (within a reasonable startup timeout), the extension MUST call `Activate()`

#### Scenario: Capture menu items disabled when Fotos is not running
- **WHEN** the Fotos D-Bus service is NOT present on the session bus
- **THEN** "Capture Region" and "Capture Fullscreen" menu items MUST be insensitive

#### Scenario: Capture menu items enabled when Fotos is running
- **WHEN** the Fotos D-Bus service IS present on the session bus
- **THEN** "Capture Region" and "Capture Fullscreen" menu items MUST be sensitive

#### Scenario: Clicking Capture Region
- **WHEN** the user clicks "Capture Region" in the panel menu
- **THEN** the extension MUST call `TakeScreenshot("region")` on the D-Bus service

#### Scenario: Clicking Capture Fullscreen
- **WHEN** the user clicks "Capture Fullscreen" in the panel menu
- **THEN** the extension MUST call `TakeScreenshot("fullscreen")` on the D-Bus service

---

### Requirement: GNOME-Native Global Keybindings

The GNOME Shell extension SHALL register two global keybindings via `Main.wm.addKeybinding()`
backed by a GSettings schema (`org.gnome.shell.extensions.fotos`):

- `capture-region-shortcut` — default `['<Control><Shift>s']` — triggers region capture
- `capture-fullscreen-shortcut` — default `['<Control><Shift>a']` — triggers fullscreen capture

When triggered, each binding SHALL follow the same cold-launch flow as the panel menu actions:
call the D-Bus service if Fotos is running, or launch Fotos first if not.

> **Known conflict**: `<Control><Shift>s` is also the in-app save-as shortcut defined in the
> `ui-shell` spec. On Wayland, GNOME Shell intercepts global keybindings before they reach the
> app window, so save-as will not fire while this extension is enabled. Users who need save-as
> must remap `capture-region-shortcut` in GNOME Settings → Keyboard → Custom Shortcuts.
> Resolving this conflict (e.g. by routing the shortcut through the extension only when the
> window is NOT focused) is deferred to a future change.

#### Scenario: Ctrl+Shift+S triggers region capture when Fotos is running
- **WHEN** the user presses Ctrl+Shift+S with the extension enabled and Fotos running
- **THEN** `TakeScreenshot("region")` MUST be called on the D-Bus service

#### Scenario: Ctrl+Shift+A triggers fullscreen capture when Fotos is running
- **WHEN** the user presses Ctrl+Shift+A with the extension enabled and Fotos running
- **THEN** `TakeScreenshot("fullscreen")` MUST be called on the D-Bus service

#### Scenario: Shortcut cold-launches Fotos when not running
- **WHEN** the user presses a capture shortcut and Fotos is not running
- **THEN** the extension MUST launch Fotos via `Gio.DesktopAppInfo`
- **THEN** the extension MUST call the appropriate `TakeScreenshot` method once the D-Bus service appears

#### Scenario: Keybindings removed on extension disable
- **WHEN** the user disables the extension in GNOME Shell
- **THEN** both keybindings MUST be removed via `Main.wm.removeKeybinding()`
- **THEN** pressing Ctrl+Shift+S or Ctrl+Shift+A MUST no longer trigger Fotos

---

### Requirement: Extension Metadata and Compatibility

The extension SHALL declare compatibility with GNOME Shell versions 45, 46, 47, and 48 in its
`metadata.json`. The extension UUID SHALL be `fotos@io.github.charly`. The extension MUST use
the GNOME 45+ ES module format (`export default class ... extends Extension`).

> **Compatibility note**: The extension MUST be manually tested against each declared GNOME Shell
> version (45, 46, 47, 48) before each release. There is no automated cross-version test
> infrastructure; the `metadata.json` `shell-version` list is the compatibility declaration.

#### Scenario: Extension cleans up on disable
- **WHEN** the extension is disabled (e.g., via GNOME Extensions app)
- **THEN** the panel indicator MUST be removed from the top bar
- **THEN** keybindings MUST be unregistered
- **THEN** the D-Bus proxy MUST be destroyed

---

### Requirement: Flatpak D-Bus Permission

The Fotos Flatpak manifest SHALL include `--own-name=io.github.charly.Fotos` in its `finish-args`
so that the sandboxed app is permitted to own that well-known name on the session bus.

#### Scenario: Fotos Flatpak registers D-Bus service
- **WHEN** Fotos is installed as a Flatpak and launched
- **THEN** the `io.github.charly.Fotos` service MUST appear on the session bus
- **THEN** the GNOME extension MUST be able to call `Activate()` and `TakeScreenshot()`

---

### Requirement: Justfile Build Recipes

The `justfile` SHALL include three GNOME extension recipes:

- `gnome-schema` — compile the GSettings XML schema (`glib-compile-schemas gnome-extension/schemas/`)
- `gnome-install` — run `gnome-schema` then copy `gnome-extension/` to `~/.local/share/gnome-shell/extensions/fotos@io.github.charly/`
- `gnome-pack` — run `gnome-schema` then package the extension as `fotos-gnome-extension.zip` for distribution

#### Scenario: gnome-install deploys the extension
- **WHEN** the developer runs `just gnome-install`
- **THEN** the compiled schema and extension files MUST be present under `~/.local/share/gnome-shell/extensions/fotos@io.github.charly/`
- **THEN** the extension MUST be loadable by GNOME Shell after re-enabling it in the Extensions app (X11: `Alt+F2` → `r`; Wayland: log out and back in)

