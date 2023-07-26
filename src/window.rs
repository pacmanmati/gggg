use std::time::Instant;

use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::input::InputEvent;

pub trait AppLoop {
    type App: AppLoop;

    fn init(window: &Window) -> Self::App;

    fn draw(&mut self);

    fn input(&mut self, _input: InputEvent) {}

    fn resized(&mut self, new_size: PhysicalSize<u32>) {}
}

pub struct App {
    title: String,
    frame_rate: f32,
    size: Option<(u32, u32)>,
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
    pub fn with_window_size(mut self, size: (u32, u32)) -> Self {
        self.size = Some(size);
        self
    }

    pub fn run<T: AppLoop + 'static>(self, app_loop_init: fn(&Window) -> T) {
        let size = self.size.unwrap_or((300, 300));
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(size.0, size.1))
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
                winit::event::WindowEvent::Resized(new_size) => app_loop.resized(new_size),
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
                winit::event::WindowEvent::KeyboardInput { .. } => {
                    app_loop.input(InputEvent::keyboard_input(event))
                }
                _ => {}
            },
            winit::event::Event::DeviceEvent { event, .. } => {
                if let winit::event::DeviceEvent::MouseMotion { .. } = event {
                    app_loop.input(InputEvent::mouse_motion(event));
                }
            }
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
        size: None,
    }
}
