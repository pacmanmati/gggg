use std::fmt::Debug;

use crate::plain::Plain;

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

unsafe impl Plain for Vertex {}

#[derive(Debug)]
pub struct BasicGeometry {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u16>>,
}

impl Geometry for BasicGeometry {
    fn contents(&self) -> &[u8] {
        self.vertices.as_bytes()
    }

    fn length(&self) -> u32 {
        self.vertices.len() as u32
    }

    fn indices(&self) -> Option<&[u8]> {
        self.indices.as_ref().map(|indices| indices.as_bytes())
    }
}

pub trait Geometry: Debug {
    fn contents(&self) -> &[u8];

    fn length(&self) -> u32;

    fn indices(&self) -> Option<&[u8]>;
}

impl Geometry for Box<dyn Geometry> {
    fn contents(&self) -> &[u8] {
        self.as_ref().contents()
    }

    fn length(&self) -> u32 {
        self.as_ref().length()
    }

    fn indices(&self) -> Option<&[u8]> {
        self.as_ref().indices()
    }
}
