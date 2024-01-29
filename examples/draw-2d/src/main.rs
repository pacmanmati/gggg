use std::{f32::consts::TAU, rc::Rc};

use gggg::{
    bind::BindHandle,
    camera::{Camera, ProjectionType},
    material::BasicMaterial,
    pipeline::PipelineHandle,
    plain::Plain,
    render::{Mesh, MeshHandle, PhysicalSize, Render, Window},
    shapes::{quad_geometry, shape_pipeline, ShapeGeometry, ShapeInstance},
    text::{
        font_bitmap_manager::FontBitmapManager,
        pipeline::{
            quad_geometry as text_quad_geometry, text_pipeline, TextGeometry, TextInstance,
        },
        text_builder::TextBuilder,
    },
    window::{make_window, AppLoop},
};
use nalgebra::{point, Rotation3, Scale3, Translation3, Vector3};

struct App<'a> {
    render: Render<'a>,
    shape_pipeline_handle: PipelineHandle,
    mesh_handle: MeshHandle,
    camera: Camera,
    defaults_bind: BindHandle,
    text_pipeline_handle: PipelineHandle,
    text_bind: BindHandle,
    text_mesh_handle: MeshHandle,
    roboto_manager: Rc<FontBitmapManager>,
    rotation: f32,
    r: f32,
    g: f32,
    b: f32,
}

impl<'a> AppLoop for App<'a> {
    type App = App<'a>;

    fn init(window: &Window, gggg: &gggg::window::App) -> Self::App {
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
                left: 0.0,
                right: 100.0,
                top: 100.0,
                bottom: 0.0,
                near: -200.0,
                far: 200.0,
            },
        );

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

        let font_atlas_handle =
            render.register_atlas(text_bind, 1, gggg::texture::TextureFormat::R8Unorm);
        let roboto_manager = Rc::new(
            FontBitmapManager::new(&mut render, "Roboto.ttf", 4096.0 / 4.0, font_atlas_handle)
                .unwrap(),
        );

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
            rotation: 0.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }

    fn draw(&mut self, gggg: &gggg::window::App) {
        // self.rotation += TAU / 100.0;
        // self.r = (self.r + 0.001) % 1.0;
        // self.g = (self.g + 0.002) % 1.0;
        // self.b = (self.b + 0.003) % 1.0;

        TextBuilder::new(
            "hello world",
            [self.r, self.g, self.b, 1.0],
            Translation3::new(0.0, 50.0, 0.0).to_homogeneous()
                // * Translation3::new(12.0, 1.0, 0.0).to_homogeneous()
                * Rotation3::from_axis_angle(&Vector3::z_axis(), self.rotation).to_homogeneous()
                // * Translation3::new(-12.0, -1.0, 0.0).to_homogeneous()
                * Scale3::new(20.0, 20.0, 1.0).to_homogeneous(),
            self.roboto_manager.clone(),
            self.text_pipeline_handle,
            self.text_mesh_handle,
            1.0,
        )
        .build(&mut self.render)
        .unwrap()
        .into_iter()
        .for_each(|obj| self.render.add_render_object(obj));

        self.render.draw();
    }

    fn resized(&mut self, new_size: PhysicalSize<u32>) {
        self.render.resize(new_size);

        self.camera = Camera::new(
            point![0.0, 0.0, 100.0],
            point![0.0, 0.0, 0.0],
            ProjectionType::Orthographic {
                left: 0.0,
                right: 100.0,
                top: 100.0,
                bottom: 0.0,
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
