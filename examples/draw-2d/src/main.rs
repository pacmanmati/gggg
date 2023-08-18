use gggg::{
    bind::BindHandle,
    camera::{Camera, ProjectionType},
    material::BasicMaterial,
    pipeline::PipelineHandle,
    plain::Plain,
    render::{Mesh, MeshHandle, Render},
    shapes::{quad_geometry, shape_pipeline, ShapeGeometry, ShapeInstance},
    text::{
        font_bitmap_manager::FontBitmapManager,
        pipeline::{
            quad_geometry as text_quad_geometry, text_pipeline, TextGeometry, TextInstance,
            TextRenderObject,
        },
    },
    window::{make_window, AppLoop},
};
use nalgebra::{point, Scale3, Translation3};

struct App {
    render: Render,
    shape_pipeline_handle: PipelineHandle,
    mesh_handle: MeshHandle,
    camera: Camera,
    defaults_bind: BindHandle,
    text_pipeline_handle: PipelineHandle,
    text_bind: BindHandle,
    text_mesh_handle: MeshHandle,
    roboto_manager: FontBitmapManager,
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
            point![0.0, 0.0, 100.0],
            point![0.0, 0.0, 0.0],
            ProjectionType::Orthographic {
                left: -20.0,
                right: 20.0,
                top: 20.0,
                bottom: -20.0,
                near: -200.0,
                far: 200.0,
            },
        );
        //  Camera::new(
        //     point![1.0, 0.0, 5.0],
        //     point![0.0, 0.0, 0.0],
        //     ProjectionType::Perspective {
        //         aspect: window.inner_size().width as f32 / window.inner_size().height as f32,
        //         fovy: 70.0,
        //         near: 0.1,
        //         far: 100.0,
        //     },
        // );

        let camera_data = camera.uniform();
        let camera_bytes = camera_data.as_bytes();

        // text
        let (text_pipeline, text_bind) = text_pipeline(&mut render);
        let text_pipeline_handle = render.add_pipeline(text_pipeline);
        let text_mesh_handle = render.add_mesh::<TextGeometry, TextInstance, BasicMaterial>(Mesh {
            material: BasicMaterial {},
            geometry: text_quad_geometry(),
        });

        render.write_buffer(camera_bytes, defaults_bind, 0);
        render.write_buffer(camera_bytes, text_bind, 0);

        let font_atlas_handle = render.register_atlas(text_bind, 1);
        let roboto_manager =
            FontBitmapManager::new(&mut render, "Roboto.ttf", 250.0, font_atlas_handle).unwrap();

        App {
            render,
            shape_pipeline_handle,
            mesh_handle,
            defaults_bind,
            camera,
            text_pipeline_handle,
            text_bind,
            text_mesh_handle,
            roboto_manager,
        }
    }

    fn draw(&mut self) {
        // self.render.add_render_object(ShapeRenderObject {
        //     transform: Translation3::new(10.0, 0.0, 0.0).to_homogeneous()
        //         * Rotation3::new(Vector3::z() * FRAC_PI_4).to_homogeneous()
        //         * Scale3::new(10.0, 10.0, 1.0).to_homogeneous(),

        //     albedo: [0.0, 1.0, 0.0, 1.0],
        //     pipeline_handle: self.shape_pipeline_handle,
        //     mesh_handle: self.mesh_handle,
        // });
        // self.render.add_render_object(ShapeRenderObject {
        //     transform: Translation3::new(5.0, 0.0, 0.0).to_homogeneous()
        //         * Rotation3::new(Vector3::z() * FRAC_PI_4).to_homogeneous()
        //         * Scale3::new(5.0, 5.0, 1.0).to_homogeneous(),
        //     albedo: [0.0, 0.0, 1.0, 1.0],
        //     pipeline_handle: self.shape_pipeline_handle,
        //     mesh_handle: self.mesh_handle,
        // });

        let metric = self.roboto_manager.get_metric('a').unwrap();
        let glyph_aspect_ratio = metric.width as f32 / metric.height as f32;
        // println!("{}", glyph_aspect_ratio);

        self.render.add_render_object(TextRenderObject {
            transform: Translation3::new(0.0, 0.0, 0.0).to_homogeneous()
                * Scale3::new(10.0 * glyph_aspect_ratio, 10.0 / glyph_aspect_ratio, 1.0)
                    .to_homogeneous(),
            albedo: [1.0, 0.0, 0.0, 1.0],
            pipeline_handle: self.text_pipeline_handle,
            mesh_handle: self.text_mesh_handle,
            character: 'a',
            manager: self.roboto_manager.clone(),
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
                top: 20.0,
                bottom: -20.0,
                near: -200.0,
                far: 200.0,
            },
        );

        let camera_data = self.camera.uniform();

        self.render
            .write_buffer(camera_data.as_bytes(), self.defaults_bind, 0);
        self.render
            .write_buffer(camera_data.as_bytes(), self.text_bind, 0);
    }
}

fn main() {
    make_window()
        .with_window_size((700, 700))
        .with_title("draw_2d")
        .run(App::init);
}
