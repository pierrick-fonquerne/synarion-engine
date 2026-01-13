//! View frustum for culling operations.

use glam::{Mat4, Vec3, Vec4};
use crate::aabb::Aabb;

/// A plane defined by its normal and distance from origin.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// The plane normal (pointing inward for frustum planes).
    pub normal: Vec3,
    /// The distance from the origin.
    pub distance: f32,
}

impl Plane {
    /// Creates a plane from a Vec4 (xyz = normal, w = distance).
    pub fn from_vec4(v: Vec4) -> Self {
        let length = Vec3::new(v.x, v.y, v.z).length();
        Self {
            normal: Vec3::new(v.x, v.y, v.z) / length,
            distance: v.w / length,
        }
    }

    /// Returns the signed distance from a point to this plane.
    pub fn signed_distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

/// A view frustum defined by six planes.
///
/// Used for frustum culling to determine which objects are visible.
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    /// The six planes of the frustum (left, right, bottom, top, near, far).
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Extracts the frustum planes from a view-projection matrix.
    pub fn from_view_projection(vp: Mat4) -> Self {
        let vp = vp.transpose();

        // Extract planes using Gribb/Hartmann method
        let left = Plane::from_vec4(vp.w_axis + vp.x_axis);
        let right = Plane::from_vec4(vp.w_axis - vp.x_axis);
        let bottom = Plane::from_vec4(vp.w_axis + vp.y_axis);
        let top = Plane::from_vec4(vp.w_axis - vp.y_axis);
        let near = Plane::from_vec4(vp.w_axis + vp.z_axis);
        let far = Plane::from_vec4(vp.w_axis - vp.z_axis);

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    /// Tests if an AABB is visible (intersects or is inside the frustum).
    pub fn contains_aabb(&self, aabb: &Aabb) -> bool {
        for plane in &self.planes {
            // Find the corner of the AABB most in the direction of the plane normal
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if plane.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if plane.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
            );

            if plane.signed_distance(p) < 0.0 {
                return false;
            }
        }

        true
    }

    /// Tests if a sphere is visible.
    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.signed_distance(center) < -radius {
                return false;
            }
        }

        true
    }

    /// Tests if a point is inside the frustum.
    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            if plane.signed_distance(point) < 0.0 {
                return false;
            }
        }

        true
    }
}
