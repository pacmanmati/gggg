use nalgebra::{Matrix4, Vector4};
use std::fmt::Debug;

use crate::plain::Plain;

// TODO: rename to just `Instance`
pub trait InstanceData: Debug {
    fn data(&self) -> &[u8];
}

impl InstanceData for Box<dyn InstanceData> {
    fn data(&self) -> &[u8] {
        self.as_ref().data()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct BasicInstance {
    transform: Matrix4<f32>,
    atlas_coords: Vector4<u32>,
}

impl InstanceData for BasicInstance {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

unsafe impl Plain for BasicInstance {}
