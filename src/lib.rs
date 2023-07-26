pub mod atlas;
pub mod bind;
pub mod camera;
pub mod geometry;
pub mod gltf;
pub mod input;
pub mod instance;
pub mod material;
pub mod pipeline;
pub mod plain;
pub mod render;
pub mod render_object;
pub mod shapes;
pub mod texture;
pub mod window;

// how is ui going to work?
// so far, most of this stuff could go in a `render` crate
// we might also want an `app` crate that contains the window and AppLoop stuff
//
// for ui, i think it would be nice to wrap it in a macro
// ui! {
//     Flex::new().with_axis(Axis::Vertical).with_flex_child(
//         Container::new()
//         .with_color(Color::RGBA(1.0, 1.0, 0.0, 1.0))
//         .with_height(250.0)
//         .with_width(250.0)
//         .build(),
//     )
// }
//
// the ui is what would get sent to the renderer to draw
// you could then make the renderer draw to a texture that gets embedded inside of a ui viewport component
// which would just be nested inside of the `ui! {}` macro like any other ui component.
//
// let's build/import the ui crate after a refactor into render + app crates
// then keep the ui crate strictly ui (any rendering of the ui is done by a pipeline whose code is defined inside of render)
// we'll provide a shader etc. for things like drawing text, drawing the ui, etc.
// then there'll be an easy way to plug the ui into the renderer (optional, via a trait that needs importing)
//
// multiple ui layouts:
// it should be possible to define different `ui! {}`s for various parts of an app.
// we will need a higher level `scene` construct to manage that
// a scene would also manage the objects being drawn / simulated in the app
