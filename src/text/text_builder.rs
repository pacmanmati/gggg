use std::{borrow::BorrowMut, rc::Rc};

use anyhow::Result;
use nalgebra::{Matrix4, Scale3, Translation3};

use crate::{pipeline::PipelineHandle, render::MeshHandle};

use super::{font_bitmap_manager::FontBitmapManager, pipeline::TextRenderObject};

pub struct TextBuilder {
    text: String,
    font_manager: Rc<FontBitmapManager>,
    albedo: [f32; 4],
    transform: Matrix4<f32>,
    pipeline_handle: PipelineHandle,
    mesh_handle: MeshHandle,
}

impl TextBuilder {
    pub fn new(
        text: &str,
        albedo: [f32; 4],
        transform: Matrix4<f32>,
        font_manager: Rc<FontBitmapManager>,
        pipeline_handle: PipelineHandle,
        mesh_handle: MeshHandle,
    ) -> Self {
        Self {
            text: text.to_string(),
            font_manager,
            albedo,
            transform,
            pipeline_handle,
            mesh_handle,
        }
    }

    pub fn build(&self) -> Result<Vec<TextRenderObject>> {
        let mut render_objs = Vec::new();
        let mut current_transform = Matrix4::identity();

        for character in self.text.chars() {
            let metrics = self.font_manager.get_metric(character)?;
            let glyph_aspect_ratio = metrics.width as f32 / metrics.height as f32;

            let transform = self.transform * current_transform;
            // * Scale3::new(1.0 * glyph_aspect_ratio, 1.0 / glyph_aspect_ratio, 1.0)
            //     .to_homogeneous();
            let render_obj = TextRenderObject {
                transform,
                albedo: self.albedo,
                pipeline_handle: self.pipeline_handle,
                mesh_handle: self.mesh_handle,
                character,
                manager: self.font_manager.clone(),
            };

            render_objs.push(render_obj);
            println!("{}", metrics.advance_width);

            current_transform = current_transform
                * Translation3::new(metrics.advance_width, 0.0, 0.0).to_homogeneous();
        }
        Ok(render_objs)
    }
}
