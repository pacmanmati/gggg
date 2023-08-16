use nalgebra::Matrix4;
use wgpu::{vertex_attr_array, BufferUsages, ShaderStages};

use crate::{
    bind::{BindEntry, BindEntryType, BindHandle},
    camera::CameraUniform,
    geometry::Geometry,
    instance::InstanceData,
    material::BasicMaterial,
    pipeline::{Pipeline, PipelineBuilder, PipelineHandle},
    plain::Plain,
    render::{MeshHandle, Render},
    render_object::RenderObject,
};

#[repr(C)]
#[derive(Clone)]
pub struct ShapeVertex {
    pos: [f32; 3],
}

unsafe impl Plain for ShapeVertex {}

#[repr(C)]
#[derive(Debug)]
pub struct ShapeInstance {
    pub transform: Matrix4<f32>,
    pub albedo: [f32; 4],
}

unsafe impl Plain for ShapeInstance {}

impl InstanceData for ShapeInstance {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

pub struct ShapeGeometry {
    pub vertices: [ShapeVertex; 4],
    pub indices: Option<[u16; 6]>,
}

impl Geometry for ShapeGeometry {
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

pub fn shape_pipeline(render: &mut Render) -> (Pipeline, BindHandle) {
    let defaults_bind = render.build_bind(&mut [BindEntry {
        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
        ty: BindEntryType::BufferUniform {
            size: std::mem::size_of::<CameraUniform>() as u64,
            usages: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        },
        count: None,
    }]);

    let pipeline_handle = PipelineBuilder::new()
        .with_format(wgpu::TextureFormat::Bgra8UnormSrgb)
        .with_cull_mode(Some(wgpu::Face::Back))
        .with_bind(defaults_bind)
        .with_shader(include_str!("shaders/shapes_shader.wgsl"))
        .with_vb::<ShapeVertex>(
            wgpu::VertexStepMode::Vertex,
            &vertex_attr_array![
                // position
                0 => Float32x3
            ],
        )
        .with_vb::<ShapeInstance>(
            wgpu::VertexStepMode::Instance,
            &vertex_attr_array![
                // transform
                1 => Float32x4,
                2 => Float32x4,
                3 => Float32x4,
                4 => Float32x4,
                // albedo
                5 => Float32x4,
            ],
        )
        .build(render);

    (pipeline_handle, defaults_bind)
}

pub const fn quad_geometry() -> ShapeGeometry {
    ShapeGeometry {
        vertices: quad_shape(),
        indices: Some([0, 2, 1, 1, 2, 3]),
    }
}

pub const fn quad_shape() -> [ShapeVertex; 4] {
    [
        ShapeVertex {
            pos: [-0.5, -0.5, 0.0],
        },
        ShapeVertex {
            pos: [0.5, -0.5, 0.0],
        },
        ShapeVertex {
            pos: [-0.5, 0.5, 0.0],
        },
        ShapeVertex {
            pos: [0.5, 0.5, 0.0],
        },
    ]
}

pub struct ShapeRenderObject {
    pub transform: Matrix4<f32>,
    pub albedo: [f32; 4],
    pub pipeline_handle: PipelineHandle,
    pub mesh_handle: MeshHandle,
}

impl RenderObject for ShapeRenderObject {
    type InstanceType = ShapeInstance;

    type GeometryType = ShapeGeometry;

    type MaterialType = BasicMaterial;

    fn instance(&self, render: &Render) -> Self::InstanceType {
        ShapeInstance {
            transform: self.transform,
            albedo: self.albedo,
        }
    }

    fn pipeline_handle(&self) -> crate::pipeline::PipelineHandle {
        self.pipeline_handle
    }

    fn mesh_handle(&self) -> crate::render::MeshHandle {
        self.mesh_handle
    }
}
