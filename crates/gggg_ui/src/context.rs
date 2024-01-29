use std::{collections::HashMap, rc::Rc};

use gggg_asset::loader::Loader;

use crate::styles::text_style::{TextStyleComputed, TextStyleHandle};

/// A global 'state' that exists above widgets, as part of the current tree. [Widgets][crate::Widget] can access the tree inside [layout][crate::Widget::layout] and [get_shapes][crate::Widget::get_shapes] methods.
///
/// For example, the [Text][crate::widgets::text::Text] widgets can share data derived from [TextStyles][crate::styles::text_style::TextStyle], this is made possible by storing the data inside of [Context].
///
pub struct Context {
    pub loader: Loader,
    pub text_styles: HashMap<TextStyleHandle, Rc<TextStyleComputed>>,
}

impl Context {
    pub fn new(loader: Loader) -> Self {
        Self {
            text_styles: HashMap::new(),
            loader,
        }
    }
}
