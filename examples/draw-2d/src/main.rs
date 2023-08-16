use std::f32::consts::FRAC_PI_4;

use gggg::{
    bind::BindHandle,
    camera::{Camera, ProjectionType},
    material::BasicMaterial,
    pipeline::PipelineHandle,
    plain::Plain,
    render::{Mesh, MeshHandle, Render},
    shapes::{quad_geometry, shape_pipeline, ShapeGeometry, ShapeInstance, ShapeRenderObject},
    window::{make_window, AppLoop},
};
use nalgebra::{point, Rotation3, Scale3, Translation3, Vector3};

struct App {
    render: Render,
    shape_pipeline_handle: PipelineHandle,
    mesh_handle: MeshHandle,
    camera: Camera,
    defaults_bind: BindHandle,
}

impl AppLoop for App {
    type App = App;

    fn init(window: &winit::window::Window) -> Self::App {
        let mut render = Render::new(window).unwrap();
        let (shape_pipeline, defaults_bind) = shape_pipeline(&mut render);
        let shape_pipeline_handle = render.add_pipeline(shape_pipeline);

        let mesh_handle = render.add_mesh::<ShapeGeometry, ShapeInstance, BasicMaterial>(Mesh {
            material: BasicMaterial {},
            geometry: quad_geometry(),
        });

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

        let camera_data = camera.uniform();
        let camera_bytes = camera_data.as_bytes();

        render.write_buffer(camera_bytes, defaults_bind, 0);

        App {
            render,
            shape_pipeline_handle,
            mesh_handle,
            defaults_bind,
            camera,
        }
    }

    fn draw(&mut self) {
        self.render.add_render_object(ShapeRenderObject {
            transform: Translation3::new(10.0, 0.0, 0.0).to_homogeneous()
                * Rotation3::new(Vector3::z() * FRAC_PI_4).to_homogeneous()
                * Scale3::new(10.0, 10.0, 1.0).to_homogeneous(),

            albedo: [0.0, 1.0, 0.0, 1.0],
            pipeline_handle: self.shape_pipeline_handle,
            mesh_handle: self.mesh_handle,
        });
        self.render.add_render_object(ShapeRenderObject {
            transform: Translation3::new(5.0, 0.0, 0.0).to_homogeneous()
                * Rotation3::new(Vector3::z() * FRAC_PI_4).to_homogeneous()
                * Scale3::new(5.0, 5.0, 1.0).to_homogeneous(),

            albedo: [0.0, 0.0, 1.0, 1.0],
            pipeline_handle: self.shape_pipeline_handle,
            mesh_handle: self.mesh_handle,
        });
        self.render.draw();
    }

    fn resized(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.render.resize(new_size);

        self.camera = Camera::new(
            point![0.0, 0.0, 100.0],
            point![0.0, 0.0, 0.0],
            ProjectionType::Orthographic {
                left: -20.0,
                right: 20.0,
                top: -20.0,
                bottom: 20.0,
                near: -200.0,
                far: 200.0,
            },
        );

        let camera_data = self.camera.uniform();

        self.render
            .write_buffer(camera_data.as_bytes(), self.defaults_bind, 0);
    }
}

fn main() {
    make_window()
        .with_window_size((700, 700))
        .with_title("draw_2d")
        .run(App::init);
}
