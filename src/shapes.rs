use wgpu::{vertex_attr_array, BufferUsages, ShaderStages};

use crate::{
    bind::{BindEntry, BindEntryType},
    camera::CameraUniform,
    geometry::Geometry,
    instance::InstanceData,
    material::BasicMaterial,
    pipeline::PipelineBuilder,
    plain::Plain,
    render::Render,
    render_object::RenderObject,
};

#[repr(C)]
pub struct ShapeVertex {
    pos: [f32; 3],
}

unsafe impl Plain for ShapeVertex {}

pub struct ShapeInstance {}

#[repr(C)]
#[derive(Debug)]
pub struct ShapeInstanceData {}

unsafe impl Plain for ShapeInstance {}

impl InstanceData for ShapeInstanceData {
    fn data(&self) -> &[u8] {
        todo!()
    }
}

pub struct ShapeGeometry {
    pub vertices: Vec<ShapeVertex>,
    pub indices: Option<Vec<u16>>,
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

pub fn pipeline(render: &mut Render) {
    let defaults_bind = render.build_bind(&mut [BindEntry {
        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
        ty: BindEntryType::BufferUniform {
            size: std::mem::size_of::<CameraUniform>() as u64,
            usages: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        },
        count: None,
    }]);

    PipelineBuilder::new()
        .with_cull_mode(Some(wgpu::Face::Back))
        .with_bind(defaults_bind)
        .with_shader("shapes_shader.wgsl")
        .with_vb::<ShapeVertex>(
            wgpu::VertexStepMode::Vertex,
            &vertex_attr_array![0 => Float32x3],
        )
        .with_vb::<ShapeInstance>(wgpu::VertexStepMode::Instance, &vertex_attr_array![])
        .build(render);
}

pub const fn quad_shape() -> [ShapeVertex; 4] {
    [
        ShapeVertex {
            pos: [0.0, 0.0, 0.0],
        },
        ShapeVertex {
            pos: [0.0, 0.0, 0.0],
        },
        ShapeVertex {
            pos: [0.0, 0.0, 0.0],
        },
        ShapeVertex {
            pos: [0.0, 0.0, 0.0],
        },
    ]
}

pub struct ShapeRenderObject {}

impl RenderObject for ShapeRenderObject {
    type InstanceType = ShapeInstanceData;

    type GeometryType = ShapeGeometry;

    type MaterialType = BasicMaterial; // temporary, material doesn't do anything right now

    fn instance(
        &self,
        render: &Render,
        // mesh: Mesh<Self::GeometryType, Self::MaterialType>,
    ) -> Self::InstanceType {
        todo!()
    }

    fn pipeline_handle(&self) -> crate::pipeline::PipelineHandle {
        todo!()
    }

    fn mesh_handle(&self) -> crate::render::MeshHandle {
        todo!()
    }
}
