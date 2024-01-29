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
    )
```

A short list of widgets allows you to achieve a lot:
- Container
- Text
- Flex
- TextInput
- Checkbox
- GestureDetector
