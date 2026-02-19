/// Screenshot capture via xcap crate.
///
/// Used on X11 Linux and Windows where direct capture APIs are available
/// without requiring a portal.

use anyhow::Result;

pub async fn capture_fullscreen() -> Result<image::DynamicImage> {
    // TODO: use xcap Monitor::all() to capture
    anyhow::bail!("xcap fullscreen capture not yet implemented")
}

pub async fn capture_monitor(index: u32) -> Result<image::DynamicImage> {
    // TODO: capture specific monitor by index
    anyhow::bail!("xcap monitor capture not yet implemented")
}

pub async fn capture_window(window_id: u64) -> Result<image::DynamicImage> {
    // TODO: capture specific window
    anyhow::bail!("xcap window capture not yet implemented")
}
