# Fotos — AI-powered screenshot capture, annotation, and analysis
# Run `just` to see available recipes.

# Distrobox name for builds (Bluefin can't install -devel packages natively)
box := "fedora"

# Default: list recipes
default:
    @just --list

# Helper: ensure the sidecar placeholder exists (Tauri build.rs requires it)
[private]
ensure-sidecar:
    #!/usr/bin/env bash
    if command -v distrobox &>/dev/null && distrobox list 2>/dev/null | grep -q "{{box}}"; then
        triple=$(distrobox enter {{box}} -- rustc -vV | grep '^host:' | cut -d' ' -f2)
    else
        triple=$(rustc -vV | grep '^host:' | cut -d' ' -f2)
    fi
    ext=""
    [[ "$triple" == *windows* ]] && ext=".exe"
    touch "src-tauri/fotos-mcp-${triple}${ext}"

# Helper: remove the sidecar placeholder
[private]
clean-sidecar:
    rm -f src-tauri/fotos-mcp-*-unknown-*

# ── Build ─────────────────────────────────────────────

# Check that both workspace crates compile (no linking)
check: ensure-sidecar
    distrobox enter {{box}} -- cargo check
    @just clean-sidecar

# Build the full app in release mode
build: ensure-sidecar
    distrobox enter {{box}} -- cargo build --release
    @just clean-sidecar

# Run the app in development mode
dev: ensure-sidecar
    distrobox enter {{box}} -- cargo tauri dev

# Build a Flatpak package
flatpak:
    ./scripts/build-flatpak.sh

# Build a distributable package (format: flatpak [default], deb, appimage, msi, nsis)
package format="flatpak": ensure-sidecar
    #!/usr/bin/env bash
    set -euo pipefail
    _build() {
        if command -v distrobox &>/dev/null && distrobox list 2>/dev/null | grep -q "{{box}}"; then
            distrobox enter {{box}} -- "$@"
        else
            "$@"
        fi
    }
    case "{{format}}" in
        flatpak)
            ./scripts/build-flatpak.sh
            ;;
        deb|appimage|msi|nsis)
            _build cargo tauri build --bundles {{format}}
            ;;
        *)
            echo "Unknown format '{{format}}'. Supported: flatpak, deb, appimage, msi, nsis" >&2
            exit 1
            ;;
    esac
    rm -f src-tauri/fotos-mcp-*-unknown-* src-tauri/fotos-mcp-*.exe 2>/dev/null || true

# Build and install Fotos locally (format: flatpak [default], deb, appimage, msi, nsis)
install format="flatpak": (package format)
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{format}}" in
        flatpak)
            flatpak remote-add --user --no-gpg-verify --if-not-exists fotos-local "$(pwd)/.flatpak-repo"
            flatpak install --user --reinstall fotos-local io.github.charly.fotos
            ;;
        deb)
            sudo dpkg -i src-tauri/target/release/bundle/deb/*.deb
            ;;
        appimage)
            mkdir -p ~/.local/bin
            cp src-tauri/target/release/bundle/appimage/*.AppImage ~/.local/bin/fotos.AppImage
            chmod +x ~/.local/bin/fotos.AppImage
            echo "Installed: ~/.local/bin/fotos.AppImage"
            ;;
        msi)
            msiexec /i "$(ls src-tauri/target/release/bundle/msi/*.msi | head -1)"
            ;;
        nsis)
            "$(ls src-tauri/target/release/bundle/nsis/*.exe | head -1)" /S
            ;;
    esac

# ── Quality ───────────────────────────────────────────

# Run clippy lints
lint: ensure-sidecar
    distrobox enter {{box}} -- cargo clippy --workspace -- -D warnings
    @just clean-sidecar

# Format all Rust code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run Rust tests
test: ensure-sidecar
    distrobox enter {{box}} -- cargo test --workspace
    @just clean-sidecar

# ── Specs ─────────────────────────────────────────────

# Validate all OpenSpec capability specs
spec-validate:
    openspec validate --all

# List all OpenSpec capabilities
spec-list:
    openspec show --all

# ── Setup ─────────────────────────────────────────────

# Install Tauri build dependencies in the fedora distrobox
setup-distrobox:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! distrobox list 2>/dev/null | grep -q {{box}}; then
        echo "Creating {{box}} distrobox..."
        distrobox create --name {{box}} --image fedora:43 --yes
    fi
    echo "Installing build dependencies..."
    distrobox enter {{box}} -- sudo dnf install -y \
        gcc gcc-c++ make cmake pkg-config \
        gtk3-devel \
        webkit2gtk4.1-devel \
        javascriptcoregtk4.1-devel \
        libsoup3-devel \
        cairo-devel \
        pango-devel \
        gdk-pixbuf2-devel \
        atk-devel \
        libappindicator-gtk3-devel \
        dbus-devel \
        pipewire-devel \
        mesa-libgbm-devel \
        clang-devel \
        openssl-devel \
        tesseract-devel \
        leptonica-devel \
        curl wget git file
    echo "Installing Rust toolchain (if needed)..."
    distrobox enter {{box}} -- bash -c "command -v cargo || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
    echo "Installing Tauri CLI..."
    distrobox enter {{box}} -- cargo install tauri-cli
    echo "Done. Run 'just check' to verify."

# Regenerate flatpak/cargo-sources.json from Cargo.lock (run after any dependency change)
gen-cargo-sources:
    #!/usr/bin/env bash
    set -euo pipefail
    SCRIPT=/tmp/flatpak-cargo-generator.py
    VENV=/tmp/flatpak-venv
    if [ ! -f "$SCRIPT" ]; then
        curl -sL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py -o "$SCRIPT"
    fi
    if [ ! -d "$VENV" ]; then
        python3 -m venv "$VENV"
        "$VENV/bin/pip" install -q aiohttp toml tomlkit attrs
    fi
    "$VENV/bin/python3" "$SCRIPT" Cargo.lock -o flatpak/cargo-sources.json
    echo "Updated flatpak/cargo-sources.json"

# Install Flatpak runtimes required for `just install` (GNOME SDK 48, Rust + LLVM extensions)
setup-flatpak:
    #!/usr/bin/env bash
    set -euo pipefail
    GNOME_VER=48
    FDO_VER=24.08
    # Determine install scope: prefer user if flathub is configured there, else system
    if flatpak remotes --user 2>/dev/null | grep -q flathub; then
        SCOPE="--user"
    else
        SCOPE="--system"
    fi
    need_install=()
    flatpak info $SCOPE org.gnome.Platform//$GNOME_VER &>/dev/null || need_install+=("org.gnome.Platform//$GNOME_VER")
    flatpak info $SCOPE org.gnome.Sdk//$GNOME_VER &>/dev/null       || need_install+=("org.gnome.Sdk//$GNOME_VER")
    flatpak info $SCOPE org.freedesktop.Sdk.Extension.rust-stable//$FDO_VER &>/dev/null \
        || need_install+=("org.freedesktop.Sdk.Extension.rust-stable//$FDO_VER")
    flatpak info $SCOPE org.freedesktop.Sdk.Extension.llvm19//$FDO_VER &>/dev/null \
        || need_install+=("org.freedesktop.Sdk.Extension.llvm19//$FDO_VER")
    if [ ${#need_install[@]} -eq 0 ]; then
        echo "All Flatpak runtimes already installed."
    else
        echo "Installing missing Flatpak runtimes ($SCOPE): ${need_install[*]}"
        flatpak install $SCOPE --noninteractive flathub "${need_install[@]}"
        echo "Done."
    fi

# Install the Tauri CLI
install-tauri-cli:
    cargo install tauri-cli

# ── GNOME Extension ───────────────────────────────────

# Compile GSettings schemas for the GNOME extension
gnome-schema:
    glib-compile-schemas gnome-extension/schemas/

# Install the GNOME extension to the user profile
gnome-install: gnome-schema
    install -d ~/.local/share/gnome-shell/extensions/fotos@io.github.charly
    cp -r gnome-extension/. ~/.local/share/gnome-shell/extensions/fotos@io.github.charly/

# Pack the GNOME extension into a zip for distribution
gnome-pack: gnome-schema
    cd gnome-extension && zip -r ../fotos-gnome-extension.zip .

# ── Utilities ─────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean
