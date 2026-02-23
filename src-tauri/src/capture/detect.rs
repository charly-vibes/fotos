#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    LinuxWaylandGnome,
    LinuxWaylandKde,
    LinuxWaylandOther,
    LinuxX11,
    Windows,
}

pub fn detect_platform() -> Platform {
    #[cfg(target_os = "windows")]
    {
        return Platform::Windows;
    }

    #[cfg(target_os = "linux")]
    {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
        return detect_linux(&session_type, &desktop, &wayland_display);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    Platform::LinuxX11
}

/// Pure detection logic for Linux, separated for testability.
fn detect_linux(session_type: &str, desktop: &str, wayland_display: &str) -> Platform {
    let is_wayland = session_type == "wayland"
        || (session_type.is_empty() && !wayland_display.is_empty());

    if is_wayland {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_session_type_gnome() {
        assert_eq!(
            detect_linux("wayland", "GNOME", ""),
            Platform::LinuxWaylandGnome
        );
    }

    #[test]
    fn wayland_session_type_kde() {
        assert_eq!(
            detect_linux("wayland", "KDE", ""),
            Platform::LinuxWaylandKde
        );
    }

    #[test]
    fn wayland_session_type_other() {
        assert_eq!(
            detect_linux("wayland", "Sway", ""),
            Platform::LinuxWaylandOther
        );
    }

    #[test]
    fn unset_session_with_wayland_display_is_wayland() {
        assert_eq!(
            detect_linux("", "Sway", "wayland-0"),
            Platform::LinuxWaylandOther
        );
    }

    #[test]
    fn unset_session_without_wayland_display_is_x11() {
        assert_eq!(detect_linux("", "", ""), Platform::LinuxX11);
    }

    #[test]
    fn x11_session_type_is_x11() {
        assert_eq!(detect_linux("x11", "GNOME", ""), Platform::LinuxX11);
    }
}
