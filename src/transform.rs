use nalgebra::{Matrix4, Rotation3, Scale3, Translation3};

type T = Translation3<f32>;
type R = Rotation3<f32>;
type S = Scale3<f32>;

pub struct Transform {
    pub translation: T,
    pub rotation: R,
    pub scale: S,
}

impl Transform {
    pub fn new(translation: T, rotation: R, scale: S) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        self.translation.to_homogeneous()
            * self.rotation.to_homogeneous()
            * self.scale.to_homogeneous()
    }
}
