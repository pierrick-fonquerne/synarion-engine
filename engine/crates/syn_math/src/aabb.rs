//! Axis-Aligned Bounding Box implementation.

use glam::Vec3;

/// An axis-aligned bounding box defined by its minimum and maximum corners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// The minimum corner of the bounding box.
    pub min: Vec3,
    /// The maximum corner of the bounding box.
    pub max: Vec3,
}

impl Aabb {
    /// Creates a new AABB from minimum and maximum corners.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Creates an AABB centered at the origin with the given half-extents.
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Creates an empty AABB (inverted, ready to be expanded).
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Returns the center of the AABB.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the half-extents of the AABB.
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Returns the size (full extents) of the AABB.
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Returns true if the AABB contains the given point.
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Returns true if this AABB intersects with another.
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Expands this AABB to include the given point.
    pub fn expand_to_include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Returns the union of this AABB with another.
    pub fn union(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self::empty()
    }
}
