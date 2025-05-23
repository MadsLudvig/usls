use aksr::Builder;
use anyhow::Result;
use image::GrayImage;

use crate::{InstanceMeta, Polygon, Style};

/// Heatmap: Gray Image.
#[derive(Builder, Default, Clone)]
pub struct HeatMap {
    map: GrayImage,
    meta: InstanceMeta,
    style: Option<Style>,
}

impl std::fmt::Debug for HeatMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeatMap")
            .field("dimensions", &self.dimensions())
            .field("uid", &self.meta.uid())
            .field("id", &self.meta.id())
            .field("name", &self.meta.name())
            .field("confidence", &self.meta.confidence())
            .finish()
    }
}

impl PartialEq for HeatMap {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl HeatMap {
    pub fn new(u8s: &[u8], width: u32, height: u32) -> Result<Self> {
        let map: image::ImageBuffer<image::Luma<_>, Vec<_>> =
            image::ImageBuffer::from_raw(width, height, u8s.to_vec())
                .ok_or(anyhow::anyhow!("Failed to build ImageBuffer."))?;

        Ok(Self {
            map,
            ..Default::default()
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.map.to_vec()
    }

    pub fn height(&self) -> u32 {
        self.map.height()
    }

    pub fn width(&self) -> u32 {
        self.map.width()
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.map.dimensions()
    }
}

impl HeatMap {
    pub fn with_uid(mut self, uid: usize) -> Self {
        self.meta = self.meta.with_uid(uid);
        self
    }
    pub fn with_id(mut self, id: usize) -> Self {
        self.meta = self.meta.with_id(id);
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.meta = self.meta.with_name(name);
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.meta = self.meta.with_confidence(confidence);
        self
    }

    pub fn uid(&self) -> usize {
        self.meta.uid()
    }

    pub fn name(&self) -> Option<&str> {
        self.meta.name()
    }

    pub fn confidence(&self) -> Option<f32> {
        self.meta.confidence()
    }

    pub fn id(&self) -> Option<usize> {
        self.meta.id()
    }
}
