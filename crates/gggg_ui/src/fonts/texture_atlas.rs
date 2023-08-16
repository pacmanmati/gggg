use std::{collections::HashMap, hash::Hash, iter::repeat, rc::Rc};

pub type TextureHandle = u32;

pub struct Image {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

pub struct TextureAtlas<T> {
    counter: u32,
    key_to_rect: Option<HashMap<T, Rect>>,

    rects: Vec<(Rect, TextureHandle)>,
    pub width: i32,
    pub height: i32,
}

impl<T: Hash + Eq> TextureAtlas<T> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            key_to_rect: None,
            rects: vec![],
            width: 0,
            height: 0,
        }
    }

    pub fn add(&mut self, w: i32, h: i32) -> TextureHandle {
        let handle = self.counter;
        self.counter += 1;
        let rect = Rect { x: 0, y: 0, w, h };
        self.rects.push((rect, handle));
        handle
    }

    pub fn pack(&mut self) {
        // let's go for a fixed width to break on
        let mut x = 0;
        let mut y = 0;
        self.width = 512;
        // sort s.t. the tallest rect is first
        // decreasing rect height means we can place anything
        self.rects.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        // self.rects.reverse();
        let mut max_h = self.rects.first().unwrap().0.h;
        for (rect, _) in self.rects.iter_mut() {
            // bounds check
            if x + rect.x + rect.w >= self.width {
                y += max_h;
                x = 0;
                max_h = rect.h;
            }
            // place rect
            rect.x = x;
            rect.y = y;
            // move along
            x += rect.w;
        }
        self.height = y + max_h;
        // println!("{}, {:?}", self.height, self.rects);
    }

    pub fn get_rect_from_handle(&self, handle: &TextureHandle) -> Option<(Rect, TextureHandle)> {
        self.rects.iter().find(|(_, x)| x == handle).copied()
    }

    pub fn get_rect(&self, t: T) -> Option<Rect> {
        self.key_to_rect.as_ref().unwrap().get(&t).copied()
    }

    pub fn merge_bitmaps(
        &mut self,
        bitmaps: HashMap<T, Image>,
        keys: HashMap<T, TextureHandle>,
    ) -> Image {
        let mut data: Vec<u8> = repeat(0)
            .take((self.width * self.height).try_into().unwrap())
            .collect();
        let handle_to_rect: HashMap<&TextureHandle, &Rect> =
            self.rects.iter().map(|(k, v)| (v, k)).collect();
        self.key_to_rect = Some(
            keys.into_iter()
                .map(|(k, v)| (k, *handle_to_rect.get(&v).copied().unwrap()))
                .collect(),
        );

        for (t, image) in bitmaps.into_iter() {
            let rect = self.key_to_rect.as_ref().unwrap().get(&t).unwrap();
            let offset: usize = (rect.x + rect.w * self.width).try_into().unwrap(); // rect.y instead of rect.w?
            data.splice(offset..offset + image.data.len(), image.data);
        }

        Image {
            data,
            width: self.width as usize,
            height: self.height as usize,
        }
    }
}

impl<T: Hash + Eq> Default for TextureAtlas<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Ord for Rect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.h).cmp(&other.h)
    }
}

impl PartialOrd for Rect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.h.partial_cmp(&other.h)
    }
}
