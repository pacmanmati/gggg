use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use anyhow::Result;
use cosmic_text::{Attrs, Buffer, Color, Family, Metrics, Shaping};
use nalgebra::{Matrix4, Scale3, Translation3};

use crate::{
    pipeline::PipelineHandle,
    render::{MeshHandle, Render},
    texture::Texture,
    transform::Transform,
};

use super::{font_bitmap_manager::FontBitmapManager, pipeline::TextRenderObject};

pub struct TextBuilder {
    text: String,
    font_manager: Rc<RefCell<FontBitmapManager>>,
    albedo: [f32; 4],
    transform: Transform,
    pipeline_handle: PipelineHandle,
    mesh_handle: MeshHandle,
    scale: f32,
    family_name: String,
}

impl TextBuilder {
    pub fn new(
        text: &str,
        family_name: &str,
        albedo: [f32; 4],
        transform: Transform,
        font_manager: Rc<RefCell<FontBitmapManager>>,
        pipeline_handle: PipelineHandle,
        mesh_handle: MeshHandle,
        scale: f32,
    ) -> Self {
        Self {
            family_name: family_name.to_string(),
            text: text.to_string(),
            font_manager,
            albedo,
            transform,
            pipeline_handle,
            mesh_handle,
            scale,
        }
    }

    pub fn build(&mut self, render: &mut Render) -> Result<Vec<TextRenderObject>> {
        let attrs = Attrs::new().family(Family::Name(&self.family_name));

        let mut render_objs = Vec::new();
        // Text metrics indicate the font size and line height of a buffer
        let metrics = Metrics::new(14.0, 20.0);
        // let metrics = Metrics::new(6.0, 10.0);

        let font_system = self.font_manager.borrow().get_font_system();
        let swash_cache = self.font_manager.borrow().get_swash_cache();
        let mutref = &mut font_system.borrow_mut();
        // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
        let mut buffer = Buffer::new(mutref, metrics);

        // Borrow buffer together with the font system for more convenient method calls
        // let mut buffer = buffer.borrow_with(mutref);

        // Set a size for the text buffer, in pixels
        buffer.set_size(mutref, 80.0, 25.0);

        // Add some text!
        buffer.set_text(mutref, &self.text, attrs, Shaping::Advanced);

        // Perform shaping as desired
        buffer.shape_until_scroll(mutref, true);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical(
                    (self.transform.translation.x, self.transform.translation.y),
                    self.scale,
                );

                let texture_handle = if let Ok(handle) = self
                    .font_manager
                    .borrow()
                    .get_texture(physical_glyph.cache_key)
                {
                    handle
                } else {
                    let image = swash_cache
                        .borrow_mut()
                        .get_image_uncached(mutref, physical_glyph.cache_key)
                        .unwrap();
                    // create in font atlas?
                    let texture = Texture {
                        data: image.data,
                        width: image.placement.width,
                        height: image.placement.height,
                        format: crate::texture::TextureFormat::R8Unorm,
                    };
                    self.font_manager
                        .borrow()
                        .add_texture(physical_glyph.cache_key, texture, render)
                        .unwrap()
                };

                let transform = self.transform.matrix()
                    * Translation3::new(glyph.x, -glyph.y, 0.0).to_homogeneous()
                    * Scale3::new(glyph.w, metrics.line_height, 1.0).to_homogeneous();

                let render_obj = TextRenderObject {
                    texture_handle,
                    transform: transform,
                    albedo: self.albedo,
                    pipeline_handle: self.pipeline_handle,
                    mesh_handle: self.mesh_handle,
                    character: 'x',
                    manager: self.font_manager.clone(),
                };

                render_objs.push(render_obj);
            }
        }

        // buffer.draw(
        //     &mut swash_cache.borrow_mut(),
        //     Color::rgb(0xFF, 0xFF, 0xFF),
        //     |x, y, w, h, color| {

        //     },
        // );

        Ok(render_objs)
    }
}
