use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
};

use anyhow::{anyhow, Result};
use fontdue::{Font, FontSettings, Metrics};

use crate::{
    render::{AtlasHandle, Render, TextureHandle},
    texture::Texture,
};

// one of these needs to exist for each font used, at each px, each font weight, etc.
// that's fine - this is just a lightweight mapping of glyph -> texturehandle
// the bigger concern is keeping so many textures loaded on the renderer
// but if that becomes an issue, we can solve it
#[derive(Clone, Debug)]
pub struct FontBitmapManager {
    map: HashMap<char, (TextureHandle, Metrics)>,
    pub atlas_handle: AtlasHandle,
}

impl FontBitmapManager {
    pub fn new(
        render: &mut Render,
        font_path: &str,
        px: f32,
        atlas_handle: AtlasHandle,
    ) -> Result<Self> {
        let file = File::open(font_path)?;
        let mut reader = BufReader::new(file);
        let mut buf = Vec::new();
        let _ = reader.read_to_end(&mut buf)?;
        let font = Font::from_bytes(buf, FontSettings::default()).map_err(|err| anyhow!(err))?;

        let map = font
            .chars()
            .iter()
            .map(|(c, _)| {
                let (metrics, mut bitmap) = font.rasterize(*c, px);
                // bitmap = bitmap.iter().fold(vec![], |mut acc, val| {
                //     // acc.extend_from_slice(&[v, v, v, 255]);
                //     acc.extend_from_slice(&[255, 255, 255, *val]);
                //     acc
                // });

                // assert!(bitmap.len() == metrics.width * metrics.height * 4);
                let texture = Texture {
                    data: bitmap,
                    width: metrics.width as u32,
                    height: metrics.height as u32,
                    format: crate::texture::TextureFormat::R8Unorm,
                };
                let texture_handle = render.add_texture(texture, atlas_handle)?;
                anyhow::Ok((*c, (texture_handle, metrics)))
            })
            .collect::<Result<HashMap<char, (TextureHandle, Metrics)>>>()?;

        Ok(Self { map, atlas_handle })
    }

    pub fn get_metric(&self, character: char) -> Result<Metrics> {
        self.map
            .get(&character)
            .ok_or(anyhow!(
                "Couldn't find metric for character '{}'",
                character
            ))
            .copied()
            .map(|inner| inner.1)
    }

    pub fn get_texture(&self, character: char) -> Result<TextureHandle> {
        self.map
            .get(&character)
            .ok_or(anyhow!(
                "Couldn't find texture for character '{}'",
                character
            ))
            .copied()
            .map(|inner| inner.0)
    }
}
