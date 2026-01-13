//! Ray implementation for raycasting.

use glam::Vec3;
use crate::aabb::Aabb;

/// A ray defined by an origin and a direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    /// The origin of the ray.
    pub origin: Vec3,
    /// The normalized direction of the ray.
    pub direction: Vec3,
}

impl Ray {
    /// Creates a new ray from origin and direction.
    ///
    /// The direction is automatically normalized.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Returns a point along the ray at the given distance.
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Tests for intersection with an AABB.
    ///
    /// Returns the distance to the intersection point if there is one.
    pub fn intersects_aabb(&self, aabb: &Aabb) -> Option<f32> {
        let inv_dir = Vec3::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (aabb.min - self.origin) * inv_dir;
        let t2 = (aabb.max - self.origin) * inv_dir;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_enter = t_min.x.max(t_min.y).max(t_min.z);
        let t_exit = t_max.x.min(t_max.y).min(t_max.z);

        if t_enter <= t_exit && t_exit >= 0.0 {
            Some(t_enter.max(0.0))
        } else {
            None
        }
    }

    /// Tests for intersection with a plane defined by a point and normal.
    ///
    /// Returns the distance to the intersection point if there is one.
    pub fn intersects_plane(&self, plane_point: Vec3, plane_normal: Vec3) -> Option<f32> {
        let denom = plane_normal.dot(self.direction);

        if denom.abs() > f32::EPSILON {
            let t = (plane_point - self.origin).dot(plane_normal) / denom;
            if t >= 0.0 {
                return Some(t);
            }
        }

        None
    }
}
