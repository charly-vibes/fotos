#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    LinuxWaylandGnome,
    LinuxWaylandKde,
    LinuxWaylandOther,
    LinuxX11,
    Windows,
}

pub fn detect_platform() -> Platform {
    // TODO: check XDG_SESSION_TYPE, XDG_CURRENT_DESKTOP, WAYLAND_DISPLAY
    #[cfg(target_os = "windows")]
    {
        return Platform::Windows;
    }

    #[cfg(target_os = "linux")]
    {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

        if session_type == "wayland" {
            if desktop.contains("GNOME") {
                Platform::LinuxWaylandGnome
            } else if desktop.contains("KDE") {
                Platform::LinuxWaylandKde
            } else {
                Platform::LinuxWaylandOther
            }
        } else {
            Platform::LinuxX11
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Platform::LinuxX11
    }
}
