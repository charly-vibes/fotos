Flatpak build pipeline: lessons learned

Getting just install working required fixing 11 issues in sequence.
Captured here so future sessions don't repeat the debugging.

## Runtime setup
- GNOME SDK 47 was not installed on the host; bumped manifest to SDK 48
  (org.gnome.Platform//48 and org.gnome.Sdk//48 were available)
- flathub remote is system-only on Bluefin; setup-flatpak must use
  --system, not --user
- LLVM 19 SDK extension (org.freedesktop.Sdk.Extension.llvm19//24.08)
  is required for bindgen/libclang

## Offline cargo build
- tesseract-rs is unusable without its build-tesseract default feature
  (the entire public API is gated behind it); switched to the tesseract
  crate which uses system libs via pkg-config
- flatpak-cargo-generator.py (from flatpak-builder-tools) generates
  cargo-sources.json from Cargo.lock; needs tomlkit + aiohttp + attrs
  in a venv; added just gen-cargo-sources recipe
- Must commit Cargo.toml changes before just install — the fotos module
  uses type: git so it checks out the branch, not the working tree
- Cargo.lock is gitignored; cargo resolves from vendor dir at build time

## C library modules (leptonica + tesseract)
- cmake-ninja buildsystem does in-source builds by default; both need
  builddir: true (tesseract's CMakeLists.txt explicitly rejects in-source)
- leptonica cmake generates lept_Release.pc not lept.pc; fixed with a
  post-install symlink
- Both libs need -DCMAKE_POSITION_INDEPENDENT_CODE=ON (Tauri builds fotos
  as a cdylib, which requires PIC from all static deps)
- Static leptonica has transitive deps (libpng etc.) not reflected in its
  linker flags; switched both to -DBUILD_SHARED_LIBS=ON to let the dynamic
  linker handle this
- tesseract-sys needs PKG_CONFIG_PATH=/app/lib/pkgconfig to find lept.pc

## Tauri sidecar
- build.rs checks for fotos-mcp-x86_64-unknown-linux-gnu at compile time;
  must touch this file before cargo build in the flatpak build-commands

## Installing from local repo
- flatpak install with file:// URIs is unreliable; correct approach:
  1. flatpak build-update-repo .flatpak-repo  (generates summary file)
  2. flatpak remote-add --no-gpg-verify fotos-local .flatpak-repo
  3. flatpak install fotos-local io.github.charly.fotos
- build-flatpak.sh was running flatpak-builder twice redundantly; fixed
  to single run with --repo flag
