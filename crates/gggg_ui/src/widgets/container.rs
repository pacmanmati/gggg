use std::{cell::RefCell, rc::Rc};

use crate::{
    context::Context,
    shape::{RectangleShape, ShapeType, UIShape},
    Offset,
};

use super::{widget::Widget, BoxConstraints, Color, Size};

pub fn container() -> Container {
    Container::default()
}

impl Default for Container {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            color: Default::default(),
            constraints: Default::default(),
            child: Default::default(),
        }
    }
}

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

impl Container {
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn child(mut self, child: Box<dyn Widget>) -> Self {
        self.child = Some(child);
        self
    }

    pub fn constraints(mut self, constraints: BoxConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }
}
