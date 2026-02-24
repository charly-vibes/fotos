/// Wayland screenshot capture via xdg-desktop-portal (ashpd).
///
/// Used on GNOME Wayland, KDE Wayland, and other Wayland compositors
/// that implement the Screenshot portal.
use anyhow::{bail, Context, Result};

pub async fn capture_via_portal() -> Result<image::DynamicImage> {
    use ashpd::desktop::screenshot::Screenshot;

    // Use interactive=true so GNOME shows a confirmation dialog on first use,
    // which also grants the screenshot permission. Without this, GNOME's portal
    // returns "Other" (error code 2) immediately if no permission has been stored.
    tracing::info!("portal: sending screenshot request (interactive=true)");

    let request = Screenshot::request()
        .interactive(true)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("portal: send() failed: {e}");
            anyhow::anyhow!("Portal unavailable: {e}")
        })?;

    tracing::info!("portal: request sent, awaiting user confirmation");

    let response = request
        .response()
        .map_err(|e| {
            tracing::error!("portal: response() failed: {e}");
            anyhow::anyhow!("Screenshot portal request failed: {e}")
        })?;

    let uri = response.uri();
    tracing::info!("portal: got URI {uri}");

    if uri.scheme() != "file" {
        bail!(
            "Portal returned unsupported URI scheme '{}' (expected file://)",
            uri.scheme()
        );
    }

    let path = uri
        .to_file_path()
        .map_err(|_| anyhow::anyhow!("Portal URI is not a valid file path: {uri}"))?;

    tracing::info!("portal: loading image from {}", path.display());

    let img = image::open(&path)
        .with_context(|| format!("Failed to load portal screenshot from {}", path.display()))?;

    tracing::info!("portal: image loaded ({}x{}), cleaning up temp file", img.width(), img.height());

    // Clean up the temp file left by the portal.
    let _ = std::fs::remove_file(&path);

    Ok(img)
}
