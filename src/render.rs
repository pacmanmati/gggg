use std::{collections::HashMap, fmt::Debug};

use anyhow::{anyhow, Result};

use generational_arena::{Arena, Index};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, BindGroupDescriptor, BindGroupLayoutDescriptor, Buffer, BufferDescriptor,
    BufferUsages, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d,
    ImageDataLayout, Instance, Operations, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};
use winit::window::Window;

use crate::{
    bind::{Bind, BindEntry, BindEntryResource, BindHandle},
    pipeline::Pipeline,
};

// renderer draws meshes
pub struct Render {
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Surface,
    pipelines: Vec<Pipeline>,
    binds: Arena<Bind>,
    meshes: Arena<(Mesh<Box<dyn Geometry>>, Buffer)>,
    instances: HashMap<MeshHandle, (Vec<Box<dyn InstanceData>>, Buffer)>,
    depth_texture: Texture,
}

impl Render {
    pub fn write_buffer(&mut self, data: &[u8], handle: BindHandle, binding: u32) {
        let resource = self
            .get_bind(handle)
            .unwrap()
            .resources
            .get(binding as usize)
            .unwrap();
        let buffer = match resource {
            BindEntryResource::Buffer(buffer) => buffer,
            _ => unreachable!(),
        };
        self.queue.write_buffer(buffer, 0, data);
    }

    pub fn write_texture(
        &mut self,
        data: &[u8],
        data_layout: ImageDataLayout,
        size: Extent3d,
        handle: BindHandle,
        binding: u32,
    ) {
        let resource = self
            .get_bind(handle)
            .unwrap()
            .resources
            .get(binding as usize)
            .unwrap();

        let texture = match resource {
            BindEntryResource::Texture(texture, ..) => texture.as_image_copy(),
            _ => unreachable!(),
        };

        self.queue.write_texture(texture, data, data_layout, size);
    }

    pub fn new(window: &Window) -> Result<Self> {
        let instance = Instance::default();

        let surface = unsafe { instance.create_surface(window)? };

        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    ..Default::default()
                })
                .await
                .ok_or(anyhow!("asd"))?;

            let (device, queue) = adapter
                .request_device(&DeviceDescriptor::default(), None)
                .await?;

            Ok::<(wgpu::Adapter, wgpu::Device, wgpu::Queue), anyhow::Error>((
                adapter, device, queue,
            ))
        })?;

        surface.configure(
            &device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: *surface
                    .get_capabilities(&adapter)
                    .formats
                    .first()
                    .ok_or(anyhow!("No formats found."))?,
                width: window.inner_size().width,
                height: window.inner_size().height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );

        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("depth texture"),
            size: Extent3d {
                width: window.inner_size().width,
                height: window.inner_size().height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        Ok(Self {
            adapter,
            device,
            queue,
            surface,
            binds: Arena::new(),
            pipelines: Vec::new(),
            meshes: Arena::new(),
            instances: HashMap::new(),
            depth_texture,
        })
    }

    pub fn add_pipeline(&mut self, pipeline: Pipeline) {
        self.pipelines.push(pipeline);
    }

    pub fn get_pipeline(&self, idx: usize) -> Result<&Pipeline> {
        self.pipelines
            .get(idx)
            .ok_or(anyhow!("No pipeline found at index {}.", idx))
    }

    pub fn get_pipeline_mut(&mut self, idx: usize) -> Result<&mut Pipeline> {
        self.pipelines
            .get_mut(idx)
            .ok_or(anyhow!("No pipeline found at index {}.", idx))
    }

    pub fn get_mesh(&self, mesh_handle: MeshHandle) -> Result<&(Mesh<Box<dyn Geometry>>, Buffer)> {
        self.meshes
            .get(mesh_handle.0)
            .ok_or(anyhow!("Mesh not found for handle id {:?}", mesh_handle.0))
    }

    pub fn add_mesh<T: Geometry + 'static, G: InstanceData>(
        &mut self,
        mesh: Mesh<T>,
    ) -> MeshHandle {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: mesh.geometry.contents().as_ref(),
            usage: BufferUsages::VERTEX,
        });
        MeshHandle(self.meshes.insert((mesh.boxed(), buffer)))
    }

    pub fn add_instance<T: InstanceData + 'static>(
        &mut self,
        mesh_handle: MeshHandle,
        instance: T,
    ) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.instances.entry(mesh_handle) {
            // create hashmap entry
            let new_buffer = self.device.create_buffer(&BufferDescriptor {
                label: Some("Instance buffer"),
                size: std::mem::size_of::<T>() as u64 * 10,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let new_data = instance.data();
            self.queue.write_buffer(&new_buffer, 0, new_data);
            e.insert((vec![Box::new(instance)], new_buffer));
        } else {
            let (instances, buffer) = self.instances.get_mut(&mesh_handle).unwrap();
            if buffer.size() < std::mem::size_of::<T>() as u64 {
                // create a bigger buffer
                let new_buffer = self.device.create_buffer(&BufferDescriptor {
                    label: Some("Instance buffer"),
                    size: buffer.size() + std::mem::size_of::<T>() as u64 * 10,
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                instances.push(Box::new(instance));
                let new_data = instances.iter().fold(Vec::new(), |mut acc, instance| {
                    acc.extend_from_slice(instance.data());
                    acc
                });
                self.queue.write_buffer(&new_buffer, 0, new_data.as_slice());
                self.instances.entry(mesh_handle).and_modify(|(_, buffer)| {
                    let _ = std::mem::replace(buffer, new_buffer);
                });

                return;
            }
            let offset = instances.len() * std::mem::size_of::<T>();
            // write this instance into the buffer, no need to resize
            self.queue
                .write_buffer(buffer, offset as u64, instance.data());
            instances.push(Box::new(instance));
        }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn build_bind(&mut self, bg: &mut [BindEntry]) -> BindHandle {
        let layout_entries = bg
            .iter()
            .enumerate()
            .map(|(idx, g)| g.layout_entry(idx as u32))
            .collect::<Vec<_>>();

        let bgl = self
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &layout_entries,
            });

        let group_entries = bg
            .iter_mut()
            .enumerate()
            .map(|(idx, g)| g.group_entry(idx as u32, &self.device))
            .collect::<Vec<_>>();

        let bind_groups = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &group_entries,
        });

        let resources = bg
            .iter_mut()
            .map(|entry| entry.resource.take().unwrap())
            .collect();

        let bind = Bind {
            bg: bind_groups,
            bgl,
            resources,
        };

        self.add_bind(bind)
    }

    pub fn add_bind(&mut self, bind: Bind) -> BindHandle {
        BindHandle(self.binds.insert(bind))
    }

    pub fn get_bind(&self, handle: BindHandle) -> Result<&Bind> {
        self.binds
            .get(handle.0)
            .ok_or(anyhow!("No Bind for handle"))
    }

    pub fn get_bind_mut(&mut self, handle: BindHandle) -> Result<&mut Bind> {
        self.binds
            .get_mut(handle.0)
            .ok_or(anyhow!("No Bind for handle"))
    }

    pub fn draw(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();

        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let depth_texture_view = &self
            .depth_texture
            .create_view(&TextureViewDescriptor::default());

        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color {
                        r: 0.9,
                        g: 0.7,
                        b: 0.7,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        let mut draw_map = HashMap::new();

        for (idx, (mesh, buffer)) in &self.meshes {
            draw_map
                .entry(mesh.material)
                .or_insert(Vec::new())
                .push(MeshHandle(idx));
        }

        for (material, mesh_handles) in draw_map {
            let pipeline = self.pipelines.get(material as usize).unwrap();
            rpass.set_pipeline(&pipeline.pipeline);
            for (idx, handle) in pipeline.binds.iter().enumerate() {
                let bind = self.get_bind(*handle).unwrap();
                rpass.set_bind_group(idx as u32, &bind.bg, &[]);
            }

            for mesh_handle in mesh_handles {
                // get the mesh
                let (mesh, vertex_buffer) = self.get_mesh(mesh_handle).expect("Mesh should exist");
                // let vertex_data = mesh.geometry.contents();

                // get all instances for this mesh handle
                let (instances, instance_buffer) = self.instances.get(&mesh_handle).unwrap();

                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.set_vertex_buffer(1, instance_buffer.slice(..));
                rpass.draw(0..mesh.geometry.length(), 0..instances.len() as u32);
                // println!("{:?}", instances);
            }
        }

        drop(rpass);

        self.queue.submit([encoder.finish()]);

        frame.present();
    }
}

pub trait InstanceData: Debug {
    fn data(&self) -> &[u8];
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct MeshHandle(pub Index);

pub struct Mesh<T: Geometry> {
    pub material: u32,
    pub geometry: T,
}

impl<T: Geometry + 'static> Mesh<T> {
    pub fn boxed(self) -> Mesh<Box<dyn Geometry>> {
        Mesh {
            material: self.material,
            geometry: Box::new(self.geometry),
        }
    }
}

pub trait Geometry {
    fn contents(&self) -> Vec<u8>;

    fn length(&self) -> u32;
}

impl Geometry for Box<dyn Geometry> {
    fn contents(&self) -> Vec<u8> {
        self.as_ref().contents()
    }

    fn length(&self) -> u32 {
        self.as_ref().length()
    }
}