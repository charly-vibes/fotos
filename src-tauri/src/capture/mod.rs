pub mod detect;
#[cfg(target_os = "linux")]
pub mod portal;
pub mod xcap_backend;

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
        self.images
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, image);
    }

    pub fn get(&self, id: &Uuid) -> Option<Arc<image::DynamicImage>> {
        self.images
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(id)
            .cloned()
    }

    pub fn remove(&self, id: &Uuid) -> Option<Arc<image::DynamicImage>> {
        self.images
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(id)
    }
}

impl Default for ImageStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    fn dummy_image() -> Arc<DynamicImage> {
        Arc::new(DynamicImage::ImageRgba8(RgbaImage::new(10, 10)))
    }

    #[test]
    fn image_store_insert_and_get() {
        let store = ImageStore::new();
        let id = Uuid::new_v4();
        let img = dummy_image();
        store.insert(id, img.clone());
        assert!(store.get(&id).is_some());
    }

    #[test]
    fn image_store_get_missing_returns_none() {
        let store = ImageStore::new();
        assert!(store.get(&Uuid::new_v4()).is_none());
    }

    #[test]
    fn image_store_remove_clears_entry() {
        let store = ImageStore::new();
        let id = Uuid::new_v4();
        store.insert(id, dummy_image());
        assert!(store.remove(&id).is_some());
        assert!(store.get(&id).is_none());
    }

    #[test]
    fn image_store_get_after_remove_is_none() {
        let store = ImageStore::new();
        let id = Uuid::new_v4();
        store.insert(id, dummy_image());
        store.remove(&id);
        assert!(store.get(&id).is_none());
    }
}
