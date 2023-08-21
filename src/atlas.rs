use std::hash::Hash;

use generational_arena::{Arena, Index};
use itertools::Itertools;

use crate::texture::TextureFormat;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct RectHandle(pub Index);

#[derive(Debug)]
pub struct Atlas {
    rects: Arena<Rect>,
    pub width: u32,
    pub height: u32,
    pub changed: bool,
    pub format: TextureFormat,
}

impl Atlas {
    pub fn new(format: TextureFormat) -> Self {
        Self {
            rects: Arena::new(),
            width: 0,
            height: 0,
            changed: false,
            format,
        }
    }

    pub fn add(&mut self, w: u32, h: u32) -> RectHandle {
        let rect = Rect { x: 0, y: 0, w, h };
        let index = self.rects.insert(rect);

        RectHandle(index)
    }

    pub fn pack(&mut self) {
        self.changed = true;
        let mut x = 0;
        let mut y = 0;
        // self.width = 512;
        let total_area = self
            .rects
            .iter()
            .fold(0.0, |acc, (_, rect)| (rect.w * rect.h) as f32 + acc);
        self.width = total_area.sqrt().round() as u32;
        // sort s.t. the tallest rect is first
        // decreasing rect height means we can pack everything in a row
        let mut sorted_rects = self
            .rects
            .iter_mut()
            .sorted_by(|a, b| b.1.h.partial_cmp(&a.1.h).unwrap())
            // .rev()
            .collect::<Vec<_>>();

        let mut max_h = sorted_rects.first().unwrap().1.h;
        // for (_, rect) in self.rects.iter_mut() {
        for (_, rect) in sorted_rects.iter_mut() {
            // bounds check
            if x + rect.w >= self.width {
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

    pub fn get_rect(&self, handle: RectHandle) -> Option<&Rect> {
        self.rects.get(handle.0)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

// impl Ord for Rect {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         (self.h).cmp(&other.h)
//     }
// }

// impl PartialOrd for Rect {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.h.partial_cmp(&other.h)
//     }
// }
