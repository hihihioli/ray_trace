use crate::environment::Environment;
use crate::scene::Scene;
use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Mat3A, Mat4};
use wgpu::BindingResource::{Sampler, TextureView};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, Queue, SamplerBindingType,
    ShaderStages, StorageTextureAccess, TextureFormat, TextureSampleType, TextureViewDimension,
    include_spirv,
};
use crate::cam::Camera;

pub struct ComputeResources {
    pub pipeline: ComputePipeline,
    pub texture_bind_group: BindGroup,
    pub texture_bind_group_layout: BindGroupLayout,
    pub shader_params: ShaderParams,
    pub uniform_bind_group: BindGroup,
    pub uniform_buffer: Buffer,
}

impl ComputeResources {
    pub fn new(
        device: &Device,
        texture_view: &wgpu::TextureView,
        scene: &Scene,
        environment: &Environment,
        camera: &Camera,
    ) -> Self {
        let shader_params = ShaderParams::new(scene.spheres.len() as u32, camera);
        let compute_shader = device.create_shader_module(include_spirv!("compute.spv"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Ray Tracing Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Ray Tracing Bind group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: TextureView(&environment.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: Sampler(&environment.sampler),
                },
            ],
        });

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Shader Parameters Uniform Buffer"),
            contents: bytemuck::cast_slice(&[shader_params]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Shader Parameters Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Shader Parameters Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: scene.spheres_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: scene.materials_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Ray Tracing Pipeline Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Ray Tracing Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            texture_bind_group,
            texture_bind_group_layout,
            shader_params,
            uniform_bind_group,
            uniform_buffer,
        }
    }

    pub fn update_uniform_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.shader_params]),
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShaderParams {
    frame_count: u32,
    accumulated_frames: u32,
    num_spheres: u32,
    focal_length: f32,
    camera_center: [f32; 3],
    _padding: u32,
    rotation_matrix: [[f32; 4]; 3],
    aperture: f32,
    focus_distance: f32,
    _padding1: [u32;2],
}

impl ShaderParams {
    fn new(num_spheres: u32, camera: &Camera) -> Self {
        Self {
            frame_count: 0,
            accumulated_frames: 0,
            num_spheres,
            focal_length: camera.focal_length,
            rotation_matrix: mat3_to_padded(camera.rotation_matrix()),
            camera_center: camera.center.into(),
            _padding: 0,
            aperture: camera.aperture,
            focus_distance: camera.focus_distance,
            _padding1: [0;2],
        }
    }

    pub fn increment_frame_count(&mut self) {
        self.frame_count += 1;
        self.accumulated_frames += 1;
    }

    pub fn reset_frame_accumulation(&mut self) {
        self.accumulated_frames = 0;
    }

    pub fn update_cam(&mut self, camera: &Camera) {
        self.rotation_matrix = mat3_to_padded(camera.rotation_matrix());
        self.camera_center = camera.center.into();

    }
}

fn mat3_to_padded(matrix: Mat3) -> [[f32;4];3] {
    let cols = matrix.to_cols_array_2d();

     [
        [cols[0][0], cols[0][1], cols[0][2], 0.0],
        [cols[1][0], cols[1][1], cols[1][2], 0.0],
        [cols[2][0], cols[2][1], cols[2][2], 0.0],
    ]

}