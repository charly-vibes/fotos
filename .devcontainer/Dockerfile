FROM fedora:43

# Tauri 2 system dependencies
RUN dnf install -y \
    # Build essentials
    gcc gcc-c++ make cmake pkg-config \
    # GTK / WebKitGTK (Tauri webview)
    gtk3-devel \
    webkit2gtk4.1-devel \
    javascriptcoregtk4.1-devel \
    libsoup3-devel \
    # Cairo / Pango / GDK (transitive deps)
    cairo-devel \
    pango-devel \
    gdk-pixbuf2-devel \
    atk-devel \
    # Tray icon
    libappindicator-gtk3-devel \
    # D-Bus (for ashpd / xdg-desktop-portal)
    dbus-devel \
    # PipeWire (for xcap screen capture)
    pipewire-devel \
    # Bindgen (needs libclang)
    clang-devel \
    # TLS
    openssl-devel \
    # OCR
    tesseract-devel \
    leptonica-devel \
    # Misc
    curl wget git file \
    && dnf clean all

# Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Tauri CLI
RUN cargo install tauri-cli
