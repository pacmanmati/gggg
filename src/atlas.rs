use std::hash::Hash;

use generational_arena::{Arena, Index};
use itertools::Itertools;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct RectHandle(pub Index);

pub struct Image {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

pub struct Atlas {
    rects: Arena<Rect>,
    pub width: u32,
    pub height: u32,
    pub changed: bool,
}

impl Atlas {
    pub fn new() -> Self {
        Self {
            rects: Arena::new(),
            width: 0,
            height: 0,
            changed: false,
        }
    }

    pub fn add(&mut self, w: u32, h: u32) -> RectHandle {
        let rect = Rect { x: 0, y: 0, w, h };
        let index = self.rects.insert(rect);

        RectHandle(index)
    }

    pub fn pack(&mut self) {
        self.changed = true;
        // let's go for a fixed width to break on
        let mut x = 0;
        let mut y = 0;
        self.width = 512;
        // sort s.t. the tallest rect is first
        // decreasing rect height means we can pack everything in a row
        let sorted_rects = self
            .rects
            .iter()
            .sorted_by(|a, b| b.1.partial_cmp(a.1).unwrap())
            // .rev()
            .collect::<Vec<_>>();

        let mut max_h = sorted_rects.first().unwrap().1.h;
        for (_, rect) in self.rects.iter_mut() {
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
    }

    // pub fn get_rect_from_handle(&self, handle: &TextureHandle) -> Option<(Rect, TextureHandle)> {
    //     self.rects.iter().find(|(_, x)| x == handle).copied()
    // }

    pub fn get_rect(&self, handle: RectHandle) -> Option<&Rect> {
        self.rects.get(handle.0)
    }

    // pub fn merge_bitmaps(
    //     &mut self,
    //     bitmaps: HashMap<T, Image>,
    //     keys: HashMap<T, TextureHandle>,
    // ) -> Image {
    //     let mut data: Vec<u8> = repeat(0)
    //         .take((self.width * self.height).try_into().unwrap())
    //         .collect();
    //     let handle_to_rect: HashMap<&TextureHandle, &Rect> =
    //         self.rects.iter().map(|(k, v)| (v, k)).collect();
    //     self.key_to_rect = Some(
    //         keys.into_iter()
    //             .map(|(k, v)| (k, *handle_to_rect.get(&v).copied().unwrap()))
    //             .collect(),
    //     );

    //     for (t, image) in bitmaps.into_iter() {
    //         let rect = self.key_to_rect.as_ref().unwrap().get(&t).unwrap();
    //         let offset: usize = (rect.x + rect.w * self.width).try_into().unwrap();
    //         data.splice(offset..offset + image.data.len(), image.data);
    //     }

    //     Image {
    //         data,
    //         width: self.width as usize,
    //         height: self.height as usize,
    //     }
    // }
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
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
