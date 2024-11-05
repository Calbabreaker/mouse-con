use std::{process::Command, time::Duration};

use uinput::event::{
    absolute::Position,
    controller::{DPad, GamePad},
    Absolute, Controller,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
};
use winit::{
    keyboard::KeyCode,
    window::{Window, WindowId},
};

const MOUSE_SENSITIVITY: f64 = 250.;

fn key_to_controller_event(key: KeyCode) -> Option<uinput::event::Controller> {
    Some(match key {
        KeyCode::KeyC => Controller::GamePad(GamePad::B),
        KeyCode::Space => Controller::GamePad(GamePad::Y),
        KeyCode::ShiftLeft => Controller::GamePad(GamePad::A),
        KeyCode::KeyM => Controller::GamePad(GamePad::Start),
        KeyCode::KeyN => Controller::GamePad(GamePad::Select),
        KeyCode::KeyQ => Controller::GamePad(GamePad::TL),
        KeyCode::KeyE => Controller::GamePad(GamePad::TR),
        KeyCode::KeyG => Controller::GamePad(GamePad::ThumbL),
        KeyCode::KeyH => Controller::GamePad(GamePad::ThumbR),
        KeyCode::ControlLeft => Controller::GamePad(GamePad::TL2),
        KeyCode::KeyI => Controller::DPad(DPad::Up),
        KeyCode::KeyJ => Controller::DPad(DPad::Left),
        KeyCode::KeyK => Controller::DPad(DPad::Down),
        KeyCode::KeyL => Controller::DPad(DPad::Right),
        // Easy access keys
        KeyCode::KeyV => Controller::DPad(DPad::Up),
        KeyCode::KeyR => Controller::DPad(DPad::Left),
        KeyCode::KeyT => Controller::DPad(DPad::Down),
        KeyCode::KeyF => Controller::DPad(DPad::Right),
        _ => return None,
    })
}

fn mouse_button_to_controller_event(button: u32) -> Option<uinput::event::Controller> {
    Some(match button {
        1 => Controller::GamePad(GamePad::X),
        3 => Controller::GamePad(GamePad::TR2),
        _ => return None,
    })
}

fn key_to_position(key: KeyCode) -> Option<(uinput::event::absolute::Position, i32)> {
    Some(match key {
        KeyCode::KeyW => (Position::Y, -127),
        KeyCode::KeyA => (Position::X, -127),
        KeyCode::KeyS => (Position::Y, 128),
        KeyCode::KeyD => (Position::X, 128),
        _ => return None,
    })
}

struct AppState {
    window: Window,
    device: uinput::Device,
}

impl AppState {
    fn new(event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
        Ok(Self {
            window: event_loop.create_window(Window::default_attributes().with_visible(false))?,
            device: uinput::default()?
                .name("Microsoft X-Box 360 pad")?
                .event(uinput::event::Controller::All)?
                .event(uinput::event::Absolute::Position(Position::Y))?
                .min(-127)
                .max(128)
                .flat(0)
                .fuzz(0)
                .event(uinput::event::Absolute::Position(Position::X))?
                .min(-127)
                .max(128)
                .flat(0)
                .fuzz(0)
                .event(uinput::event::Absolute::Position(Position::RX))?
                .min(-127)
                .max(128)
                .flat(0)
                .fuzz(0)
                .event(uinput::event::Absolute::Position(Position::RY))?
                .min(-127)
                .max(128)
                .flat(0)
                .fuzz(0)
                .vendor(0x045e)
                .product(0x028e)
                .vendor(0x110)
                .create()?,
        })
    }

    fn send(&mut self, event: impl Into<uinput::Event>, value: i32) {
        let event = event.into();
        // eprintln!("Sending: {:?}, {value}", event);
        if let Err(err) = self.device.send(event, value) {
            eprintln!("Error while sending event: {err}");
        }

        if let Err(err) = self.device.synchronize() {
            eprintln!("Error while synchronizing event: {err}");
        }
    }

    fn do_key(&mut self, key: winit::keyboard::KeyCode, pressed: bool) {
        if let Some((position, value)) = key_to_position(key) {
            self.send(
                Absolute::Position(position),
                if pressed { value } else { 0 },
            );
        } else if let Some(uinput_event) = key_to_controller_event(key) {
            self.send(uinput_event, if pressed { 1 } else { 0 });
        }
    }

    fn do_mouse_button(&mut self, button: u32, pressed: bool) {
        if let Some(uinput_event) = mouse_button_to_controller_event(button) {
            self.send(uinput_event, if pressed { 1 } else { 0 });
        }
    }

    fn do_mouse_move(&mut self, delta: (f64, f64)) {
        self.window.set_cursor_visible(false);
        let range = 10. / MOUSE_SENSITIVITY;
        let mut stick_x = map_range(delta.0, -range, range, -127., 128.);
        let mut stick_y = map_range(delta.1, -range, range, -127., 128.) * 1.2;

        stick_x = stick_x.signum() * stick_x.abs().sqrt();
        stick_y = stick_y.signum() * stick_y.abs().sqrt();

        // Send right analog stick input through uinput
        self.send(Absolute::Position(Position::RX), stick_x as i32);
        self.send(Absolute::Position(Position::RY), stick_y as i32);
    }

    fn do_recenter(&mut self, pos1: Position, pos2: Position) {
        self.send(Absolute::Position(pos1), 0);
        self.send(Absolute::Position(pos2), 0);
    }
}

fn map_range(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

#[derive(Default)]
struct App {
    state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut state = AppState::new(event_loop).expect("Failed to create data");
        // Center joystick
        state.do_recenter(Position::X, Position::Y);
        state.do_recenter(Position::RX, Position::RY);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if event == WindowEvent::CloseRequested {
            println!("The close button was pressed; stopping");
            event_loop.exit();
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        let state = match self.state.as_mut() {
            Some(state) => state,
            None => return,
        };

        if matches!(cause, winit::event::StartCause::ResumeTimeReached { .. }) {
            state.do_recenter(Position::RX, Position::RY);
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let state = match self.state.as_mut() {
            Some(state) => state,
            None => return,
        };

        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                state.do_mouse_move(delta);
                event_loop.set_control_flow(ControlFlow::wait_duration(Duration::from_millis(20)));
            }
            winit::event::DeviceEvent::Button {
                button,
                state: button_state,
            } => {
                state.do_mouse_button(button, button_state.is_pressed());
            }
            winit::event::DeviceEvent::Key(event) => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    if key == KeyCode::Delete {
                        event_loop.exit();
                        return;
                    }
                    state.do_key(key, event.state.is_pressed());
                }
            }
            winit::event::DeviceEvent::Added {} => {}
            _ => (),
        }
    }
}
fn main() {
    let mut process = Command::new("xbanish")
        .args(["-a", "-i", "mod4", "-m", "se"])
        .spawn()
        .inspect_err(|err| eprintln!("Failed to run xbanish: {err}"));

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("Failed to create window");

    if let Ok(process) = process.as_mut() {
        process.kill().unwrap();
        process.wait().unwrap();
    }
}
