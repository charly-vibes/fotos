pub mod detect;
pub mod portal;
pub mod xcap_backend;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
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
    pub image: Arc<image::DynamicImage>,
    pub metadata: CaptureMetadata,
}

/// Global image store shared across the application.
/// Used by capture, AI processing, and file operations.
#[derive(Clone)]
pub struct ImageStore {
    images: Arc<RwLock<HashMap<Uuid, Arc<image::DynamicImage>>>>,
}

impl ImageStore {
    pub fn new() -> Self {
        Self {
            images: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, id: Uuid, image: Arc<image::DynamicImage>) {
        self.images.write().unwrap().insert(id, image);
    }

    pub fn get(&self, id: &Uuid) -> Option<Arc<image::DynamicImage>> {
        self.images.read().unwrap().get(id).cloned()
    }

    pub fn remove(&self, id: &Uuid) -> Option<Arc<image::DynamicImage>> {
        self.images.write().unwrap().remove(id)
    }
}

impl Default for ImageStore {
    fn default() -> Self {
        Self::new()
    }
}
