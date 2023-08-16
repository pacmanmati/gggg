use crate::{
    fonts::texture_atlas::Rect,
    styles::text_style::TextStyleHandle,
    widgets::{flex::Axis, Color, Size},
    Offset,
};

pub struct UIShape {
    pub offset: Offset,
    pub size: Size,
    pub shape: ShapeType,
}

#[derive(Debug)]
pub enum ShapeType {
    Rectangle(RectangleShape),
    Glyph(GlyphShape),
}

#[derive(Debug)]
pub struct RectangleShape {
    pub color: Color,
}

#[derive(Debug)]
pub struct GlyphShape {
    pub character: char,
    pub font_family: String,
    pub color: Color,
    pub text_style_handle: TextStyleHandle,
    pub atlas_rect: Rect,
}

impl UIShape {
    pub fn offset_by(mut self, offset: &Offset) -> Self {
        self.offset += *offset;
        self
    }

    pub fn set_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn set_size_on_axis(mut self, s: f32, axis: Axis) -> Self {
        self.size.set_on_axis(s, axis);
        self
    }
}

pub trait DrainShapes {
    fn glyphs_inner(&mut self) -> Vec<GlyphShape>;
    fn glyphs(&mut self) -> Vec<UIShape>;
    fn rectangles_inner(&mut self) -> Vec<RectangleShape>;
    fn rectangles(&mut self) -> impl Iterator<Item = UIShape> + '_;
}

impl DrainShapes for Vec<UIShape> {
    fn glyphs_inner(&mut self) -> Vec<GlyphShape> {
        self.drain(..)
            .filter(|shape| matches!(shape.shape, ShapeType::Glyph(_)))
            .map(|shape| {
                if let ShapeType::Glyph(glyph) = shape.shape {
                    glyph
                } else {
                    panic!()
                }
            })
            .collect()
    }

    fn glyphs(&mut self) -> Vec<UIShape> {
        self.drain(..)
            .filter(|shape| matches!(shape.shape, ShapeType::Glyph(_)))
            .collect()
    }

    fn rectangles_inner(&mut self) -> Vec<RectangleShape> {
        self.drain(..)
            .filter(|shape| matches!(shape.shape, ShapeType::Rectangle(_)))
            .map(|shape| {
                if let ShapeType::Rectangle(rectangle) = shape.shape {
                    rectangle
                } else {
                    panic!()
                }
            })
            .collect()
    }

    fn rectangles(&mut self) -> impl Iterator<Item = UIShape> + '_ {
        self.drain(..)
            .filter(|shape| matches!(shape.shape, ShapeType::Rectangle(_)))
    }
}
