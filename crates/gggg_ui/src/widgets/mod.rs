use self::flex::Axis;

pub mod container;
pub mod flex;
pub mod text;
pub mod widget;

#[derive(Clone, Copy, Debug)]
pub enum Color {
    RGBA(f32, f32, f32, f32),
}

fn f32_to_u8_color(x: f32) -> u8 {
    (x * 255.0).floor().clamp(0.0, 255.0) as u8
}

impl Color {
    pub fn as_rgba_f32(&self) -> [f32; 4] {
        match self {
            Color::RGBA(r, g, b, a) => [*r, *g, *b, *a],
        }
    }

    pub fn as_rgba_u8(&self) -> [u8; 4] {
        match self {
            Color::RGBA(r, g, b, a) => [
                f32_to_u8_color(*r),
                f32_to_u8_color(*g),
                f32_to_u8_color(*b),
                f32_to_u8_color(*a),
            ],
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::RGBA(0.0, 0.0, 0.0, 1.0)
    }
}

#[derive(Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }

    pub fn constrain(&self, constraints: BoxConstraints) -> Self {
        Size {
            width: self
                .width
                .clamp(constraints.min.width, constraints.max.width),
            height: self
                .height
                .clamp(constraints.min.height, constraints.max.height),
        }
    }

    pub fn set_on_axis(&mut self, s: f32, axis: Axis) {
        match axis {
            Axis::Horizontal => self.width = s,
            Axis::Vertical => self.height = s,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BoxConstraints {
    min: Size,
    max: Size,
}

impl BoxConstraints {
    pub fn new(min: Size, max: Size) -> Self {
        Self { min, max }
    }

    pub fn tight(size: Size) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    pub fn min_on_axis(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => self.min.width,
            Axis::Vertical => self.min.height,
        }
    }

    pub fn max_on_axis(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => self.max.width,
            Axis::Vertical => self.max.height,
        }
    }

    pub fn bound_max(&self, axis: Axis) -> bool {
        match axis {
            Axis::Horizontal => self.max.width.is_finite(),
            Axis::Vertical => self.max.height.is_finite(),
        }
    }

    pub fn bound_min(&self, axis: Axis) -> bool {
        match axis {
            Axis::Horizontal => self.min.width > 0.0,
            Axis::Vertical => self.min.height > 0.0,
        }
    }
}
