use std::{cell::RefCell, rc::Rc};

use gggg_asset::loader::Loader;
use gggg_ui::{
    build_tree,
    context::Context,
    widgets::{container::container, BoxConstraints, Size},
};

fn main() {
    let ui = Box::new(container());

    let context = Rc::new(RefCell::new(Context::new(Loader::new())));

    let ui_shapes = build_tree(ui, BoxConstraints::tight(Size::new(100.0, 100.0)), context);
}
