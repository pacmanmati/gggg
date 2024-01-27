use std::{error::Error, time::Instant};

use winit::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::input::InputEvent;

pub trait AppLoop {
    type App: AppLoop;

    fn init(window: &Window, gggg: &App) -> Self::App;

    fn draw(&mut self, gggg: &App);

    fn input(&mut self, _input: InputEvent, gggg: &App) {}

    fn resized(&mut self, new_size: PhysicalSize<u32>) {}
}

pub struct App {
    title: String,
    frame_rate: f32,
    size: Option<(u32, u32)>,
    mouse_position: PhysicalPosition<f64>,
}

impl App {
    pub fn get_mouse_position(&self) -> (f32, f32) {
        self.mouse_position.into()
    }

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

    pub fn run<T: AppLoop + 'static>(
        mut self,
        app_loop_init: fn(&Window, &App) -> T,
    ) -> Result<(), Box<dyn Error>> {
        let size = self.size.unwrap_or((300, 300));
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(size.0, size.1))
            .with_title(self.title.clone())
            .build(&event_loop)
            .unwrap();

        let mut app_loop = app_loop_init(&window, &self);

        let mut now = Instant::now();

        Ok(event_loop.run(move |event, target| match event {
            winit::event::Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                winit::event::WindowEvent::Resized(new_size) => app_loop.resized(new_size),
                winit::event::WindowEvent::CloseRequested => target.exit(),
                // winit::event::WindowEvent::AxisMotion {
                //     device_id,
                //     axis,
                //     value,
                // } => println!("axis motion {:?} {:?} {:?}", device_id, axis, value),
                winit::event::WindowEvent::MouseInput { .. } => {
                    app_loop.input(InputEvent::mouse_button(event), &self)
                }
                winit::event::WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                } => self.mouse_position = position,
                winit::event::WindowEvent::MouseWheel { .. } => {
                    app_loop.input(InputEvent::mouse_wheel(event), &self)
                }
                winit::event::WindowEvent::KeyboardInput { .. } => {
                    app_loop.input(InputEvent::keyboard_input(event), &self)
                }
                _ => {}
            },
            winit::event::Event::DeviceEvent { event, .. } => {
                if let winit::event::DeviceEvent::MouseMotion { .. } = event {
                    app_loop.input(InputEvent::mouse_motion(event), &self);
                }
            }
            winit::event::Event::AboutToWait => {
                if now.elapsed().as_secs_f32() >= 1.0 / self.frame_rate {
                    now = Instant::now();
                    app_loop.draw(&self);
                }
            }
            _ => {}
        })?)
    }
}

pub fn make_window() -> App {
    env_logger::init();

    App {
        title: "gggg".into(),
        frame_rate: 60.0,
        size: None,
        mouse_position: PhysicalPosition::default(),
    }
}
