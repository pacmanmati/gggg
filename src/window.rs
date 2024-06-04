use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::input::InputEvent;

pub trait AppLoop
where
    Self: Sized,
{
    type App: AppLoop;

    fn init(window: Arc<Window>, gggg: &App<Self>) -> Self::App;

    fn draw(&mut self, gggg: &App<Self>);

    fn input(&mut self, _input: InputEvent, gggg: &App<Self>) {}

    fn resized(&mut self, new_size: PhysicalSize<u32>) {}
}

type AppLoopInitFn<T> = fn(Arc<Window>, &App<T>) -> T;

pub struct App<T: AppLoop + 'static> {
    title: String,
    window: Option<Arc<Window>>,
    frame_rate: f32,
    size: Option<(u32, u32)>,
    mouse_position: PhysicalPosition<f64>,
    last_frame: Instant,
    app_loop_init: Option<AppLoopInitFn<T>>,
    app_loop: Option<T>,
}

impl<T: AppLoop + 'static> App<T> {
    fn default() -> Self {
        Self {
            frame_rate: 60.0,
            window: None,
            mouse_position: PhysicalPosition::default(),
            size: None,
            title: "gggg".into(),
            last_frame: Instant::now(),
            app_loop: None,
            app_loop_init: None,
        }
    }
}

impl<T: AppLoop + 'static> ApplicationHandler for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        self.window = Some(window.clone());
        window.request_redraw();
        self.app_loop = Some(self.app_loop_init.unwrap()(window, &self));
        // self.painter = Some(Painter::new(
        //     window.clone(),
        //     window.clone().inner_size().into(),
        // ));
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        match cause {
            winit::event::StartCause::Poll => {
                self.window.as_ref().unwrap().request_redraw();
            }
            winit::event::StartCause::ResumeTimeReached {
                start: _,
                requested_resume: _,
            } => {
                self.window.as_ref().unwrap().request_redraw();
                self.last_frame = Instant::now()
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let mut app_loop = self.app_loop.take().unwrap();

        match event {
            DeviceEvent::MouseMotion { .. } => {
                app_loop.input(InputEvent::mouse_motion(event), self)
            }
            _ => {}
        }
        self.app_loop = Some(app_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // let mut last_frame = Instant::now();
        let frame_duration = 1.0 / self.frame_rate; // seconds
        let mut app_loop = self.app_loop.take().unwrap();
        match event {
            // inputs
            WindowEvent::MouseInput { .. } => app_loop.input(InputEvent::mouse_button(event), self),
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => self.mouse_position = position,
            WindowEvent::MouseWheel { .. } => app_loop.input(InputEvent::mouse_wheel(event), self),
            WindowEvent::KeyboardInput { .. } => {
                app_loop.input(InputEvent::keyboard_input(event), self)
            }

            // other
            // WindowEvent::Resized(new_size) => self.app_loop.as_mut().,
            WindowEvent::Resized(new_size) => app_loop.resized(new_size),
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                // FIX: temporary workaround for macos bug https://github.com/rust-windowing/winit/issues/3668#issuecomment-2094976299
                let _ = self.window.take();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // println!("redraw");
                // update(&self); // do work
                // self.painter.as_ref().unwrap().paint();
                app_loop.draw(self);

                // if vsync disabled do this, otherwise use poll and set vsync in wgpu
                let this_frame = Instant::now();

                let time_used = (this_frame - self.last_frame).as_secs_f32(); // how long did it take?
                let time_remaining = frame_duration - time_used; // 0.01666667 - time_used
                                                                 // let actual_framerate = 1.0 / (time_used + time_remaining);
                                                                 // println!("actual framerate {actual_framerate}");

                if time_remaining > 0.0 {
                    // wait out remaining time
                    // println!("waiting for {time_remaining}");

                    event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                        this_frame + (Duration::from_secs_f32(time_remaining)),
                    ));
                } else {
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
                    self.last_frame = Instant::now();
                }
            }
            _ => (),
        }
        self.app_loop = Some(app_loop)
    }
}

impl<T: AppLoop + 'static> App<T> {
    pub fn get_mouse_position(&self) -> (f32, f32) {
        self.mouse_position.into()
    }

    // pub fn run<T: AppLoop + 'static>(
    //     mut self,
    //     app_loop_init: fn(&Window, &App<T>) -> T,
    // ) -> Result<(), Box<dyn Error>> {
    //     let size = self.size.unwrap_or((300, 300));
    //     let event_loop = EventLoop::new().unwrap();
    //     let window = WindowBuilder::new()
    //         .with_inner_size(LogicalSize::new(size.0, size.1))
    //         .with_title(self.title.clone())
    //         .build(&event_loop)
    //         .unwrap();

    //     let mut app_loop = app_loop_init(&window, &self);

    //     let mut now = Instant::now();

    //     Ok(event_loop.run(move |event, target| match event {
    //         winit::event::Event::WindowEvent {
    //             window_id: _,
    //             event,
    //         } => match event {
    //             winit::event::WindowEvent::Resized(new_size) => app_loop.resized(new_size),
    //             winit::event::WindowEvent::CloseRequested => target.exit(),
    //             // winit::event::WindowEvent::AxisMotion {
    //             //     device_id,
    //             //     axis,
    //             //     value,
    //             // } => println!("axis motion {:?} {:?} {:?}", device_id, axis, value),
    //             winit::event::WindowEvent::MouseInput { .. } => {
    //                 app_loop.input(InputEvent::mouse_button(event), &self)
    //             }
    //             winit::event::WindowEvent::CursorMoved {
    //                 device_id: _,
    //                 position,
    //             } => self.mouse_position = position,
    //             winit::event::WindowEvent::MouseWheel { .. } => {
    //                 app_loop.input(InputEvent::mouse_wheel(event), &self)
    //             }
    //             winit::event::WindowEvent::KeyboardInput { .. } => {
    //                 app_loop.input(InputEvent::keyboard_input(event), &self)
    //             }
    //             _ => {}
    //         },
    //         winit::event::Event::DeviceEvent { event, .. } => {
    //             if let winit::event::DeviceEvent::MouseMotion { .. } = event {
    //                 app_loop.input(InputEvent::mouse_motion(event), &self);
    //             }
    //         }
    //         winit::event::Event::AboutToWait => {
    //             if now.elapsed().as_secs_f32() >= 1.0 / self.frame_rate {
    //                 now = Instant::now();
    //                 app_loop.draw(&self);
    //             }
    //         }
    //         _ => {}
    //     })?)
    // }
}

#[derive(Default)]
pub struct AppBuilder {
    title: Option<String>,
    frame_rate: Option<f32>,
    window_size: Option<(u32, u32)>,
}

impl AppBuilder {
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_framerate(mut self, frame_rate: f32) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }
    pub fn with_window_size(mut self, size: (u32, u32)) -> Self {
        self.window_size = Some(size);
        self
    }

    pub fn run<T: AppLoop + 'static>(self, app_loop_init: AppLoopInitFn<T>) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App::default();

        app.app_loop_init = Some(app_loop_init);
        let _ = event_loop.run_app(&mut app);
    }
}

pub fn make_app() -> AppBuilder {
    env_logger::init();

    AppBuilder::default()
}
