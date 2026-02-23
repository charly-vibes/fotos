/// Wayland screenshot capture via xdg-desktop-portal (ashpd).
///
/// Used on GNOME Wayland, KDE Wayland, and other Wayland compositors
/// that implement the Screenshot portal.
use anyhow::Result;

pub async fn capture_via_portal() -> Result<image::DynamicImage> {
    // TODO: use ashpd Screenshot::request().interactive(true).send().await
    // Portal returns file:///tmp/screenshot-XXXX.png which we load
    anyhow::bail!("Portal capture not yet implemented")
}
