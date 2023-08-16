use std::collections::HashMap;

use anyhow::{anyhow, Result};

use generational_arena::{Arena, Index};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, Buffer, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, Device,
    DeviceDescriptor, Extent3d, ImageCopyTexture, ImageDataLayout, Instance, Operations, Origin3d,
    Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    atlas::{Atlas, RectHandle},
    bind::{Bind, BindEntry, BindEntryResource, BindHandle},
    geometry::Geometry,
    instance::InstanceData,
    material::Material,
    pipeline::{Pipeline, PipelineHandle},
    render_object::RenderObject,
    texture::Texture,
};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct MeshAndPipelineHandleComposite(MeshHandle, PipelineHandle);

// renderer draws meshes
pub struct Render {
    adapter: Adapter,
    device: Option<Device>,
    queue: Queue,
    surface: Surface,
    pipelines: Arena<Pipeline>,
    binds: Arena<Bind>,
    meshes: Arena<(
        Mesh<Box<dyn Geometry>, Box<dyn Material>>,
        Buffer,         // vertex
        Option<Buffer>, // index
    )>,
    textures: Arena<Texture>,
    atlas_bind: Option<(BindHandle, u32)>,
    atlas: Atlas,
    rect_to_tex: HashMap<RectHandle, TextureHandle>,
    instances: HashMap<
        MeshHandle,
        (
            Vec<Box<dyn InstanceData>>,
            Buffer, // instance
        ),
    >,
    // we're in giga type hell now
    render_objects: HashMap<
        MeshAndPipelineHandleComposite,
        (
            Vec<
                Box<
                    dyn RenderObject<
                        InstanceType = Box<dyn InstanceData>,
                        GeometryType = Box<dyn Geometry>,
                        MaterialType = Box<dyn Material>,
                    >,
                >,
            >,
            Buffer, // instance
        ),
    >,
    depth_texture: wgpu::Texture,
}

impl Render {
    fn replace_resource(&mut self, resource: BindEntryResource, handle: BindHandle, binding: u32) {
        let device = self.device.take();
        let bind = self.get_bind_mut(handle).unwrap();
        bind.replace_resource(resource, binding, device.as_ref().unwrap());
        self.device = device;
    }

    pub fn write_buffer(&mut self, data: &[u8], handle: BindHandle, binding: u32) {
        let bind = self.get_bind(handle).unwrap();
        let resource = bind.resources.get(binding as usize).unwrap();
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
        let bind = self.get_bind(handle).unwrap();
        let resource = bind.resources.get(binding as usize).unwrap();

        let texture = match resource {
            BindEntryResource::Texture(texture, ..) => texture,
            _ => unreachable!(),
        };

        // offset is in bytes (four bytes represents one pixel in the case of rgba8)
        // TODO: support different texture types
        let x = data_layout.offset * 4 % data_layout.bytes_per_row.unwrap() as u64;
        let y = data_layout.offset * 4 / data_layout.bytes_per_row.unwrap() as u64;
        let overflow_x = x as i64 + size.width as i64 - texture.width() as i64;
        let overflow_y = y as i64 + size.height as i64 - texture.height() as i64;

        if overflow_x > 0 || overflow_y > 0 {
            let new_size = Extent3d {
                width: texture.width().max(texture.width() + overflow_x as u32),
                height: texture.height().max(texture.height() + overflow_y as u32),
                depth_or_array_layers: texture.depth_or_array_layers(),
            };
            let descriptor = TextureDescriptor {
                label: None,
                size: new_size,
                mip_level_count: texture.mip_level_count(),
                sample_count: texture.sample_count(),
                dimension: texture.dimension(),
                format: texture.format(),
                usage: texture.usage(),
                view_formats: &[],
            };

            let new_texture =
                self.device()
                    .create_texture_with_data(&self.queue, &descriptor, data);

            let view = new_texture.create_view(&TextureViewDescriptor::default());
            let resource = BindEntryResource::Texture(new_texture, view);
            self.replace_resource(resource, handle, binding);
        } else {
            self.queue
                .write_texture(texture.as_image_copy(), data, data_layout, size);
        }
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
                .ok_or(anyhow!("No suitable adapter found."))?;

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
            device: Some(device),
            queue,
            surface,
            binds: Arena::new(),
            pipelines: Arena::new(),
            meshes: Arena::new(),
            textures: Arena::new(),
            atlas_bind: None,
            atlas: Atlas::new(),
            rect_to_tex: HashMap::new(),
            instances: HashMap::new(),
            render_objects: HashMap::new(),
            depth_texture,
        })
    }

    pub fn add_pipeline(&mut self, pipeline: Pipeline) -> PipelineHandle {
        PipelineHandle(self.pipelines.insert(pipeline))
    }

    pub fn get_pipeline(&self, handle: PipelineHandle) -> Result<&Pipeline> {
        self.pipelines
            .get(handle.0)
            .ok_or(anyhow!("No pipeline found at index {:?}.", handle))
    }

    pub fn get_pipeline_mut(&mut self, handle: PipelineHandle) -> Result<&mut Pipeline> {
        self.pipelines
            .get_mut(handle.0)
            .ok_or(anyhow!("No pipeline found at index {:?}.", handle))
    }

    /// Returns [x, y, x, y] for top left and top right. These values are fractional and represent where this texture is contained on the atlas_texture.
    pub fn get_atlas_coords_for_texture(&self, texture_handle: TextureHandle) -> [f32; 4] {
        let tex_to_rect = self
            .rect_to_tex
            .iter()
            .map(|(rect, tex)| (tex, rect))
            .collect::<HashMap<_, _>>();
        let rect_handle = **tex_to_rect.get(&texture_handle).unwrap();
        let rect = self.atlas.get_rect(rect_handle).unwrap();
        [
            rect.x as f32 / self.atlas.width as f32,
            rect.y as f32 / self.atlas.height as f32,
            (rect.x as f32 + rect.w as f32) / self.atlas.width as f32,
            (rect.y as f32 + rect.h as f32) / self.atlas.height as f32,
        ]
    }

    pub fn get_texture() -> Result<()> {
        // TODO
        Ok(())
    }

    pub fn add_texture(&mut self, texture: Texture) -> TextureHandle {
        // whenever a texture gets added, we want to stitch it into the texture atlas and remember where it goes
        // when we expand the material definition, it'll be able to reference textures via handle
        let rect_handle = self.atlas.add(texture.width(), texture.height());
        let texture_handle = TextureHandle(self.textures.insert(texture));
        self.rect_to_tex.insert(rect_handle, texture_handle);
        self.atlas.pack();
        texture_handle
    }

    pub fn get_mesh(
        &self,
        mesh_handle: MeshHandle,
    ) -> Result<&(
        Mesh<Box<dyn Geometry>, Box<dyn Material>>,
        Buffer,
        Option<Buffer>,
    )> {
        self.meshes
            .get(mesh_handle.0)
            .ok_or(anyhow!("Mesh not found for handle id {:?}", mesh_handle.0))
    }

    pub fn add_mesh<G: Geometry + 'static, I: InstanceData, M: Material + 'static>(
        &mut self,
        mesh: Mesh<G, M>,
    ) -> MeshHandle {
        let buffer = self
            .device
            .as_ref()
            .unwrap()
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: mesh.geometry.contents(),
                // contents: mesh.geometry.contents().as_ref(),
                usage: BufferUsages::VERTEX,
            });
        let index_buffer = if let Some(indices) = mesh.geometry.indices() {
            let index_buffer =
                self.device
                    .as_ref()
                    .unwrap()
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: indices,
                        usage: BufferUsages::INDEX,
                    });
            Some(index_buffer)
        } else {
            None
        };
        MeshHandle(self.meshes.insert((mesh.boxed(), buffer, index_buffer)))
    }

    fn add_instance<T: InstanceData + 'static>(&mut self, mesh_handle: MeshHandle, instance: T) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.instances.entry(mesh_handle) {
            // create hashmap entry
            let new_buffer = self
                .device
                .as_ref()
                .unwrap()
                .create_buffer(&BufferDescriptor {
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
                let new_buffer = self
                    .device
                    .as_ref()
                    .unwrap()
                    .create_buffer(&BufferDescriptor {
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

    // a render object encapsulates all the information we need, including instance data
    // one problem: we usually write the instance data in a buffer immediately. now we have instance data that can change (needs to be determined dynamically)
    // this was always going to be the case. note that eventually we were going to make objects non-persistent in renderer storage, e.g. the instance data would be getting written to a buffer each frame (or so) anyway.
    pub fn add_render_object<R: RenderObject + 'static>(&mut self, render_object: R) {
        let instance = render_object.instance(self);
        let mesh_handle = render_object.mesh_handle();
        let pipeline_handle = render_object.pipeline_handle();
        let key = MeshAndPipelineHandleComposite(mesh_handle, pipeline_handle);
        // self.add_instance(render_object.mesh_handle(), instance);

        if let std::collections::hash_map::Entry::Vacant(e) = self.render_objects.entry(key) {
            // create hashmap entry
            let new_buffer = self
                .device
                .as_ref()
                .unwrap()
                .create_buffer(&BufferDescriptor {
                    label: Some("Instance buffer"),
                    size: std::mem::size_of::<R::InstanceType>() as u64 * 10,
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            let new_data = instance.data();
            self.queue.write_buffer(&new_buffer, 0, new_data);
            e.insert((vec![Box::new(render_object.boxed())], new_buffer));
        } else {
            let (instances, buffer) = self.render_objects.get_mut(&key).unwrap();
            if buffer.size() < std::mem::size_of::<R::InstanceType>() as u64 {
                // create a bigger buffer
                let new_buffer = self
                    .device
                    .as_ref()
                    .unwrap()
                    .create_buffer(&BufferDescriptor {
                        label: Some("Instance buffer"),
                        size: buffer.size() + std::mem::size_of::<R::InstanceType>() as u64 * 10,
                        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                instances.push(Box::new(render_object.boxed()));
                let (instances, buffer) = self.render_objects.get(&key).unwrap();
                let new_data = instances.iter().fold(Vec::new(), |mut acc, instance| {
                    acc.extend_from_slice(instance.instance(self).data());
                    acc
                });
                self.queue.write_buffer(&new_buffer, 0, new_data.as_slice());
                self.instances.entry(mesh_handle).and_modify(|(_, buffer)| {
                    let _ = std::mem::replace(buffer, new_buffer);
                });

                return;
            }
            let offset = instances.len() * std::mem::size_of::<R::InstanceType>();
            // write this instance into the buffer, no need to resize
            self.queue
                .write_buffer(buffer, offset as u64, instance.data());
            instances.push(Box::new(render_object.boxed()));
        }
    }

    pub fn device(&self) -> &Device {
        self.device.as_ref().unwrap()
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn set_atlas(&mut self, handle: BindHandle, binding: u32) {
        self.atlas_bind = Some((handle, binding));
    }

    pub fn build_bind(&mut self, bind_entries: &mut [BindEntry]) -> BindHandle {
        let bind = Bind::new(bind_entries.to_vec(), self.device.as_ref().unwrap());
        self.add_bind(bind)
    }

    fn add_bind(&mut self, bind: Bind) -> BindHandle {
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

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface.configure(
            self.device(),
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: *self
                    .surface
                    .get_capabilities(&self.adapter)
                    .formats
                    .first()
                    .unwrap(),
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );

        self.depth_texture = self.device().create_texture(&TextureDescriptor {
            label: Some("depth texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
    }

    pub fn draw(&mut self) {
        if let Some((atlas_bind, binding)) = self.atlas_bind {
            if self.atlas.changed {
                self.atlas.pack();
                self.atlas.changed = false;
                // update the atlas texture
                let atlas_texture =
                    Texture::from_atlas(&self.atlas, &self.rect_to_tex, &self.textures);
                // let _ = image::save_buffer(
                //     "atlas.png",
                //     &atlas_texture.data,
                //     atlas_texture.width,
                //     atlas_texture.height,
                //     image::ColorType::Rgba8,
                // );
                self.write_texture(
                    &atlas_texture.data,
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * atlas_texture.width),
                        rows_per_image: None,
                    },
                    Extent3d {
                        width: atlas_texture.width,
                        height: atlas_texture.height,
                        depth_or_array_layers: 1,
                    },
                    atlas_bind,
                    binding,
                );
            }
        }

        let frame = self.surface.get_current_texture().unwrap();

        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .as_ref()
            .unwrap()
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let depth_texture_view = &self
            .depth_texture
            .create_view(&TextureViewDescriptor::default());

        // so now i think of draw map as a hashmap of hashmaps
        // because the hierarchy goes like this (to minimise state changes)
        // bind pipeline -> bind vertex/index buffer -> draw instances for some buffer
        // which means we want to do a HashMap<PipelineHandle, HashMap<MeshHandle, (num_instances, instance_buffer)>>
        let mut draw_map: HashMap<PipelineHandle, HashMap<MeshHandle, (u32, &Buffer)>> =
            HashMap::new();

        for (key, (render_objects, buffer)) in &self.render_objects {
            draw_map
                .entry(key.1)
                .or_insert(HashMap::new())
                .insert(key.0, (render_objects.len() as u32, buffer));
        }

        let mut rpass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color {
                        r: 0.05,
                        g: 0.05,
                        b: 0.05,
                        a: 1.0,
                        // r: 0.9,
                        // g: 0.7,
                        // b: 0.7,
                        // a: 1.0,
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

        for (pipeline_handle, meshes_and_render_objects) in draw_map.iter() {
            let pipeline = self.pipelines.get(pipeline_handle.0).unwrap();
            rpass.set_pipeline(&pipeline.pipeline);
            for (idx, handle) in pipeline.binds.iter().enumerate() {
                let bind = self.get_bind(*handle).unwrap();
                let bg = &bind.bg;
                rpass.set_bind_group(idx as u32, bg.as_ref().unwrap(), &[]);
            }

            for (mesh_handle, (num_instances, instance_buffer)) in meshes_and_render_objects {
                // get the mesh
                let (mesh, vertex_buffer, index_buffer) =
                    self.get_mesh(*mesh_handle).expect("Mesh should exist");

                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.set_vertex_buffer(1, instance_buffer.slice(..));
                // println!("drawing");
                if let Some(index_buffer) = index_buffer {
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    rpass.draw_indexed(
                        0..(index_buffer.size() as u32 / std::mem::size_of::<u16>() as u32),
                        0,
                        0..*num_instances,
                    )
                } else {
                    rpass.draw(0..mesh.geometry.length(), 0..*num_instances);
                }
            }
        }

        drop(rpass);

        self.queue.submit([encoder.finish()]);

        frame.present();

        self.render_objects.clear();
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct MeshHandle(pub Index);

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct TextureHandle(pub Index);

pub struct Mesh<G: Geometry, M: Material> {
    pub material: M,
    pub geometry: G,
}

impl<T: Geometry + 'static, M: Material + 'static> Mesh<T, M> {
    pub fn boxed(self) -> Mesh<Box<dyn Geometry>, Box<dyn Material>> {
        Mesh {
            material: Box::new(self.material),
            geometry: Box::new(self.geometry),
        }
    }
}
