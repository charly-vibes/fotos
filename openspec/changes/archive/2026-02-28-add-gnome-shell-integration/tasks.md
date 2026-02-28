## 1. Rust D-Bus Service

- [x] 1.1 Verify `zbus` has `tokio` feature in `src-tauri/Cargo.toml`; add `features = ["tokio"]` if absent (zbus 5.x requires this for async/await on Tokio)
- [x] 1.2 Create `src-tauri/src/dbus.rs` with `FotosService` struct and `#[interface]` impl
  - `Activate()` → `window.show()` + `window.set_focus()`
  - `TakeScreenshot(mode)` → check `AtomicBool` capture guard first (return `Failed` if busy), then emit `"global-capture-region"` or `"global-capture-fullscreen"` Tauri event; return `InvalidArgs` for unknown modes
  - `Version` property → `env!("CARGO_PKG_VERSION")`
  - `start_service(app_handle)` async fn using zbus 5.x API: `zbus::connection::Builder::session()?.name("io.github.charly.Fotos")?.serve_at("/io/github/charly/Fotos", service)?.build().await?`
- [x] 1.3 In `src-tauri/src/lib.rs`: add `#[cfg(target_os = "linux")] mod dbus;` declaration, then spawn `dbus::start_service` in the setup closure (non-fatal on error — log warning and continue)

## 2. Flatpak Permission

- [x] 2.1 Add `- --own-name=io.github.charly.Fotos` to `finish-args` in `flatpak/io.github.charly.fotos.yml`

## 3. GNOME Extension — Structure and Metadata

- [x] 3.1 Create `gnome-extension/metadata.json` (uuid, name, description, shell-version 45–48, settings-schema)
- [x] 3.2 Create `gnome-extension/schemas/org.gnome.shell.extensions.fotos.gschema.xml` with `capture-region-shortcut` and `capture-fullscreen-shortcut` keys

## 4. GNOME Extension — Core Logic

- [x] 4.1 Create `gnome-extension/extension.js` with `export default class FotosExtension extends Extension`
- [x] 4.2 Define `FOTOS_IFACE_XML` as an inline const string in `extension.js` (extensions cannot read files at runtime); create `Gio.DBusProxy.makeProxyWrapper(FOTOS_IFACE_XML)` proxy class. All proxy method calls MUST use the async/callback form (e.g. `proxy.ActivateRemote(callback)`) — never the `*Sync` variants which block the Shell main loop
- [x] 4.3 Implement `NameOwnerChanged` listener to track Fotos running state and update menu item sensitivity
- [x] 4.4 Implement `PanelMenu.Button` with popup menu: "Open Fotos", separator, "Capture Region", "Capture Fullscreen"
- [x] 4.5 Implement `_launchAndThen(callback)` helper: if Fotos is on bus → call immediately; else → `Gio.DesktopAppInfo.new('io.github.charly.fotos.desktop').launch()` + poll for bus name (5s timeout), then call callback
- [x] 4.6 Implement `enable()`: create proxy, add panel button, register keybindings via `Main.wm.addKeybinding()`
- [x] 4.7 Implement `disable()`: remove panel button, call `Main.wm.removeKeybinding()` for both shortcuts, destroy proxy

## 5. Justfile Recipes

- [x] 5.1 Add `gnome-schema` recipe: `glib-compile-schemas gnome-extension/schemas/`
- [x] 5.2 Add `gnome-install` recipe: depends on `gnome-schema`, copies to `~/.local/share/gnome-shell/extensions/fotos@io.github.charly/`
- [x] 5.3 Add `gnome-pack` recipe: depends on `gnome-schema`, zips `gnome-extension/` to `fotos-gnome-extension.zip`

## 6. Validation

- [x] 6.1 `just check` — Rust compiles with new `dbus.rs`
- [x] 6.2 `just lint` — no Clippy warnings
- [x] 6.3 `just test` — existing tests pass
- [ ] 6.4 **[manual]** D-Bus smoke test: run Fotos, then `gdbus call --session --dest io.github.charly.Fotos --object-path /io/github/charly/Fotos --method io.github.charly.Fotos.Activate` — window should appear/focus
- [ ] 6.5 **[manual]** `just gnome-install` — extension deploys to user profile
- [ ] 6.6 **[manual]** Enable extension, run Fotos, confirm panel indicator appears and menu actions work
- [ ] 6.7 **[manual]** Press Ctrl+Shift+S with Fotos running → region capture triggers (note: save-as shortcut is shadowed while extension is enabled — expected)
- [ ] 6.8 **[manual]** Click "Open Fotos" with Fotos not running → Fotos launches and window appears
- [x] 6.9 `just spec-validate` — all specs pass
