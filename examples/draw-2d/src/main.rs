use gggg::{
    render::Render,
    window::{make_window, AppLoop},
};

struct App {
    render: Render,
}

impl AppLoop for App {
    type App = App;

    fn init(window: &winit::window::Window) -> Self::App {
        let mut render = Render::new(window).unwrap();

        App { render }
    }

    fn draw(&mut self) {
        println!("draw");
    }
}

fn main() {
    make_window()
        .with_window_size((700, 700))
        .with_title("draw_2d")
        .run(App::init);
}
