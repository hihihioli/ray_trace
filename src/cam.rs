use glam::{Mat3, Vec3};

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub center: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            center: Vec3::ZERO,
        }
    }

    pub fn rotation_matrix(&self) -> Mat3 {
        Mat3::from_rotation_x(self.pitch) * Mat3::from_rotation_y(self.yaw)
    }
}
