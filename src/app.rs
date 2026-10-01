use crate::graphics::GpuState;
use crate::input::Input;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{DeviceEvent, DeviceId, KeyEvent};
use winit::window::{CursorGrabMode, Fullscreen};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent::{self, CloseRequested, RedrawRequested, Resized},
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

pub struct App {
    // This is controlled by winit
    gpu: Option<GpuState>,
    input: Input,
    instant: Instant,
    avg_fps: f64,
    num: u64,
    display_time: f64,
}

impl App {
    pub fn new() -> Self {
        Self {
            gpu: None,
            input: Input::new(),
            instant: Instant::now(),
            avg_fps: 0.0,
            num: 0,
            display_time: 0.0,
        }
    }
}

impl ApplicationHandler for App {
    // necessary for winit to run the app
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            // ignore a recreation request
            return;
        }
        let window_attributes = WindowAttributes::default()
            .with_title("Wgpu Intro")
            .with_fullscreen(Some(Fullscreen::Borderless(None)));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        window
            .set_cursor_grab(CursorGrabMode::Locked)
            .expect("Couldnt lock cursor");
        window.set_cursor_visible(false);

        let gpu = pollster::block_on(GpuState::new(window));

        self.gpu = Some(gpu);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let gpu = if let Some(gpu) = self.gpu.as_mut() {
            gpu
        } else {
            return;
        };
        match event {
            Resized(size) => gpu.resize(size),
            CloseRequested => {
                event_loop.exit();
            }
            RedrawRequested => {
                let dt = self.instant.elapsed();
                let frame_time = dt.as_secs_f64() * 1000.0;
                if frame_time != 0.0 {
                    self.avg_fps = self.avg_fps * (self.num as f64 / (self.num as f64 + 1.0));
                    self.avg_fps += frame_time / (self.num as f64 + 1.0);
                    self.num += 1;
                }
                if self.display_time <= 0.0 {
                    println!("hi");
                    gpu.window
                        .set_title(format!("Ray Tracer: {:.4} ms", self.avg_fps).as_str());
                    self.display_time = 1000.0;
                    self.num = 0;
                }
                self.display_time -= frame_time;

                self.instant = Instant::now();

                self.input.update_camera(0.001, &mut gpu.camera, dt.as_secs_f32());

                gpu.render(self.input.changed);
                self.input.reset();
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: key,
                    state: pressed,
                    ..
                },
                ..
            } => {self.input.handle_key_press(key,pressed)}
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.input.mouse_delta.0 += delta.0;
                self.input.mouse_delta.1 += delta.1;
                self.input.changed = true;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(gpu) = &self.gpu {
            gpu.window.request_redraw();
        }
    }
}
