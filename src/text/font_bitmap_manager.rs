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
            // [
            //     'a',
            //     'b',
            //     'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            //     'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
            //     'V',
            //     'W', 'X', 'Y', 'Z',
            // ]
            .iter()
            .map(|(c, _)| {
                // .map(|c| {
                let (metrics, mut bitmap) = font.rasterize(*c, px);
                // for now, we wastefully convert bitmap from an r8unorm (grayscale) format to an rgba8unorm format (e.g. insert 3 copies of each value)
                // let random = rand::random::<u8>();
                // let v = (*c as u32).try_into().unwrap_or_else(|_| random);
                bitmap = bitmap.iter().fold(vec![], |mut acc, val| {
                    // acc.extend_from_slice(&[v, v, v, 255]);
                    acc.extend_from_slice(&[255, 255, 255, *val]);
                    acc
                });

                assert!(bitmap.len() == metrics.width * metrics.height * 4);
                let texture = Texture {
                    data: bitmap,
                    // data: std::iter::repeat(255)
                    //     .take(bitmap.len().try_into().unwrap())
                    //     .collect(),
                    width: metrics.width as u32,
                    height: metrics.height as u32,
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
