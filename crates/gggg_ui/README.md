# gggg_ui
## Summary
A simple immediate-mode ui framework that exposes a flutter-like interface. UIs can be build using a syntax like this:
```rust
let ui = container()
    .color(Colors::Red)
    .width(300)
    .height(300)
    .child(text("hello world")
        .weight(FontWeight::Bold)
    );
```

Widgets - simple but powerful structs that can be composed together:
- Container
- Text
- Flex
- TextInput
- Checkbox
- GestureDetector

## Pipeline
The user creates some widgets, they form a tree structure where some are parents, others are children or siblings. The top of the tree contains one root widget, which gets converted into a drawable representation. In order to convert the root, we need to convert its children, their children, etc.

The root node gets told how much space it's allowed to occupy, then communicates that down to its children. Once the deepest child has computed its size, its parents are able to decide what their size is. Sizes can easily be fetched from a 'context' store, passed down to all components.

Once the size of a widget is known we can also extract its UIShapes representation. The UI library only represents what should be drawn, for example the tree below might return something like this:

```rust
container()
    .color(Colors::Red)
    .width(300)
    .height(300)
    .child(text("hello world")
        .weight(FontWeight::Bold)
    );
```

```
[
    UIShape::Rectangle {
        position: ...,
        size: ...,
        color: ...,
        ...
    },
    UIShape::Glyph {
        char: ...,
        font_size: ...,
        font_weight: ...,
        ...
    },
    ...
]
```

<sup><sub>UIShapes are supposed to be drawn in the order that they're given. // UIShapes contain a three dimensional position, where the z-coord defines how they should stack in a 2d context. <-- probably this is better</sub></sup>

It's then up to the shape consumer to implement how those shapes are drawn. 