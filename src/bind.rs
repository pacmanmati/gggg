use std::num::NonZeroU32;

use generational_arena::Index;
use itertools::Itertools;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferBinding, BufferDescriptor, BufferUsages, Device, Extent3d,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, StorageTextureAccess, Texture,
    TextureDescriptor, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout,
    VertexStepMode,
};

#[derive(Clone, Copy)]
pub struct BindHandle(pub Index);

#[derive(Clone)]
pub enum BindEntryType<'a> {
    BufferUniform {
        size: u64,
        usages: BufferUsages,
    },
    BufferStorage {
        size: u64,
        read_only: bool,
        usages: BufferUsages,
    },
    Sampler {
        binding_type: SamplerBindingType,
        descriptor: SamplerDescriptor<'a>,
    },
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

impl BindEntryResource {
    pub fn buffer(&self) -> &Buffer {
        match self {
            BindEntryResource::Buffer(buffer) => buffer,
            _ => unreachable!(),
        }
    }
    pub fn sampler(&self) -> &Sampler {
        match self {
            BindEntryResource::Sampler(sampler) => sampler,
            _ => unreachable!(),
        }
    }
    pub fn texture_view(&self) -> (&Texture, &TextureView) {
        match self {
            BindEntryResource::Texture(texture, view) => (texture, view),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct BindEntry<'a> {
    pub visibility: ShaderStages,
    pub ty: BindEntryType<'a>,
    pub count: Option<NonZeroU32>,
    // Pass as None. This will be removed.
    // pub resource: Option<BindEntryResource>,
}

impl<'a> BindEntry<'a> {
    pub fn layout_entry(&self, binding: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: match &self.ty {
                BindEntryType::BufferUniform { .. } => wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                BindEntryType::BufferStorage { read_only, .. } => wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: *read_only,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                BindEntryType::Sampler {
                    binding_type,
                    descriptor,
                } => wgpu::BindingType::Sampler(*binding_type),
                BindEntryType::Texture {
                    sample_type,
                    view_dimension,
                    sample_count,
                    ..
                } => wgpu::BindingType::Texture {
                    sample_type: *sample_type,
                    view_dimension: *view_dimension,
                    multisampled: *sample_count != 1,
                },
                BindEntryType::StorageTexture {
                    access,
                    format,
                    view_dimension,
                    ..
                } => wgpu::BindingType::StorageTexture {
                    access: *access,
                    format: *format,
                    view_dimension: *view_dimension,
                },
            },
            count: self.count,
        }
    }

    pub fn group_entry<'b>(
        &'b self,
        binding: u32,
        // device: &Device,
        resource: &'b BindEntryResource,
    ) -> BindGroupEntry {
        let binding_resource = match &self.ty {
            BindEntryType::BufferUniform { size, usages }
            | BindEntryType::BufferStorage { size, usages, .. } => {
                wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: resource.buffer(),
                    offset: 0,
                    size: None,
                })
            }
            BindEntryType::Sampler {
                binding_type,
                descriptor,
            } => wgpu::BindingResource::Sampler(resource.sampler()),
            BindEntryType::Texture {
                view_dimension,
                sample_count,
                format,
                size,
                usage,
                ..
            } => wgpu::BindingResource::TextureView(resource.texture_view().1),
            BindEntryType::StorageTexture {
                format,
                view_dimension,
                sample_count,
                size,
                usage,
                ..
            } => wgpu::BindingResource::TextureView(resource.texture_view().1),
        };

        BindGroupEntry {
            binding,
            resource: binding_resource,
        }
    }

    pub fn sampler(&self, device: &Device, sampler_descriptor: SamplerDescriptor) -> Sampler {
        device.create_sampler(&sampler_descriptor)
    }

    pub fn buffer(&self, device: &Device, size: u64, usage: BufferUsages) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    pub fn texture(
        &self,
        device: &Device,
        size: Extent3d,
        sample_count: u32,
        view_dimension: TextureViewDimension,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> (Texture, TextureView) {
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
        (texture, view)
    }

    pub fn binding_resource(&self, device: &Device) -> BindEntryResource {
        match &self.ty {
            BindEntryType::BufferUniform { size, usages }
            | BindEntryType::BufferStorage { size, usages, .. } => {
                BindEntryResource::Buffer(self.buffer(device, *size, *usages))
            }
            BindEntryType::Sampler { descriptor, .. } => {
                BindEntryResource::Sampler(self.sampler(device, descriptor.clone()))
            }
            BindEntryType::Texture {
                view_dimension,
                sample_count,
                format,
                size,
                usage,
                ..
            } => {
                let (texture, view) = self.texture(
                    device,
                    *size,
                    *sample_count,
                    *view_dimension,
                    *format,
                    *usage,
                );
                BindEntryResource::Texture(texture, view)
            }
            BindEntryType::StorageTexture {
                format,
                view_dimension,
                sample_count,
                size,
                usage,
                ..
            } => {
                let (texture, view) = self.texture(
                    device,
                    *size,
                    *sample_count,
                    *view_dimension,
                    *format,
                    *usage,
                );
                BindEntryResource::Texture(texture, view)
            }
        }
    }
}

pub struct Bind<'a> {
    pub bg: Option<BindGroup>,
    pub bgl: BindGroupLayout,
    pub resources: Vec<BindEntryResource>,
    pub bind_entries: Vec<BindEntry<'a>>,
}

impl<'a> Bind<'a> {
    pub fn new(mut bind_entries: Vec<BindEntry<'a>>, device: &Device) -> Self {
        let layout_entries = bind_entries
            .iter()
            .enumerate()
            .map(|(idx, g)| g.layout_entry(idx as u32))
            .collect::<Vec<_>>();

        let bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &layout_entries,
        });
        let resources = bind_entries
            .iter_mut()
            .map(|g| g.binding_resource(device))
            .collect_vec();

        let mut bind = Self {
            bg: None,
            bgl,
            resources,
            bind_entries,
        };

        bind.create_bind_group(device);
        bind
    }

    pub fn create_bind_group(&mut self, device: &Device) {
        let group_entries = self
            .bind_entries
            .iter_mut()
            .enumerate()
            .map(|(idx, g)| g.group_entry(idx as u32, self.resources.get(idx).unwrap()))
            .collect::<Vec<_>>();
        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bgl,
            entries: &group_entries,
        });
        self.bg = Some(bg);
    }

    pub fn replace_resource(
        &mut self,
        new_resource: BindEntryResource,
        binding: u32,
        device: &Device,
    ) {
        let _ = std::mem::replace(
            self.resources.get_mut(binding as usize).unwrap(),
            new_resource,
        );

        self.create_bind_group(device);
    }
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
