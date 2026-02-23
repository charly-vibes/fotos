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

        // Collect monitor geometry using actual position offsets.
        // This correctly handles vertical stacking, non-contiguous monitors,
        // and negative offsets (e.g. a monitor placed to the left of the primary).
        struct MonitorGeom {
            x: i32,
            y: i32,
            width: u32,
            height: u32,
            image: ImageBuffer<Rgba<u8>, Vec<u8>>,
        }

        let mut geoms: Vec<MonitorGeom> = Vec::with_capacity(monitors.len());
        for monitor in monitors {
            geoms.push(MonitorGeom {
                x: monitor.x()?,
                y: monitor.y()?,
                width: monitor.width()?,
                height: monitor.height()?,
                image: monitor.capture_image()?,
            });
        }

        // Compute bounding box across all monitor positions.
        let min_x = geoms.iter().map(|g| g.x).min().unwrap();
        let min_y = geoms.iter().map(|g| g.y).min().unwrap();
        let max_x = geoms.iter().map(|g| g.x + g.width as i32).max().unwrap();
        let max_y = geoms.iter().map(|g| g.y + g.height as i32).max().unwrap();

        let canvas_w = (max_x - min_x) as u32;
        let canvas_h = (max_y - min_y) as u32;

        let mut composite = ImageBuffer::from_pixel(canvas_w, canvas_h, Rgba([0, 0, 0, 255]));

        for geom in geoms {
            let offset_x = (geom.x - min_x) as i64;
            let offset_y = (geom.y - min_y) as i64;
            image::imageops::overlay(&mut composite, &geom.image, offset_x, offset_y);
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
