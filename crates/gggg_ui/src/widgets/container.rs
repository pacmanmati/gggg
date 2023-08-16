use std::{cell::RefCell, f32::INFINITY, rc::Rc};

use crate::{
    context::{self, Context},
    shape::{RectangleShape, ShapeType, UIShape},
    Offset,
};

use super::{widget::Widget, BoxConstraints, Color, Size};

#[derive(Clone)]
pub struct Container {
    width: f32,
    height: f32,
    color: Color,
    constraints: Option<BoxConstraints>,
    child: Option<Box<dyn Widget>>,
}

impl Widget for Container {
    fn clone_dyn(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }

    fn layout(
        &mut self,
        constraints: super::BoxConstraints,
        context: Rc<RefCell<Context>>,
    ) -> super::Size {
        // what about the child?
        // parent size constrains, child fills the parent (in flutter)
        // in effect the parent's size becomes the child's constraints
        // however - if our size was unbounded (infinite) we would depend on our child's size?
        // ok so a container without a size will fill its box constraints unless it has a child, then it will use the child's constraints
        if let Some(child) = &mut self.child {
            let child_size = child.layout(constraints, context);
            Size::new(
                if self.width.is_finite() {
                    self.width
                } else {
                    child_size.width
                },
                if self.height.is_finite() {
                    self.height
                } else {
                    child_size.height
                },
            )
            .constrain(constraints)
        } else {
            Size::new(self.width, self.height).constrain(constraints)
        }
    }

    fn get_shapes(
        &mut self,
        offset: &Offset,
        constraints: super::BoxConstraints,
        context: Rc<RefCell<Context>>,
    ) -> Vec<UIShape> {
        let size = Size {
            width: self.width,
            height: self.height,
        }
        .constrain(constraints);
        vec![UIShape {
            offset: *offset,
            size,
            shape: ShapeType::Rectangle(RectangleShape { color: self.color }),
        }]
    }

    fn get_constraints(&self) -> Option<super::BoxConstraints> {
        self.constraints
    }
}

// impl Clone for Container {
//     fn clone(&self) -> Self {
//         Self {
//             width: self.width.clone(),
//             height: self.height.clone(),
//             color: self.color.clone(),
//             child: self.child.,
//         }
//     }
// }

#[derive(Default)]
pub struct ContainerBuilder {
    width: Option<f32>,
    height: Option<f32>,
    color: Option<Color>,
    constraints: Option<BoxConstraints>,
    child: Option<Box<dyn Widget>>,
}

impl ContainerBuilder {
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_child(mut self, child: Box<dyn Widget>) -> Self {
        self.child = Some(child);
        self
    }

    pub fn with_constraints(mut self, constraints: BoxConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    pub fn build(self) -> Container {
        Container {
            width: self.width.unwrap_or(INFINITY),
            height: self.height.unwrap_or(INFINITY),
            color: self.color.unwrap_or(Color::RGBA(0.0, 0.0, 0.0, 0.0)),
            child: self.child,
            constraints: self.constraints,
        }
    }
}

impl Container {
    pub fn new() -> ContainerBuilder {
        ContainerBuilder::default()
    }
}
