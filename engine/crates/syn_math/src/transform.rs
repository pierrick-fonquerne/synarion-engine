//! Hierarchical transform component.

use glam::{Mat4, Quat, Vec3};

/// A transform representing position, rotation, and scale.
///
/// This is the fundamental component for positioning objects in 3D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The position in local space.
    pub translation: Vec3,
    /// The rotation as a quaternion.
    pub rotation: Quat,
    /// The scale factor.
    pub scale: Vec3,
}

impl Transform {
    /// The identity transform (no translation, rotation, or scale).
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Creates a new transform with the given translation, rotation, and scale.
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Creates a transform with only translation.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Creates a transform with only rotation.
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// Creates a transform with only scale.
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    /// Creates a transform with uniform scale.
    pub fn from_uniform_scale(scale: f32) -> Self {
        Self::from_scale(Vec3::splat(scale))
    }

    /// Returns a new transform with the given translation.
    pub fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    /// Returns a new transform with the given rotation.
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Returns a new transform with the given scale.
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    /// Converts this transform to a 4x4 matrix.
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Creates a transform from a 4x4 matrix.
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Returns the forward direction (-Z) of this transform.
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Returns the right direction (+X) of this transform.
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Returns the up direction (+Y) of this transform.
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Transforms a point from local space to world space.
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.rotation * (point * self.scale) + self.translation
    }

    /// Transforms a direction from local space to world space (ignores translation).
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation * direction
    }

    /// Multiplies two transforms together (parent * child).
    pub fn mul_transform(&self, child: &Transform) -> Transform {
        Transform {
            translation: self.transform_point(child.translation),
            rotation: self.rotation * child.rotation,
            scale: self.scale * child.scale,
        }
    }

    /// Returns the inverse of this transform.
    pub fn inverse(&self) -> Transform {
        let inv_rotation = self.rotation.inverse();
        let inv_scale = Vec3::ONE / self.scale;
        Transform {
            translation: inv_rotation * (-self.translation * inv_scale),
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }

    /// Linearly interpolates between two transforms.
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        self.mul_transform(&rhs)
    }
}
