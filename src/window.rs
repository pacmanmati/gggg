use std::time::Instant;

use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::input::InputEvent;

pub trait AppLoop {
    type App: AppLoop;

    fn init(window: &Window) -> Self::App;

    fn draw(&mut self);

    fn input(&mut self, input: InputEvent) {}
}

pub struct App {
    title: String,
    frame_rate: f32,
}

impl App {
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_framerate(mut self, frame_rate: f32) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn run<T: AppLoop + 'static>(self, app_loop_init: fn(&Window) -> T) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(self.title)
            .build(&event_loop)
            .unwrap();

        let mut app_loop = app_loop_init(&window);

        let mut now = Instant::now();

        event_loop.run(move |event, _target, cf| match event {
            winit::event::Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                winit::event::WindowEvent::Resized(new_size) => {
                    println!("resized {:?}", new_size);
                }
                winit::event::WindowEvent::CloseRequested => cf.set_exit(),
                // winit::event::WindowEvent::AxisMotion {
                //     device_id,
                //     axis,
                //     value,
                // } => println!("axis motion {:?} {:?} {:?}", device_id, axis, value),
                winit::event::WindowEvent::MouseInput { .. } => {
                    app_loop.input(InputEvent::mouse_button(event))
                }

                winit::event::WindowEvent::MouseWheel { .. } => {
                    app_loop.input(InputEvent::mouse_wheel(event))
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => app_loop.input(InputEvent::keyboard_input(event)),
                _ => {}
            },
            winit::event::Event::DeviceEvent { device_id, event } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    app_loop.input(InputEvent::mouse_motion(event));
                }
                _ => {}
            },
            winit::event::Event::MainEventsCleared => {
                if now.elapsed().as_secs_f32() >= 1.0 / self.frame_rate {
                    now = Instant::now();
                    app_loop.draw();
                }
            }
            _ => {}
        });
    }
}

pub fn make_window() -> App {
    env_logger::init();

    App {
        title: "gggg".into(),
        frame_rate: 60.0,
    }
}
