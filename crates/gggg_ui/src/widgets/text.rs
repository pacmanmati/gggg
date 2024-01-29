use std::{cell::RefCell, rc::Rc};

use gggg_asset::loader::Loader;

use crate::{
    context::Context,
    shape::{GlyphShape, ShapeType, UIShape},
    styles::text_style::TextStyle,
};

use super::{widget::Widget, BoxConstraints, Size};

// TODO: implement this widget (and others) as a macro, this will eliminate the Box problem and make it easier to write ui.

#[derive(Clone)]
struct LetterRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Clone)]
pub struct Text {
    value: String,
    style: TextStyle,
    letter_bounds: Option<Vec<LetterRect>>,
}

impl Text {
    pub fn new(value: String) -> Self {
        Self {
            value,
            style: TextStyle::default(),
            letter_bounds: None,
        }
    }

    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Text {
    /// Here we attempt to fit our characters within the bounding box.
    /// [Text] will layout such that word's characters will prefer to be together on the same line, putting the word on another line if necessary.
    /// If there isn't enough space, we can (optionally) insert an ellipsis to communicate that the text is cut off.
    fn layout(&mut self, constraints: BoxConstraints, context: Rc<RefCell<Context>>) -> Size {
        let mut letter_bounds = Vec::new();
        let loader = Loader::new();
        let computed = self.style.compute(&context);
        let mut max_line_width = 0_f32;
        let mut line_width = 0_f32;
        let line_height = self.style.font_size * 1.5;
        let mut y = 0_f32;

        // separate our string by spaces and newlines
        for word in self.value.split(' ').flat_map(|char| char.split('\n')) {
            let mut word_letter_bounds: Vec<LetterRect> = Vec::new();

            // handle the newline character
            if word.starts_with('\n') {
                line_width = 0.0;
                y += line_height;
                continue;
            }

            let mut word_width = 0_f32;
            let mut word_height = 0_f32;
            // we want to go through each letter and check if the entire word fits on this line
            for char in word.chars() {
                let metrics = computed.get_char_metrics(&char);
                let rect = computed.get_char_rect(char);
                // check whether we have to break the word
                if word_width > constraints.max.width {
                    y += line_height;
                    max_line_width = max_line_width.max(line_width);
                    line_width = 0.0;
                    word_width = 0.0;
                    word_height = 0.0;
                }
                word_letter_bounds.push(LetterRect {
                    x: word_width,
                    y: line_height,
                    w: rect.w as f32,
                    h: rect.h as f32,
                });
                word_width += metrics.advance_width;
                word_height = word_height.max(metrics.advance_height);
            }
            if word_width + line_width > constraints.max.width {
                if word_height + line_height + y <= constraints.max.width {
                    // if the word won't fit and we have enough vertical space, break the word onto the next line
                    y += line_height;
                    max_line_width = max_line_width.max(line_width);
                    line_width = 0.0;

                    // update the line heights for this word
                    word_letter_bounds
                        .iter_mut()
                        .for_each(|rect| rect.y = line_height);
                } else {
                    // if we don't have enough vertical space we need to chop
                    break;
                }
            }
            // finally, we need to append this word's letters onto our letters
            letter_bounds.extend(word_letter_bounds);
        }
        self.letter_bounds = Some(letter_bounds);

        Size {
            width: max_line_width,
            height: y,
        }
    }

    fn get_shapes(
        &mut self,
        offset: &crate::Offset,
        constraints: super::BoxConstraints,
        context: Rc<RefCell<Context>>,
    ) -> Vec<crate::shape::UIShape> {
        let handle = self.style.handle();
        let ctx = context.borrow();
        let computed = ctx.text_styles.get(&handle).unwrap();

        self.letter_bounds
            .as_ref()
            .unwrap()
            .clone()
            .iter()
            .enumerate()
            .map(|(idx, letter)| {
                let c = self.value.chars().nth(idx).unwrap();
                let rect = computed.get_char_rect(c);
                UIShape {
                    offset: crate::Offset {
                        dx: letter.x,
                        dy: letter.y,
                    },
                    size: Size {
                        width: letter.w,
                        height: letter.h,
                    },
                    shape: ShapeType::Glyph(GlyphShape {
                        character: c,
                        font_family: self.style.font_family.clone(),
                        color: self.style.color,
                        atlas_rect: rect,
                        text_style_handle: handle,
                    }),
                }
            })
            .collect()
    }

    fn get_constraints(&self) -> Option<super::BoxConstraints> {
        None
    }

    fn clone_dyn(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}
