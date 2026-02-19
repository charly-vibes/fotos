pub mod detect;
pub mod portal;
pub mod xcap_backend;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureMode {
    Fullscreen,
    Monitor(u32),
    Region { x: i32, y: i32, w: u32, h: u32 },
    Window(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMetadata {
    pub timestamp: DateTime<Utc>,
    pub mode: CaptureMode,
    pub monitor: Option<String>,
    pub window_title: Option<String>,
    pub dimensions: (u32, u32),
}

#[derive(Debug)]
pub struct CaptureResult {
    pub id: Uuid,
    pub image: image::DynamicImage,
    pub metadata: CaptureMetadata,
}
