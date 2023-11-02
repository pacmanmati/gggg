use gggg::{
    bind::{
        vertex_attr_array, BindEntry, BindEntryType, BindHandle, BufferUsages, Extent3d, Face,
        SamplerBindingType, SamplerDescriptor, ShaderStages, TextureFormat, TextureSampleType,
        TextureUsages, TextureViewDimension, VertexStepMode,
    },
    camera::{Camera, ProjectionType},
    geometry::Geometry,
    instance::InstanceData,
    material::BasicMaterial,
    pipeline::{PipelineBuilder, PipelineHandle},
    plain::Plain,
    render::{AtlasHandle, Mesh, MeshHandle, PhysicalSize, Render, TextureHandle, Window},
    render_object::BasicRenderObject,
    texture::Texture,
    window::{make_window, AppLoop},
};
use nalgebra::{point, Matrix4, Translation3, Vector4};

#[repr(C)]
#[derive(Debug)]
struct Instance {
    transform: Matrix4<f32>,
    atlas_coords: Vector4<u32>,
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

#[derive(Debug)]
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
    fn contents(&self) -> &[u8] {
        self.vertices.as_bytes()
    }

    fn length(&self) -> u32 {
        self.vertices.len() as u32
    }

    fn indices(&self) -> Option<&[u8]> {
        None
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

// struct CustomRenderObject {}

// impl RenderObject for CustomRenderObject {
//     type InstanceType = Instance;

//     type GeometryType = Cube;

//     fn instance_data(render: &Render, mesh: Mesh<Self::GeometryType>) -> Self::InstanceType {
//         todo!()
//     }
// }

struct App<'a> {
    render: Render<'a>,
    camera: Camera,
    rot_y: f32,
    rot_x: f32,
    camera_distance: f32,
    defaults_bind: BindHandle,
    pipeline_handle: PipelineHandle,
    cube_handle: MeshHandle,
    cobble_handle: TextureHandle,
    stone_handle: TextureHandle,
    atlas_handle: AtlasHandle,
}

impl<'a> App<'a> {
    pub fn update_camera(&mut self) {
        let x = self.camera_distance * self.rot_y.cos();
        let z = self.camera_distance * self.rot_y.sin();
        let y = self.camera_distance * self.rot_x.sin();
        self.camera.eye = point![x, y, z];

        let camera_data = CameraUniform {
            view_proj: self.camera.view_projection().into(),
            position: self.camera.eye.into(),
            padding: 0,
        };

        self.render
            .write_buffer(camera_data.as_bytes(), self.defaults_bind, 0);
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

impl<'a> AppLoop for App<'a> {
    type App = Self;

    fn init(window: &Window) -> App<'a> {
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

        let defaults_bind = render.build_bind(&mut [
            // camera
            BindEntry {
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindEntryType::BufferUniform {
                    size: std::mem::size_of_val(&camera_data) as u64,
                    usages: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                },

                count: None,
            },
            // texture atlas
            BindEntry {
                visibility: ShaderStages::FRAGMENT,
                ty: BindEntryType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    sample_count: 1,
                    format: TextureFormat::Rgba8Unorm,
                    size: Extent3d {
                        width: 512,
                        height: 100,
                        depth_or_array_layers: 1,
                    },
                    usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                },
                count: None,
            },
        ]);

        let atlas_handle =
            render.register_atlas(defaults_bind, 1, gggg::texture::TextureFormat::Rgba8Unorm);

        let cobble_tex = Texture::from_path("cobble.png");
        let stone_tex = Texture::from_path("stone.png");

        let cobble_handle = render.add_texture(cobble_tex, atlas_handle).unwrap();
        let stone_handle = render.add_texture(stone_tex, atlas_handle).unwrap();

        let sampler_bind_handle = render.build_bind(&mut [BindEntry {
            visibility: ShaderStages::FRAGMENT,
            ty: BindEntryType::Sampler {
                binding_type: SamplerBindingType::NonFiltering,
                descriptor: SamplerDescriptor::default(),
            },
            count: None,
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
        }]);

        render.write_buffer(light_data.as_bytes(), lights_bind_handle, 0);

        let pipeline = PipelineBuilder::new()
            .with_cull_mode(Some(Face::Back))
            .with_bind(defaults_bind)
            .with_bind(sampler_bind_handle)
            .with_bind(lights_bind_handle)
            .with_format(TextureFormat::Bgra8UnormSrgb)
            .with_shader(include_str!("shader.wgsl"))
            .with_vb::<Vertex>(
                VertexStepMode::Vertex,
                &vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3],
            )
            .with_vb::<Instance>(
                VertexStepMode::Instance,
                &vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4],
            )
            .build(&render);

        let camera_bytes = camera_data.as_bytes();

        render.write_buffer(camera_bytes, defaults_bind, 0);

        // render.write_texture(
        //     img.as_bytes(),
        //     ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: Some(img.width() * 4),
        //         rows_per_image: None,
        //     },
        //     Extent3d {
        //         width: img.width(),
        //         height: img.height(),
        //         depth_or_array_layers: 1,
        //     },
        //     texture_bind_handle,
        //     0,
        // );

        let pipeline_handle = render.add_pipeline(pipeline);

        let cube_mesh = Mesh {
            material: BasicMaterial {},
            geometry: Cube::new(),
        };

        let cube_handle = render.add_mesh::<Cube, Instance, BasicMaterial>(cube_mesh);

        // let house_meshes = load_mesh("jet.glb").unwrap();

        // for mesh in house_meshes {
        //     let house_mesh_handle = render.add_mesh::<BasicGeometry, Instance, BasicMaterial>(mesh);
        //     render.add_render_object(BasicRenderObject {
        //         pipeline_handle,
        //         mesh_handle: house_mesh_handle,
        //         transform: Translation3::new(0.0, -1.0, -2.0).to_homogeneous(),
        //         texture_handle: stone_handle,
        //     });
        // }

        // how do we go from texture_handle -> atlas_coords?
        // problem: we call pack every time an image is added
        // the atlas_coords instance data might become outdated
        // atlas_coords needs to be re-evaluated every time
        //
        // rework:
        // create a render object trait, requiring a function that returns instance data
        //
        //

        // render.add_instance(
        //     cube_handle,
        //     Instance {
        //         transform: Translation3::new(0.0, 0.0, 0.0).to_homogeneous(),
        //         atlas_coords: Vector4::new(0, 0, 0, 0),
        //     },
        // );

        // render.add_instance(
        //     cube_handle,
        //     Instance {
        //         transform: Translation3::new(1.0, 0.0, 0.0).to_homogeneous(),
        //         atlas_coords: Vector4::new(0, 0, 0, 0),
        //     },
        // );

        Self {
            render,
            camera,
            rot_y: 0.0,
            rot_x: 0.0,
            camera_distance: 5.0,
            defaults_bind,
            pipeline_handle,
            cube_handle,
            cobble_handle,
            stone_handle,
            atlas_handle,
        }
    }

    fn draw(&mut self) {
        self.render.add_render_object(BasicRenderObject {
            pipeline_handle: self.pipeline_handle,
            mesh_handle: self.cube_handle,
            transform: Translation3::new(0.0, 0.0, 0.0).to_homogeneous(),
            texture_handle: self.cobble_handle,
            atlas_handle: self.atlas_handle,
        });

        self.render.add_render_object(BasicRenderObject {
            pipeline_handle: self.pipeline_handle,
            mesh_handle: self.cube_handle,
            transform: Translation3::new(1.0, 0.0, 0.0).to_homogeneous(),
            texture_handle: self.stone_handle,
            atlas_handle: self.atlas_handle,
        });

        self.render.draw();
    }

    fn input(&mut self, input: gggg::input::InputEvent) {
        println!("{:?}", input);
        match input {
            gggg::input::InputEvent::KeyboardInput { key, pressed } => {
                if key == "q" {
                    self.zoom_camera((0.0, 10.0));
                } else if key == "e" {
                    self.zoom_camera((0.0, -10.0));
                } else if key == "t" && pressed {
                    let texture = Texture::from_path("red.png");
                    let _ = self.render.add_texture(texture, self.atlas_handle);
                }
            }
            gggg::input::InputEvent::MouseInput(input) => match input {
                // gggg::input::MouseInputEvent::MouseMovement { delta } => self.move_camera(delta),
                gggg::input::MouseInputEvent::MouseScroll { delta } => self.move_camera(delta),
                _ => {}
            },
            _ => {}
        }
    }

    fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.render.resize(new_size);

        self.camera = Camera::new(
            point![1.0, 0.0, 5.0],
            point![0.0, 0.0, 0.0],
            ProjectionType::Perspective {
                aspect: new_size.width as f32 / new_size.height as f32,
                fovy: 70.0,
                near: 0.1,
                far: 100.0,
            },
        );

        self.move_camera((0.0, 0.0));

        let camera_data = CameraUniform {
            view_proj: self.camera.view_projection().into(),
            position: self.camera.eye.into(),
            padding: 0,
        };

        self.render
            .write_buffer(camera_data.as_bytes(), self.defaults_bind, 0);
    }
}

fn main() {
    make_window()
        .with_window_size((700, 700))
        .with_title("hello")
        .run(App::init);
}
