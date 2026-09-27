use bytemuck::{Pod, Zeroable, cast_slice};
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
        let spheres = vec![
            SphereGpu {
                radius: 0.5,
                center: [0.5, 0., -2.],
                material_index: 0,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.5,
                center: [-0.5, 0., -2.],
                material_index: 0,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 100.,
                center: [0., -100.5, -2.],
                material_index: 1,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.2,
                center: [0., -0.3, -1.7],
                material_index: 2,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.5,
                center: [-1., 1., -1.],
                material_index: 3,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.5,
                center: [-1., 1., -4.],
                material_index: 3,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.5,
                center: [1., 1., -1.],
                material_index: 3,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.5,
                center: [1., 1., -4.],
                material_index: 3,
                _padding: [0; 3],
            },
            SphereGpu {
                radius: 0.2,
                center: [-0.8,-0.4,-1.3],
                material_index: 3,
                _padding: [0;3],
            }
        ];

        let materials = vec![MaterialGpu {
            color: [0.7, 0.7, 0.7],
            emission_strength: 0.0,
            emission_color: [0., 0., 0.],
            smoothness: 0.4,
        }, MaterialGpu {
            color: [0.15, 0.3, 0.2],
            emission_strength: 0.0,
            emission_color: [0., 0., 0.],
            smoothness: 0.0,
        }, MaterialGpu {
            color: [0.2, 0.4, 0.9],
            emission_strength: 0.4,
            emission_color: [1., 0., 0.],
            smoothness: 0.2,
        }, MaterialGpu {
            color: [0.0, 0.0, 0.0],
            emission_strength: 5.0,
            emission_color: [1., 1., 1.],
            smoothness: 0.0,
        }];

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
}
