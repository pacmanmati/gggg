use std::mem::take;

use generational_arena::Index;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, Device, Face, FragmentState, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, TextureFormat, VertexAttribute, VertexState,
    VertexStepMode,
};

use crate::{
    bind::{BindHandle, VertexBufferEntry},
    render::Render,
};

#[derive(Debug)]
pub struct PipelineHandle(pub Index);

/// An abstraction over wgpu pipelines that simplifies custom pipeline creation.
/// A pipeline is constructed using a [PipelineBuilder] which bundles the creation of [wgpu::RenderPipeline], [wgpu::ShaderModule], etc.
///
/// The api could look like this:
/// ```
/// let pipeline = PipelineBuilder::new().with_globals(DEFAULT_GLOBALS).with_cull_mode(CullMode::BACK).build();
///
/// ```
// from a usability perspective: how will shaders work?
// with the default globals (e.g. camera) how do we get shader code that does what we want? do we want to auto generate it? specify which binding/group it is via documentation?
// it's probably best to expose an api that lets you set up your own bindings and groups - then the DEFAULT_GLOBALS just use that underneath.
pub struct Pipeline {
    pub pipeline: RenderPipeline,
    pub binds: Vec<BindHandle>,
}

// impl Pipeline {}

pub struct PipelineBuilder {
    // bgs: Vec<Vec<BindEntry>>,
    binds: Vec<BindHandle>,
    shader_src: Option<String>,
    primitive_state: PrimitiveState,
    format: TextureFormat,
    vertex_entries: Vec<VertexBufferEntry>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            binds: Vec::new(),
            shader_src: None,
            primitive_state: PrimitiveState::default(),
            format: TextureFormat::Rgba8Unorm,
            vertex_entries: Vec::new(),
        }
    }

    pub fn with_cull_mode(mut self, cull_mode: Option<Face>) -> Self {
        self.primitive_state.cull_mode = cull_mode;
        self
    }

    pub fn with_shader(mut self, shader_src: &str) -> Self {
        self.shader_src = Some(shader_src.into());
        self
    }

    pub fn with_format(mut self, format: TextureFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_bind(mut self, handle: BindHandle) -> Self {
        self.binds.push(handle);
        self
    }

    // pub fn with_bg(mut self, binds: Vec<BindEntry>) -> Self {
    //     self.bgs.push(binds);
    //     self
    // }

    pub fn with_vb<T>(mut self, step_mode: VertexStepMode, attributes: &[VertexAttribute]) -> Self {
        self.vertex_entries.push(VertexBufferEntry {
            array_stride: std::mem::size_of::<T>() as u64,
            step_mode,
            attributes: attributes.into(),
        });
        self
    }

    // fn build_bind(bg: &mut [BindEntry], device: &Device) -> Bind {
    //     let layout_entries = bg
    //         .iter()
    //         .enumerate()
    //         .map(|(idx, g)| g.layout_entry(idx as u32))
    //         .collect::<Vec<_>>();

    //     let bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
    //         label: None,
    //         entries: &layout_entries,
    //     });

    //     let group_entries = bg
    //         .iter_mut()
    //         .enumerate()
    //         .map(|(idx, g)| g.group_entry(idx as u32, device))
    //         .collect::<Vec<_>>();

    //     let bind_groups = device.create_bind_group(&BindGroupDescriptor {
    //         label: None,
    //         layout: &bgl,
    //         entries: &group_entries,
    //     });

    //     let resources = bg
    //         .iter_mut()
    //         .map(|entry| entry.resource.take().unwrap())
    //         .collect();

    //     Bind {
    //         bg: bind_groups,
    //         bgl,
    //         resources,
    //     }
    // }

    // fn build_binds(&mut self, device: &Device) -> Vec<Bind> {
    //     let mut binds = Vec::new();
    //     for bg in self.bgs.iter_mut() {
    //         binds.push(PipelineBuilder::build_bind(bg, device));
    //     }
    //     binds
    // }

    fn create_module(&self, device: &Device) -> ShaderModule {
        device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                self.shader_src
                    .as_ref()
                    .expect("Shader source should be set.")
                    .into(),
            ),
        })
    }

    pub fn build(&mut self, render: &Render) -> Pipeline {
        // println!("{:?}", self.primitive_state);

        let bgls = self
            .binds
            .iter()
            .map(|handle| &render.get_bind(*handle).unwrap().bgl)
            .collect::<Vec<_>>();

        let pipeline_layout = render
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: bgls.as_slice(),
                push_constant_ranges: &[],
            });

        let module = self.create_module(render.device());

        let vbs = self
            .vertex_entries
            .iter()
            .map(|ent| ent.layout())
            .collect::<Vec<_>>();

        let pipeline = render
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &module,
                    entry_point: "vertex",
                    buffers: vbs.as_slice(),
                },
                primitive: self.primitive_state,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(),     // 2.
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: "fragment",
                    targets: &[Some(ColorTargetState {
                        format: self.format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::all(),
                    })],
                }),
                multiview: None,
            });

        Pipeline {
            pipeline,
            binds: take(&mut self.binds),
        }
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
