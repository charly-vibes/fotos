## Flatpak IPC + Screenshot Debugging — 2026-02-24

### Symptoms
- App opened in Flatpak but all screenshot/action buttons appeared dead
- Annotation tool buttons (pure-JS, no IPC) worked fine
- No error visible in the status bar

### Root Causes Found (in order of discovery)

#### 1. Missing `connect-src` in CSP (tauri.conf.json)
The `security.csp` field overrides Tauri 2's default CSP entirely.
Tauri 2's default includes `connect-src ipc: http://ipc.localhost` which is
required for `invoke()` calls. Without it, all IPC is blocked by the
browser sandbox in production builds — silently, with no error thrown.

**Fix:** Add `connect-src ipc: http://ipc.localhost` to the csp string.

In dev mode (`cargo tauri dev`) this doesn't manifest because Tauri injects
the IPC headers at the transport level, bypassing the custom CSP.

#### 2. CSP fix silently excluded from Flatpak builds
The Flatpak manifest sources the app via `type: git, path: .., branch: main`.
flatpak-builder reads the **committed** git state, not the working tree.
The fix was in the working directory but not committed, so every rebuild
used the old `tauri.conf.json`.

**Fix:** Added a guard in `scripts/build-flatpak.sh` that aborts if there
are uncommitted changes in source directories (src-tauri, src-ui, src-mcp, flatpak).
Initial version was too broad (caught .beads, .wai, untracked files); narrowed
to only the directories that matter to the build.

#### 3. No logging infrastructure
`tracing` and `tracing-subscriber` were in Cargo.toml but
`tracing_subscriber::fmt().init()` was never called. Zero stderr output.

**Fix:** Added `init_logging()` in `run()` using `EnvFilter` (reads `RUST_LOG`).
Default level is `info`. Run with `RUST_LOG=debug` for full verbosity.
Also required adding the `env-filter` feature to `tracing-subscriber`.

#### 4. Global shortcuts used wrong capture backend in Flatpak
`do_capture_and_emit()` (Ctrl+Shift+S / Ctrl+Shift+A global shortcuts) called
`xcap_backend::capture_fullscreen()` unconditionally. xcap uses direct display
server access which is blocked in the Flatpak sandbox.
The `take_screenshot` IPC command already had the correct `FLATPAK_ID` check
to route through the XDG portal, but the global shortcut path did not.

**Fix:** Added `FLATPAK_ID` check in `do_capture_and_emit()` to use
`capture::portal::capture_via_portal()` when running in Flatpak.

### Key Tauri 2 + Flatpak Rules Learned
- Custom `security.csp` fully overrides Tauri defaults — must include
  `connect-src ipc: http://ipc.localhost` manually.
- `type: git` Flatpak sources read committed state only; changes must be
  committed before `just install` will pick them up.
- After changing Cargo.toml features, run `just gen-cargo-sources` to
  update `flatpak/cargo-sources.json`, then commit it before building.
- Adding `env-filter` feature to `tracing-subscriber` is required for
  `EnvFilter` and `with_env_filter()`.

### Commits
- 329dff2 Fix Flatpak IPC: add connect-src to CSP, guard uncommitted builds
- f992dde Fix build-flatpak guard: only check source directories
- 15e3040 Add tracing, fix global shortcuts to use portal in Flatpak
- fbf49f8 Enable env-filter feature for tracing-subscriber
- 65d77d2 Regenerate cargo-sources.json for env-filter feature
