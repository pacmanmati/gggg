use gggg::{
    bind::BindHandle,
    camera::{Camera, ProjectionType},
    material::BasicMaterial,
    pipeline::PipelineHandle,
    plain::Plain,
    render::{Mesh, MeshHandle, PhysicalSize, Render},
    shapes::{quad_shape_offset, shape_pipeline, ShapeGeometry, ShapeInstance, ShapeRenderObject},
    window::{make_window, AppLoop},
};
use nalgebra::{point, Translation3};

struct Pixel {
    position: [u32; 2],
    color: [f32; 4],
}

struct App<'a> {
    render: Render<'a>,
    camera: Camera,
    pipeline: PipelineHandle,
    bind: BindHandle,
    mesh_handle: MeshHandle,
    pixels: Vec<Pixel>,
}

pub fn quad_geometry() -> ShapeGeometry {
    ShapeGeometry {
        vertices: quad_shape_offset(0.5, 0.5),
        indices: Some([0, 2, 1, 1, 2, 3]),
    }
}

impl<'a> AppLoop for App<'a> {
    type App = App<'a>;

    fn init(window: &gggg::render::Window, gggg: &gggg::window::App) -> Self::App {
        let mut render = Render::new(window).unwrap();
        let (pixel_pipeline, pixel_bind) = shape_pipeline(&mut render);

        let pipeline = render.add_pipeline(pixel_pipeline);

        let mesh_handle = render.add_mesh::<ShapeGeometry, ShapeInstance, BasicMaterial>(Mesh {
            material: BasicMaterial {},
            geometry: quad_geometry(),
        });

        let camera = Camera::new(
            point![0.0, 0.0, -100.0],
            point![0.0, 0.0, 0.0],
            ProjectionType::Orthographic {
                left: 0.0,
                right: 5.0,
                top: 5.0,
                bottom: 0.0,
                near: -200.0,
                far: 200.0,
            },
        );

        let camera_data = camera.uniform();
        let camera_bytes = camera_data.as_bytes();

        render.write_buffer(camera_bytes, pixel_bind, 0);

        App {
            render,
            bind: pixel_bind,
            camera,
            pipeline,
            mesh_handle,
            pixels: vec![
                Pixel {
                    position: [0, 0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                Pixel {
                    position: [1, 1],
                    color: [1.0, 1.0, 0.0, 1.0],
                },
                Pixel {
                    position: [2, 2],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
            ],
        }
    }

    fn input(&mut self, _input: gggg::input::InputEvent, gggg: &gggg::window::App) {
        gggg.get_mouse_position();
        println!("{:?}", _input);
    }

    fn draw(&mut self, gggg: &gggg::window::App) {
        for pixel in &self.pixels {
            self.render.add_render_object(ShapeRenderObject {
                transform: Translation3::new(
                    pixel.position[0] as f32,
                    pixel.position[1] as f32,
                    0.0,
                )
                .to_homogeneous(),
                albedo: pixel.color,
                pipeline_handle: self.pipeline,
                mesh_handle: self.mesh_handle,
            });
        }
        self.render.draw();
    }

    fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.render.resize(new_size);

        self.camera = Camera::new(
            point![0.0, 0.0, 100.0],
            point![0.0, 0.0, 0.0],
            ProjectionType::Orthographic {
                left: 0.0,
                right: 50.0,
                top: 50.0,
                bottom: 0.0,
                near: -200.0,
                far: 200.0,
            },
        );

        let camera_data = self.camera.uniform();

        self.render
            .write_buffer(camera_data.as_bytes(), self.bind, 0);
    }
}

fn main() {
    make_window()
        .with_window_size((700, 700))
        .with_title("falling sand")
        .run(App::init);
}
