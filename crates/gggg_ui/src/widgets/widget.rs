use std::{cell::RefCell, rc::Rc};

use crate::{context::Context, shape::UIShape, Offset};

use super::{BoxConstraints, Size};

pub trait Widget {
    fn clone_dyn(&self) -> Box<dyn Widget>;

    /// Given a set of constraints, return desired size.
    fn layout(&mut self, constraints: BoxConstraints, context: Rc<RefCell<Context>>) -> Size;

    fn get_shapes(
        &mut self,
        offset: &Offset,
        constraints: BoxConstraints,
        context: Rc<RefCell<Context>>,
    ) -> Vec<UIShape>;

    fn get_constraints(&self) -> Option<BoxConstraints>;
}

impl Clone for Box<dyn Widget> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}
