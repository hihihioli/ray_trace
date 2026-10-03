use bytemuck::{Pod, Zeroable, cast_slice};
use rand::random;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device};

pub struct Scene {
    pub spheres: Vec<SphereGpu>,
    pub materials: Vec<MaterialGpu>,

    pub spheres_buffer: Buffer,
    pub materials_buffer: Buffer,
}

impl Scene {
    pub fn new(device: &Device) -> Self {
        let mut spheres = vec![
            SphereGpu {
                center: [-35., 10., -10.],
                radius: 15.0,
                material_index: 4,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.4,
                center: [0.5, -0.1, -2.],
                material_index: 0,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.5,
                center: [-0.5, -0.0, -2.],
                material_index: 2,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.0,
                center: [-0.5, -0.0, -2.],
                material_index: 5,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 3.0,
                center: [-0.7, -6.0, -2.0],
                material_index: 3,
                _padding: [0; 3],
            },
        ];
        for _ in 0..30 {
            let rand_x: f32 = random::<f32>() * 20.0 + 0.3;
            let rand_z: f32 = random::<f32>() * 20.0 + 0.3;

            spheres.push(SphereGpu {
                radius: 0.4,
                center: [rand_x, -0.1, rand_z],
                material_index: 0,
                _padding: [0; 3],
            });
        }

        let materials = vec![
            MaterialGpu {
                color: [0.7, 0.7, 0.7],
                emission_strength: 0.0,
                emission_color: [1., 1., 1.0],
                smoothness: 1.0,
                ior: 1.0,
                transmission: 0.0,
                _padding: [0;2]
            },
            MaterialGpu {
                color: [0.7, 0.7, 0.7],
                emission_strength: 0.0,
                emission_color: [1., 1., 1.],
                smoothness: 0.2,
                ior: 1.0,
                transmission: 0.0,
                _padding: [0;2],
            },
            MaterialGpu {
                color: [0.0, 1.0, 0.0],
                emission_strength: 0.,
                emission_color: [0.3, 0., 1.],
                smoothness: 1.0,
                ior: 1.5,
                transmission: 1.0,
                _padding: [0;2],
            },
            MaterialGpu {
                color: [0.0, 0.0, 0.0],
                emission_strength: 5.0,
                emission_color: [1., 1., 1.],
                smoothness: 1.0,
                ior: 1.0,
                transmission: 0.0,
                _padding: [0;2],
            },
            MaterialGpu {
                color: [0.0, 0.0, 0.0],
                emission_strength: 10.0,
                emission_color: [1., 1., 1.],
                smoothness: 0.0,
                ior: 1.0,
                transmission: 0.0,
                _padding: [0;2],
            },
            MaterialGpu {
                color: [1.0, 1.0, 1.0],
                emission_strength: 0.,
                emission_color: [0.3, 0., 1.],
                smoothness: 1.0,
                ior: 1.0/1.5,
                transmission: 1.0,
                _padding: [0;2],
            },
        ];

        let spheres_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Sphere Buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            contents: cast_slice(&spheres),
        });

        let materials_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Materials Buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            contents: cast_slice(&materials),
        });

        Scene {
            spheres,
            materials,
            spheres_buffer,
            materials_buffer,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SphereGpu {
    center: [f32; 3],
    radius: f32,
    material_index: u32,
    _padding: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct MaterialGpu {
    color: [f32; 3],
    emission_strength: f32,
    emission_color: [f32; 3],
    smoothness: f32,
    ior: f32,
    transmission: f32,
    _padding: [u32;2],
}
