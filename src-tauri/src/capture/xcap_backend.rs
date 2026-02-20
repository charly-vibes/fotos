/// Screenshot capture via xcap crate.
///
/// Used on X11 Linux and Windows where direct capture APIs are available
/// without requiring a portal.

use anyhow::Result;
use image::{DynamicImage, ImageBuffer, Rgba};
use xcap::Monitor;

pub async fn capture_fullscreen() -> Result<DynamicImage> {
    // Run xcap in a blocking task to avoid nested runtime issues
    // xcap uses zbus which creates a tokio runtime internally on Wayland
    tokio::task::spawn_blocking(|| {
        // Capture all monitors and composite into a single image
        let monitors = Monitor::all()?;

        if monitors.is_empty() {
            anyhow::bail!("No monitors detected");
        }

        // For tracer-bullet: capture all monitors and place side-by-side
        // Find total width and max height
        let mut total_width: u32 = 0;
        let mut max_height: u32 = 0;

        for monitor in &monitors {
            total_width += monitor.width()?;
            let height = monitor.height()?;
            if height > max_height {
                max_height = height;
            }
        }

        // Create composite image
        let mut composite = ImageBuffer::from_pixel(total_width, max_height, Rgba([0, 0, 0, 255]));

        let mut x_offset = 0u32;
        for monitor in monitors {
            let screenshot = monitor.capture_image()?;
            // screenshot is already an ImageBuffer<Rgba<u8>, Vec<u8>>
            image::imageops::overlay(&mut composite, &screenshot, x_offset as i64, 0);
            x_offset += screenshot.width();
        }

        Ok(DynamicImage::ImageRgba8(composite))
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
}

pub async fn capture_monitor(index: u32) -> Result<image::DynamicImage> {
    // TODO: capture specific monitor by index
    anyhow::bail!("xcap monitor capture not yet implemented")
}

pub async fn capture_window(window_id: u64) -> Result<image::DynamicImage> {
    // TODO: capture specific window
    anyhow::bail!("xcap window capture not yet implemented")
}
