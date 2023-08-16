use nalgebra::Matrix4;
use wgpu::{vertex_attr_array, BufferUsages, ShaderStages};

use crate::{
    bind::{BindEntry, BindEntryType, BindHandle},
    camera::CameraUniform,
    geometry::Geometry,
    instance::InstanceData,
    pipeline::{Pipeline, PipelineBuilder},
    plain::Plain,
    render::Render,
};

#[repr(C)]
#[derive(Clone)]
pub struct TextVertex {
    pos: [f32; 3],
}

unsafe impl Plain for TextVertex {}

#[repr(C)]
#[derive(Debug)]
pub struct TextInstance {
    pub transform: Matrix4<f32>,
    pub albedo: [f32; 4],
}

unsafe impl Plain for TextInstance {}

impl InstanceData for TextInstance {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

pub struct TextGeometry {
    pub vertices: [TextVertex; 4], // once a quad, always a quad
    pub indices: Option<[u16; 6]>,
}

impl Geometry for TextGeometry {
    fn contents(&self) -> &[u8] {
        todo!()
    }

    fn length(&self) -> u32 {
        todo!()
    }

    fn indices(&self) -> Option<&[u8]> {
        todo!()
    }
}

pub fn text_pipeline(render: &mut Render) -> (Pipeline, BindHandle) {
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
        .with_shader(include_str!("../shaders/text.wgsl"))
        .with_vb::<TextVertex>(
            wgpu::VertexStepMode::Vertex,
            &vertex_attr_array![
                // position
                0 => Float32x3
            ],
        )
        .with_vb::<TextInstance>(
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
