use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::{anyhow, Result};
use cosmic_text::{CacheKey, FontSystem, SwashCache};

use crate::{
    bind::BindHandle,
    render::{AtlasHandle, Render, TextureHandle},
    texture::{Texture, TextureFormat},
};

// one of these needs to exist for each font used, at each px, each font weight, etc.
// that's fine - this is just a lightweight mapping of glyph -> texturehandle
// the bigger concern is keeping so many textures loaded on the renderer
// but if that becomes an issue, we can solve it
#[derive(Debug)]
pub struct FontBitmapManager {
    pub font_system: Rc<RefCell<FontSystem>>,
    pub swash_cache: Rc<RefCell<SwashCache>>,
    pub font_atlas_handle: AtlasHandle,
    map: Rc<RefCell<HashMap<CacheKey, TextureHandle>>>,
}

impl FontBitmapManager {
    pub fn new(render: &mut Render, text_bind: BindHandle) -> Result<Self> {
        let font_system = Rc::new(RefCell::new(FontSystem::new()));
        let swash_cache = Rc::new(RefCell::new(SwashCache::new()));

        // register an atlas ourselves?
        let font_atlas_handle = render.register_atlas(text_bind, 1, TextureFormat::R8Unorm);

        Ok(Self {
            font_atlas_handle,
            font_system,
            swash_cache,
            map: Rc::new(RefCell::new(HashMap::new())),
        })
    }

    pub fn get_font_system(&self) -> Rc<RefCell<FontSystem>> {
        self.font_system.clone()
    }

    pub fn get_swash_cache(&self) -> Rc<RefCell<SwashCache>> {
        self.swash_cache.clone()
    }

    pub fn add_texture(
        &self,
        key: CacheKey,
        texture: Texture,
        render: &mut Render,
    ) -> Result<TextureHandle> {
        let texture_handle = render.add_texture(texture, self.font_atlas_handle)?;
        self.map.borrow_mut().insert(key, texture_handle);
        Ok(texture_handle)
    }

    pub fn get_texture(&self, key: CacheKey) -> Result<TextureHandle> {
        self.map
            .borrow()
            .get(&key)
            .ok_or(anyhow!("Couldn't find texture for cachekey '{:?}'", key))
            .copied()
    }
}
