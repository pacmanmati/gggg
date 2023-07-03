use std::num::NonZeroU32;

use generational_arena::Index;
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, Buffer, BufferBinding,
    BufferDescriptor, BufferUsages, Device, Extent3d, Sampler, SamplerBindingType,
    SamplerDescriptor, ShaderStages, StorageTextureAccess, Texture, TextureDescriptor,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexStepMode,
};

#[derive(Clone, Copy)]
pub struct BindHandle(pub Index);

pub enum BindEntryType {
    BufferUniform {
        size: u64,
        usages: BufferUsages,
    },
    BufferStorage {
        size: u64,
        read_only: bool,
        usages: BufferUsages,
    },
    Sampler(SamplerBindingType),
    Texture {
        sample_type: TextureSampleType,
        view_dimension: TextureViewDimension,
        sample_count: u32,
        format: TextureFormat,
        size: Extent3d,
        usage: TextureUsages,
    },
    StorageTexture {
        access: StorageTextureAccess,
        format: TextureFormat,
        view_dimension: TextureViewDimension,
        sample_count: u32,
        size: Extent3d,
        usage: TextureUsages,
    },
}

pub enum BindEntryResource {
    Buffer(Buffer),
    Texture(Texture, TextureView),
    Sampler(Sampler),
}

pub struct BindEntry {
    pub visibility: ShaderStages,
    pub ty: BindEntryType,
    pub count: Option<NonZeroU32>,
    /// Pass as None. This will be removed.
    pub resource: Option<BindEntryResource>,
}

impl BindEntry {
    pub fn layout_entry(&self, binding: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: match self.ty {
                BindEntryType::BufferUniform { .. } => wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                BindEntryType::BufferStorage { read_only, .. } => wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                BindEntryType::Sampler(ty) => wgpu::BindingType::Sampler(ty),
                BindEntryType::Texture {
                    sample_type,
                    view_dimension,
                    sample_count,
                    format,
                    size,
                    usage,
                } => wgpu::BindingType::Texture {
                    sample_type,
                    view_dimension,
                    multisampled: sample_count != 1,
                },
                BindEntryType::StorageTexture {
                    access,
                    format,
                    view_dimension,
                    sample_count,
                    size,
                    usage,
                } => wgpu::BindingType::StorageTexture {
                    access,
                    format,
                    view_dimension,
                },
            },
            count: self.count,
        }
    }

    pub fn group_entry(&mut self, binding: u32, device: &Device) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: match self.ty {
                BindEntryType::BufferUniform { size, usages }
                | BindEntryType::BufferStorage { size, usages, .. } => {
                    let buffer = self.buffer(device, size, usages);
                    wgpu::BindingResource::Buffer(BufferBinding {
                        buffer,
                        offset: 0,
                        size: None,
                    })
                }
                BindEntryType::Sampler(ty) => {
                    let sampler = self.sampler(device);
                    wgpu::BindingResource::Sampler(sampler)
                }
                BindEntryType::Texture {
                    sample_type,
                    view_dimension,
                    sample_count,
                    format,
                    size,
                    usage,
                } => {
                    let (texture, view) =
                        self.texture(device, size, sample_count, view_dimension, format, usage);

                    wgpu::BindingResource::TextureView(view)
                }
                BindEntryType::StorageTexture {
                    access,
                    format,
                    view_dimension,
                    sample_count,
                    size,
                    usage,
                } => {
                    let (texture, view) =
                        self.texture(device, size, sample_count, view_dimension, format, usage);

                    wgpu::BindingResource::TextureView(view)
                }
            },
        }
    }

    pub fn sampler(&mut self, device: &Device) -> &Sampler {
        if let Some(ref resource) = self.resource {
            match resource {
                BindEntryResource::Sampler(sampler) => sampler,
                _ => unreachable!(),
            }
        } else {
            let sampler = device.create_sampler(&SamplerDescriptor::default());
            self.resource = Some(BindEntryResource::Sampler(sampler));
            match self.resource.as_ref().unwrap() {
                BindEntryResource::Sampler(sampler) => sampler,
                _ => unreachable!(),
            }
        }
    }

    pub fn buffer(&mut self, device: &Device, size: u64, usage: BufferUsages) -> &Buffer {
        if let Some(ref resource) = self.resource {
            match resource {
                BindEntryResource::Buffer(buffer) => buffer,
                _ => unreachable!(),
            }
        } else {
            let buffer = device.create_buffer(&BufferDescriptor {
                label: None,
                size,
                usage,
                mapped_at_creation: false,
            });
            self.resource = Some(BindEntryResource::Buffer(buffer));
            match self.resource.as_ref().unwrap() {
                BindEntryResource::Buffer(buffer) => buffer,
                _ => unreachable!(),
            }
        }
    }

    pub fn texture(
        &mut self,
        device: &Device,
        size: Extent3d,
        sample_count: u32,
        view_dimension: TextureViewDimension,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> (&Texture, &TextureView) {
        if let Some(ref resource) = self.resource {
            match resource {
                BindEntryResource::Texture(texture, view) => (texture, view),
                _ => unreachable!(),
            }
        } else {
            let texture = device.create_texture(&TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count,
                dimension: view_dimension.compatible_texture_dimension(),
                format,
                usage,
                view_formats: &[],
            });
            let view = texture.create_view(&TextureViewDescriptor::default());
            self.resource = Some(BindEntryResource::Texture(texture, view));
            match self.resource.as_ref().unwrap() {
                BindEntryResource::Texture(sampler, view) => (sampler, view),
                _ => unreachable!(),
            }
        }
    }
}

pub struct Bind {
    pub bg: BindGroup,
    pub bgl: BindGroupLayout,
    pub resources: Vec<BindEntryResource>,
}

pub struct VertexBufferEntry {
    pub array_stride: u64,
    pub step_mode: VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

impl VertexBufferEntry {
    pub fn layout(&self) -> VertexBufferLayout {
        VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: self.attributes.as_slice(),
        }
    }
}
