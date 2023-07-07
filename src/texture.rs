use std::{collections::HashMap, iter::repeat, path::Path};

use generational_arena::Arena;

use crate::{
    atlas::{Atlas, RectHandle},
    render::TextureHandle,
};

pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
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
        Self {
            width: img.width(),
            height: img.height(),
            data: img.into_bytes(),
        }
    }

    pub fn from_atlas(
        atlas: &Atlas,
        rect_to_tex: &HashMap<RectHandle, TextureHandle>,
        textures: &Arena<Texture>,
    ) -> Self {
        let mut data: Vec<u8> = repeat(0)
            .take((atlas.width * atlas.height).try_into().unwrap())
            .collect();
        for (rect_handle, texture_handle) in rect_to_tex {
            let rect = atlas.get_rect(*rect_handle).unwrap();
            let offset: usize = (rect.x + rect.w * atlas.width).try_into().unwrap();
            let texture = textures.get(texture_handle.0).unwrap();
            data.splice(offset..offset + texture.data.len(), texture.data.clone());
        }

        Self {
            data,
            width: atlas.width,
            height: atlas.height,
        }
    }
}
