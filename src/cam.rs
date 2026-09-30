use glam::{Mat3, Vec3};

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub center: Vec3,
    pub focal_length: f32,
    pub aperture: f32,
    pub focus_distance: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            center: Vec3::ZERO,
            focal_length: 0.024,
            aperture: 0.5,
            focus_distance: 2.0,
        }
    }

    pub fn rotation_matrix(&self) -> Mat3 {
        Mat3::from_rotation_y(self.yaw) * Mat3::from_rotation_x(self.pitch)
    }
}
