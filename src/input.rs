use winit::event::{DeviceEvent, ElementState, VirtualKeyCode, WindowEvent};

#[derive(Debug)]
pub enum InputEvent {
    MouseInput(MouseInputEvent),
    KeyboardInput { key: VirtualKeyCode, pressed: bool },
}

impl InputEvent {
    pub fn mouse_wheel(event: WindowEvent) -> InputEvent {
        match event {
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase,
                ..
            } => {
                let delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(size) => {
                        (size.x as f32, size.y as f32)
                    }
                };
                InputEvent::MouseInput(MouseInputEvent::MouseScroll { delta })
            }
            _ => unreachable!(),
        }
    }
    pub fn keyboard_input(event: WindowEvent) -> InputEvent {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => InputEvent::KeyboardInput {
                key: input.virtual_keycode.unwrap(),
                pressed: input.state == ElementState::Pressed,
            },
            _ => unreachable!(),
        }
    }
    pub fn mouse_motion(event: DeviceEvent) -> InputEvent {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                InputEvent::MouseInput(MouseInputEvent::MouseMovement {
                    delta: (delta.0 as f32, delta.1 as f32),
                })
            }
            _ => unreachable!(),
        }
    }
    pub fn mouse_button(event: WindowEvent) -> InputEvent {
        match event {
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => InputEvent::MouseInput(MouseInputEvent::MouseButton {
                button: match button {
                    winit::event::MouseButton::Left => MouseButton::MouseLeft,
                    winit::event::MouseButton::Right => MouseButton::MouseRight,
                    winit::event::MouseButton::Middle => MouseButton::MouseMiddle,
                    winit::event::MouseButton::Other(_) => MouseButton::MouseLeft, // todo: handle
                },
                pressed: state == ElementState::Pressed,
            }),

            // DeviceEvent::Button { button, state } => {
            //     match button {

            //     }
            //     InputEvent::MouseInput(MouseInputEvent::MouseButton {
            //         button: ,
            //         pressed: state == ElementState::Pressed,
            //     })
            // }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum MouseInputEvent {
    MouseMovement { delta: (f32, f32) },
    MouseButton { button: MouseButton, pressed: bool },
    MouseScroll { delta: (f32, f32) },
}

#[derive(Debug)]
pub enum MouseButton {
    MouseLeft,
    MouseRight,
    MouseMiddle,
}

pub enum KeyboardInputEvent {}
