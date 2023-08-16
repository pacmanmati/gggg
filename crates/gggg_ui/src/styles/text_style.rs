use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    rc::Rc,
};

use fontdue::{Font, FontSettings, Metrics};

use crate::{
    context::Context,
    fonts::texture_atlas::{Image, Rect, TextureAtlas},
    widgets::Color,
};

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct TextStyleHandle(u32);

#[derive(Clone)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub color: Color,
}

impl TextStyle {
    pub fn new(color: Color, font_family: String, font_size: f32) -> Self {
        Self {
            color,
            font_family,
            font_size,
        }
    }

    pub(crate) fn handle(&self) -> TextStyleHandle {
        let mut hasher = DefaultHasher::default();
        hasher.write(self.font_family.as_bytes());
        hasher.write(&self.font_size.to_ne_bytes());
        let value = u32::try_from(hasher.finish()).unwrap();
        TextStyleHandle(value)
    }

    pub(crate) fn compute(&mut self, context: &RefCell<Context>) -> Rc<TextStyleComputed> {
        let handle = self.handle();

        // check if it already exists
        if let Some(computed) = context.borrow().text_styles.get(&handle) {
            return computed.clone();
        }

        // resolve font family into filepath
        // let asset = Loader::load(&mut self, path).unwrap();
        let asset = context.borrow().loader.get_font_by_family().unwrap();
        let computed = TextStyleComputed::new(asset.bytes, self.font_size);

        context
            .borrow_mut()
            .text_styles
            .insert(handle, Rc::new(computed));

        context.borrow().text_styles.get(&handle).unwrap().clone()
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new(
            Color::RGBA(0.0, 0.0, 0.0, 1.0), // black
            "Roboto".into(),
            20.0,
        )
    }
}

/// Contains the atlas and metrics of a text style's font, in addition to any other contents worth 'caching'.
pub struct TextStyleComputed {
    font_image: Image,
    atlas: TextureAtlas<char>,
    char_metrics: HashMap<char, Metrics>,
}

impl TextStyleComputed {
    pub fn new(font_bytes: Vec<u8>, px: f32) -> Self {
        let font = Font::from_bytes(font_bytes, FontSettings::default()).unwrap();
        let mut atlas = TextureAtlas::<char>::new();

        let mut char_to_handle = HashMap::new();
        let mut images = HashMap::new();
        let mut char_metrics = HashMap::new();

        for (char, _) in font.chars() {
            let (metrics, bitmap) = font.rasterize(*char, px);
            images.insert(
                *char,
                Image {
                    data: bitmap,
                    width: metrics.width,
                    height: metrics.height,
                },
            );
            let handle = atlas.add(metrics.width as i32, metrics.height as i32);
            char_to_handle.insert(*char, handle);
            char_metrics.insert(*char, metrics);
        }
        atlas.pack();

        let font_image = atlas.merge_bitmaps(images, char_to_handle);

        Self {
            font_image,
            atlas,
            char_metrics,
        }
    }

    pub fn image(&self) -> &Image {
        &self.font_image
    }

    pub fn get_char_rect(&self, c: char) -> Rect {
        self.atlas.get_rect(c).unwrap()
    }

    pub fn get_char_metrics(&self, c: &char) -> &Metrics {
        self.char_metrics.get(c).unwrap()
    }
}
