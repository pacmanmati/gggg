// #![feature(drain_filter)]

use std::{
    cell::RefCell,
    ops::{Add, AddAssign},
    rc::Rc,
};

use context::Context;
use shape::UIShape;
use widgets::{widget::Widget, BoxConstraints};

pub mod context;
pub mod fonts;
pub mod shape;
pub mod styles;
pub mod widgets;

pub fn build_tree(
    mut root: Box<dyn Widget>,
    constraints: BoxConstraints,
    context: Rc<RefCell<Context>>,
) -> Vec<UIShape> {
    // tree traversal - not sure whether breadth or depth is best but let's try depth

    // what if instead of calling layout (recursively) we did a look-up.
    // look-ups are cheap + memoization / caching friendly.
    // we could convert recursion into iteration via a task scheduler with dependency tracking.
    // e.g. we call layout on a root widget which has children, which attempts to look-up its children
    // the children aren't calculated (yet) so we await a future, returning when the children are done.
    // look-up cons:
    // - higher memory consumption
    // - we need a way to uniquely identify everything? we can generate ids for everything but then need to update them appropriately...
    // - maybe more complicated than it needs to be

    // let visit = VecDeque::new();

    // let current =
    // while

    root.layout(constraints, context.clone());
    root.get_shapes(&Offset { dx: 0.0, dy: 0.0 }, constraints, context.clone())
}

#[derive(Copy, Clone)]
pub struct Offset {
    pub dx: f32,
    pub dy: f32,
}

impl AddAssign for Offset {
    fn add_assign(&mut self, rhs: Self) {
        self.dx += rhs.dx;
        self.dy += rhs.dy;
    }
}

impl Add for Offset {
    type Output = Offset;

    fn add(self, rhs: Self) -> Self::Output {
        Offset {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}

impl Add for &Offset {
    type Output = Offset;

    fn add(self, rhs: Self) -> Self::Output {
        Offset {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}
