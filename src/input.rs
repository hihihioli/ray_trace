use crate::cam::Camera;
use glam::{Mat3, Vec3};
use std::collections::HashSet;
use winit::event::ElementState;
use winit::keyboard::{Key, KeyCode, PhysicalKey};
use crate::graphics::GpuState;

pub struct Input {
    pub keys: HashSet<PhysicalKey>,
    pub mouse_delta: (f64, f64),
    pub changed: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            changed: false,
        }
    }

    pub fn reset(&mut self) {
        self.mouse_delta = (0.0,0.0);
        self.changed = false;
    }

    pub fn update_camera(&mut self, sensitivity: f32, camera: &mut Camera, dt: f32) {
        camera.yaw -= self.mouse_delta.0 as f32 * sensitivity;
        camera.pitch -= self.mouse_delta.1 as f32 * sensitivity;

        let rotation = Mat3::from_rotation_y(camera.yaw);

        let forward = rotation * Vec3::new(0.0,0.0,-1.0);
        let right = rotation * Vec3::new(1.0,0.0,0.0);
        let up = Vec3::new(0.0,1.0,0.0);

        for key in self.keys.iter() {
            self.changed = true;
            match key {
                PhysicalKey::Code(KeyCode::KeyW) => {camera.center += forward * dt},
                PhysicalKey::Code(KeyCode::KeyS) => {camera.center -= forward * dt},
                PhysicalKey::Code(KeyCode::KeyA) => {camera.center -= right * dt},
                PhysicalKey::Code(KeyCode::KeyD) => {camera.center += right * dt},
                PhysicalKey::Code(KeyCode::Space) => {camera.center += up * dt},
                PhysicalKey::Code(KeyCode::ShiftLeft) => {camera.center -= up * dt},
                PhysicalKey::Code(KeyCode::KeyF) => {camera.focusing = true},
                _ => {}
            }
        }
    }

    pub fn handle_key_press(&mut self, key: PhysicalKey, pressed: ElementState, gpu: &mut GpuState) {
        match pressed {
            ElementState::Pressed => {self.keys.insert(key); }
            ElementState::Released => {
                self.keys.remove(&key);

                if  key == PhysicalKey::Code(KeyCode::KeyF) {
                    gpu.camera.focusing = false;
                    self.changed = true;
                    gpu.read_depth = true;
                }
            }
        }
    }
}
