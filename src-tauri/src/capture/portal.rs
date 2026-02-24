/// Wayland screenshot capture via xdg-desktop-portal (ashpd).
///
/// Used on GNOME Wayland, KDE Wayland, and other Wayland compositors
/// that implement the Screenshot portal.
use anyhow::{bail, Context, Result};

pub async fn capture_via_portal() -> Result<image::DynamicImage> {
    use ashpd::desktop::screenshot::Screenshot;

    let response = Screenshot::request()
        .interactive(false)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Portal unavailable: {e}"))?
        .response()
        .map_err(|e| anyhow::anyhow!("Screenshot portal request failed: {e}"))?;

    let uri = response.uri();

    if uri.scheme() != "file" {
        bail!(
            "Portal returned unsupported URI scheme '{}' (expected file://)",
            uri.scheme()
        );
    }

    let path = uri
        .to_file_path()
        .map_err(|_| anyhow::anyhow!("Portal URI is not a valid file path: {uri}"))?;

    let img = image::open(&path)
        .with_context(|| format!("Failed to load portal screenshot from {}", path.display()))?;

    // Clean up the temp file left by the portal.
    let _ = std::fs::remove_file(&path);

    Ok(img)
}
