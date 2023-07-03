use std::{f32::consts::FRAC_PI_2, io::Read};

use gggg::{
    bind::{BindEntry, BindEntryType, BindHandle},
    camera::{Camera, ProjectionType},
    pipeline::PipelineBuilder,
    plain::Plain,
    render::{Geometry, InstanceData, Mesh, Render},
    window::{make_window, AppLoop},
};
use image::{io::Reader, EncodableLayout};
use nalgebra::{point, Isometry3, IsometryMatrix3, Matrix4, SimdValue, Translation3, Vector3};
use wgpu::{
    vertex_attr_array, BufferUsages, Extent3d, ImageDataLayout, ShaderStages, TextureUsages,
};
use winit::window::Window;

#[repr(C)]
#[derive(Debug)]
struct Instance {
    transform: Matrix4<f32>,
}

impl InstanceData for Instance {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

unsafe impl Plain for Instance {}

#[repr(C)]
#[derive(Debug)]
struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
    normal: [f32; 3],
}

unsafe impl Plain for Vertex {}

fn v(x: f32, y: f32, z: f32, u: f32, v: f32, nx: f32, ny: f32, nz: f32) -> Vertex {
    Vertex {
        pos: [x, y, z],
        uv: [u, v],
        normal: [nx, ny, nz],
    }
}

struct Cube {
    vertices: Vec<Vertex>,
}

impl Cube {
    pub fn new() -> Self {
        Self {
            vertices: vec![
                // Front face
                v(-0.5, -0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 1.0),
                v(0.5, -0.5, 0.5, 1.0, 0.0, 0.0, 0.0, 1.0),
                v(0.5, 0.5, 0.5, 1.0, 1.0, 0.0, 0.0, 1.0),
                v(0.5, 0.5, 0.5, 1.0, 1.0, 0.0, 0.0, 1.0),
                v(-0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0),
                v(-0.5, -0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 1.0),
                // Right face
                v(0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0),
                v(0.5, -0.5, -0.5, 1.0, 0.0, 1.0, 0.0, 0.0),
                v(0.5, 0.5, -0.5, 1.0, 1.0, 1.0, 0.0, 0.0),
                v(0.5, 0.5, -0.5, 1.0, 1.0, 1.0, 0.0, 0.0),
                v(0.5, 0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0),
                v(0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0),
                // Back face
                v(0.5, -0.5, -0.5, 0.0, 0.0, 0.0, 0.0, -1.0),
                v(-0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, -1.0),
                v(-0.5, 0.5, -0.5, 1.0, 1.0, 0.0, 0.0, -1.0),
                v(-0.5, 0.5, -0.5, 1.0, 1.0, 0.0, 0.0, -1.0),
                v(0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, -1.0),
                v(0.5, -0.5, -0.5, 0.0, 0.0, 0.0, 0.0, -1.0),
                // Left face
                v(-0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0),
                v(-0.5, -0.5, 0.5, 1.0, 0.0, -1.0, 0.0, 0.0),
                v(-0.5, 0.5, 0.5, 1.0, 1.0, -1.0, 0.0, 0.0),
                v(-0.5, 0.5, 0.5, 1.0, 1.0, -1.0, 0.0, 0.0),
                v(-0.5, 0.5, -0.5, 0.0, 1.0, -1.0, 0.0, 0.0),
                v(-0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0),
                // Top face
                v(-0.5, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0, 0.0),
                v(0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0, 0.0),
                v(0.5, 0.5, -0.5, 1.0, 1.0, 0.0, 1.0, 0.0),
                v(0.5, 0.5, -0.5, 1.0, 1.0, 0.0, 1.0, 0.0),
                v(-0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 0.0),
                v(-0.5, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0, 0.0),
                // Bottom face
                v(-0.5, -0.5, -0.5, 0.0, 0.0, 0.0, -1.0, 0.0),
                v(0.5, -0.5, -0.5, 1.0, 0.0, 0.0, -1.0, 0.0),
                v(0.5, -0.5, 0.5, 1.0, 1.0, 0.0, -1.0, 0.0),
                v(0.5, -0.5, 0.5, 1.0, 1.0, 0.0, -1.0, 0.0),
                v(-0.5, -0.5, 0.5, 0.0, 1.0, 0.0, -1.0, 0.0),
                v(-0.5, -0.5, -0.5, 0.0, 0.0, 0.0, -1.0, 0.0),
            ],
        }
    }
}

impl Geometry for Cube {
    fn contents(&self) -> Vec<u8> {
        let slice = self.vertices.as_slice();
        let bytes = slice.as_bytes();
        Vec::from(bytes)
    }

    fn length(&self) -> u32 {
        self.vertices.len() as u32
    }
}

#[repr(C)]
struct LightUniform {
    position: [f32; 4],
    color: [f32; 4],
}

unsafe impl Plain for LightUniform {}

#[repr(C)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    position: [f32; 3],
    padding: u32,
}

unsafe impl Plain for CameraUniform {}

struct App {
    render: Render,
    camera: Camera,
    rot_y: f32,
    rot_x: f32,
    camera_distance: f32,
    camera_bind: BindHandle,
}

impl App {
    pub fn update_camera(&mut self) {
        let x = self.camera_distance * self.rot_y.cos();
        let z = self.camera_distance * self.rot_y.sin();
        let y = self.camera_distance * self.rot_x.sin();
        self.camera.eye = point![x, y, z];

        // let camera_data = self.camera.view_projection();

        // let camera_slice = camera_data.as_slice();
        // let camera_bytes = camera_slice.as_bytes();

        let camera_data = CameraUniform {
            view_proj: self.camera.view_projection().into(),
            position: self.camera.eye.into(),
            padding: 0,
        };

        self.render
            .write_buffer(camera_data.as_bytes(), self.camera_bind, 0);
    }

    pub fn zoom_camera(&mut self, delta: (f32, f32)) {
        self.camera_distance -= delta.1 * 0.01;
        self.update_camera();
    }

    pub fn move_camera(&mut self, delta: (f32, f32)) {
        self.rot_y += delta.0 * 0.01;
        self.rot_x += delta.1 * 0.01;

        self.update_camera();
    }
}

impl AppLoop for App {
    type App = Self;

    fn init(window: &Window) -> App {
        let mut render = Render::new(window).unwrap();

        let camera = Camera::new(
            point![1.0, 0.0, 5.0],
            point![0.0, 0.0, 0.0],
            ProjectionType::Perspective {
                aspect: window.inner_size().width as f32 / window.inner_size().height as f32,
                fovy: 70.0,
                near: 0.1,
                far: 100.0,
            },
        );

        let camera_data = CameraUniform {
            view_proj: camera.view_projection().into(),
            position: camera.eye.into(),
            padding: 0,
        };

        let camera_bind_handle = render.build_bind(&mut [BindEntry {
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindEntryType::BufferUniform {
                size: std::mem::size_of_val(&camera_data) as u64,
                usages: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },

            count: None,
            resource: None,
        }]);

        let readimg = Reader::open("cobble.png").unwrap().decode().unwrap();
        let img = readimg.to_rgba8();

        let texture_bind_handle = render.build_bind(&mut [BindEntry {
            visibility: ShaderStages::FRAGMENT,
            ty: BindEntryType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_count: 1,
                format: wgpu::TextureFormat::Rgba8Unorm,
                size: wgpu::Extent3d {
                    width: img.width(),
                    height: img.height(),
                    depth_or_array_layers: 1,
                },
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
            count: None,
            resource: None,
        }]);

        let sampler_bind_handle = render.build_bind(&mut [BindEntry {
            visibility: ShaderStages::FRAGMENT,
            ty: BindEntryType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
            resource: None,
        }]);

        let lights = vec![
            LightUniform {
                position: [1.0, 1.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            LightUniform {
                position: [2.0, -5.0, 2.0, 1.0],
                color: [0.5, 0.1, 0.1, 1.0],
            },
            LightUniform {
                position: [-2.0, 0.0, -2.0, 1.0],
                color: [0.5, 1.0, 0.1, 1.0],
            },
        ];

        let light_data = lights.as_slice();

        let lights_bind_handle = render.build_bind(&mut [BindEntry {
            visibility: ShaderStages::FRAGMENT,
            ty: BindEntryType::BufferStorage {
                size: std::mem::size_of_val(light_data) as u64,
                usages: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                read_only: true,
            },
            // ty: BindEntryType::BufferUniform {
            //     size: std::mem::size_of_val(light_data) as u64,
            //     usages: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            // },
            count: None,
            resource: None,
        }]);

        render.write_buffer(light_data.as_bytes(), lights_bind_handle, 0);

        let pipeline = PipelineBuilder::new()
            .with_cull_mode(Some(wgpu::Face::Back))
            .with_bind(camera_bind_handle)
            .with_bind(texture_bind_handle)
            .with_bind(sampler_bind_handle)
            .with_bind(lights_bind_handle)
            .with_format(wgpu::TextureFormat::Bgra8UnormSrgb)
            .with_shader(include_str!("shader.wgsl"))
            .with_vb::<Vertex>(
                wgpu::VertexStepMode::Vertex,
                &vertex_attr_array![0 => Float32x3, 1=> Float32x2, 2 => Float32x3],
            )
            .with_vb::<Instance>(
                wgpu::VertexStepMode::Instance,
                &vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4 ],
            )
            .build(&render);

        let camera_bytes = camera_data.as_bytes();

        render.write_buffer(camera_bytes, camera_bind_handle, 0);

        render.write_texture(
            img.as_bytes(),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(img.width() * 4),
                rows_per_image: None,
            },
            Extent3d {
                width: img.width(),
                height: img.height(),
                depth_or_array_layers: 1,
            },
            texture_bind_handle,
            0,
        );

        render.add_pipeline(pipeline);

        let cube_mesh = Mesh {
            material: 0,
            geometry: Cube::new(),
        };

        let cube_handle = render.add_mesh::<Cube, Instance>(cube_mesh);

        render.add_instance(
            cube_handle,
            Instance {
                transform: Translation3::new(0.0, 0.0, 0.0).to_homogeneous(),
            },
        );

        Self {
            render,
            camera,
            rot_y: 0.0,
            rot_x: 0.0,
            camera_distance: 5.0,
            camera_bind: camera_bind_handle,
        }
    }

    fn draw(&mut self) {
        self.render.draw();
    }

    fn input(&mut self, input: gggg::input::InputEvent) {
        // println!("{:?}", input);
        match input {
            gggg::input::InputEvent::MouseInput(input) => match input {
                // gggg::input::MouseInputEvent::MouseMovement { delta } => self.move_camera(delta),
                gggg::input::MouseInputEvent::MouseScroll { delta } => self.move_camera(delta),
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() {
    make_window().with_title("hello").run(App::init);
}