use std::{collections::HashMap, iter::repeat, path::Path};

use generational_arena::Arena;

use crate::{
    atlas::{Atlas, RectHandle},
    render::TextureHandle,
};

#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    Rgba8Unorm,
    R8Unorm,
    Rgba16,
}

impl TextureFormat {
    pub fn pixel_size(&self) -> u32 {
        match self {
            TextureFormat::Rgba8Unorm => 4,
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rgba16 => 8,
        }
    }
}

pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

impl Texture {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let img = image::open(path).unwrap();
        let buff = img.to_rgba8();
        Self {
            width: buff.width(),
            height: buff.height(),
            data: buff.to_vec(),
            format: TextureFormat::Rgba8Unorm,
        }
    }

    pub fn from_atlas(
        atlas: &Atlas,
        rect_to_tex: &HashMap<RectHandle, TextureHandle>,
        textures: &Arena<Texture>,
    ) -> Self {
        let pixel_size = atlas.format.pixel_size();
        let mut data: Vec<u8> = repeat(0)
            .take(
                (atlas.width * atlas.height * pixel_size)
                    .try_into()
                    .unwrap(),
            )
            .collect();
        for (rect_handle, texture_handle) in rect_to_tex {
            let rect = atlas.get_rect(*rect_handle).unwrap();
            let offset: usize = (pixel_size * (rect.x + rect.y * atlas.width))
                .try_into()
                .unwrap();
            let texture = textures.get(texture_handle.0).unwrap();
            let tex_width = texture.width as usize;

            // we cannot naively paste the texture data into a 1d array
            // we need to do it row by row
            // we'll always be at the same x offset but our y offset can change
            // and we need to iterate over the texture's rows
            for row in 0..texture.height as usize {
                let row_data = texture
                    .data
                    .get(
                        (row * tex_width * pixel_size as usize)
                            ..((row + 1) * tex_width * pixel_size as usize),
                    )
                    .unwrap()
                    .to_vec();
                let row_offset = row * atlas.width as usize * pixel_size as usize + offset;
                // println!("{:?}", row_offset..row_offset + row_data.len());
                data.splice(row_offset..row_offset + row_data.len(), row_data);
            }
        }

        Self {
            data,
            width: atlas.width,
            height: atlas.height,
            format: atlas.format,
        }
    }
}
