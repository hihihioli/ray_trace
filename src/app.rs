use crate::graphics::GpuState;
use crate::input::Input;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent::{self, CloseRequested, RedrawRequested, Resized},
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};
use winit::dpi::PhysicalSize;
use winit::dpi::Size::Physical;

pub struct App {
    // This is controlled by winit
    gpu: Option<GpuState>,
    input: Input,
}

impl App {
    pub fn new() -> Self {
        Self {
            gpu: None,
            input: Input::new(),
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
        let window_attributes = WindowAttributes::default().with_inner_size(Physical(PhysicalSize::new(200,300)))
            .with_title("Wgpu Intro");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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
            RedrawRequested => gpu.render(),
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(gpu) = &self.gpu {
            gpu.window.request_redraw();
        }
    }
}
