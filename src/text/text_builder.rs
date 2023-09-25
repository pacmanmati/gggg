use std::rc::Rc;

use anyhow::Result;
use nalgebra::{Matrix4, Scale3, Translation3};

use crate::{
    pipeline::PipelineHandle,
    render::{MeshHandle, Render},
};

use super::{font_bitmap_manager::FontBitmapManager, pipeline::TextRenderObject};

pub struct TextBuilder {
    text: String,
    font_manager: Rc<FontBitmapManager>,
    albedo: [f32; 4],
    transform: Matrix4<f32>,
    pipeline_handle: PipelineHandle,
    mesh_handle: MeshHandle,
    scale: f32,
}

impl TextBuilder {
    pub fn new(
        text: &str,
        albedo: [f32; 4],
        transform: Matrix4<f32>,
        font_manager: Rc<FontBitmapManager>,
        pipeline_handle: PipelineHandle,
        mesh_handle: MeshHandle,
        scale: f32,
    ) -> Self {
        Self {
            text: text.to_string(),
            font_manager,
            albedo,
            transform,
            pipeline_handle,
            mesh_handle,
            scale,
        }
    }

    pub fn build(&self, render: &mut Render) -> Result<Vec<TextRenderObject>> {
        let mut render_objs = Vec::new();
        let mut x = 0.0;
        let mut y = 0.0;

        let scale = self.scale / self.font_manager.px;

        for character in self.text.chars() {
            let metrics = self.font_manager.get_metric(character)?;

            let xpos = x + metrics.xmin as f32 * scale;
            let ypos = y + metrics.ymin as f32 * scale;
            let w = metrics.width as f32 * scale;
            let h = metrics.height as f32 * scale;

            let transform = self.transform
                * Translation3::new(xpos, ypos, 0.0).to_homogeneous()
                * Scale3::new(w, h, 1.0).to_homogeneous();

            let render_obj = TextRenderObject {
                transform: transform,
                albedo: self.albedo,
                pipeline_handle: self.pipeline_handle,
                mesh_handle: self.mesh_handle,
                character,
                manager: self.font_manager.clone(),
            };

            render_objs.push(render_obj);

            x += metrics.advance_width * scale;
        }
        Ok(render_objs)
    }
}
