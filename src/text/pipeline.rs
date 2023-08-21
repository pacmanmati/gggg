use std::rc::Rc;

use nalgebra::{Matrix4, Vector4};
use wgpu::{vertex_attr_array, BufferUsages, Extent3d, ShaderStages, TextureUsages};

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

use super::font_bitmap_manager::FontBitmapManager;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct TextVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

unsafe impl Plain for TextVertex {}

#[repr(C)]
#[derive(Debug)]
pub struct TextInstance {
    pub transform: Matrix4<f32>,
    pub albedo: [f32; 4],
    pub atlas_coords: Vector4<f32>,
}

unsafe impl Plain for TextInstance {}

impl InstanceData for TextInstance {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[derive(Debug)]
pub struct TextGeometry {
    pub vertices: [TextVertex; 4],
    pub indices: Option<[u16; 6]>,
}

impl Geometry for TextGeometry {
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

pub const fn quad_geometry() -> TextGeometry {
    TextGeometry {
        vertices: quad_shape(),
        indices: Some([0, 2, 1, 1, 2, 3]),
    }
}

pub const fn quad_shape() -> [TextVertex; 4] {
    [
        TextVertex {
            pos: [0.0, 0.0, 0.0],
            uv: [0.0, 1.0],
        },
        TextVertex {
            pos: [1.0, 0.0, 0.0],
            uv: [1.0, 1.0],
        },
        TextVertex {
            pos: [0.0, 1.0, 0.0],
            uv: [0.0, 0.0],
        },
        TextVertex {
            pos: [1.0, 1.0, 0.0],
            uv: [1.0, 0.0],
        },
    ]
}

#[derive(Debug)]
pub struct TextRenderObject {
    pub transform: Matrix4<f32>,
    pub albedo: [f32; 4],
    pub pipeline_handle: PipelineHandle,
    pub mesh_handle: MeshHandle,
    pub character: char,
    pub manager: Rc<FontBitmapManager>,
}

impl RenderObject for TextRenderObject {
    type InstanceType = TextInstance;

    type GeometryType = TextGeometry;

    type MaterialType = BasicMaterial;

    fn instance(&self, render: &Render) -> Self::InstanceType {
        // we pass an external object which knows which letter maps onto which texture handle
        // do we want the texture handles to be registered to the renderer?
        // probably yes - existing code relies on any atlas textures being stored on the renderer
        // so the new 'font' struct will need to co-operate with the renderer
        let texture_handle = self.manager.get_texture(self.character).unwrap();
        let atlas_coords = render
            .get_atlas_coords_for_texture(texture_handle, self.manager.atlas_handle)
            .unwrap();
        TextInstance {
            transform: self.transform,
            albedo: self.albedo,
            atlas_coords: atlas_coords.into(),
        }
    }

    fn pipeline_handle(&self) -> crate::pipeline::PipelineHandle {
        self.pipeline_handle
    }

    fn mesh_handle(&self) -> crate::render::MeshHandle {
        self.mesh_handle
    }
}

pub fn text_pipeline(render: &mut Render) -> (Pipeline, BindHandle) {
    let defaults_bind = render.build_bind(&mut [
        // camera
        BindEntry {
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindEntryType::BufferUniform {
                size: std::mem::size_of::<CameraUniform>() as u64,
                usages: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },
            count: None,
        },
        // texture atlas
        BindEntry {
            visibility: ShaderStages::FRAGMENT,
            ty: BindEntryType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_count: 1,
                format: wgpu::TextureFormat::Rgba8Unorm,
                // format: wgpu::TextureFormat::R8Unorm,
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            },
            count: None,
        },
        // sampler
        BindEntry {
            visibility: ShaderStages::FRAGMENT,
            ty: BindEntryType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        },
    ]);

    let pipeline_handle = PipelineBuilder::new()
        .with_format(wgpu::TextureFormat::Bgra8UnormSrgb)
        // .with_cull_mode(Some(wgpu::Face::Back))
        .with_cull_mode(None)
        .with_bind(defaults_bind)
        .with_shader(include_str!("../shaders/text.wgsl"))
        .with_vb::<TextVertex>(
            wgpu::VertexStepMode::Vertex,
            &vertex_attr_array![
                // position
                0 => Float32x3,
                // uv
                1 => Float32x2,
            ],
        )
        .with_vb::<TextInstance>(
            wgpu::VertexStepMode::Instance,
            &vertex_attr_array![
                // transform
                2 => Float32x4,
                3 => Float32x4,
                4 => Float32x4,
                5 => Float32x4,
                // albedo
                6 => Float32x4,
                // atlas coords
                7 => Float32x4,
            ],
        )
        .build(render);

    (pipeline_handle, defaults_bind)
}
