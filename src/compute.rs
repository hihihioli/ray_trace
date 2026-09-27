use crate::scene::Scene;
use bytemuck::{Pod, Zeroable};
use wgpu::BindingResource::TextureView;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, Queue, ShaderStages,
    StorageTextureAccess, TextureFormat, TextureViewDimension, include_spirv,
};

pub struct ComputeResources {
    pub pipeline: ComputePipeline,
    pub texture_bind_group: BindGroup,
    pub texture_bind_group_layout: BindGroupLayout,
    pub shader_params: ShaderParams,
    pub uniform_bind_group: BindGroup,
    pub uniform_buffer: Buffer,
}

impl ComputeResources {
    pub fn new(device: &Device, texture_view: &wgpu::TextureView, scene: &Scene) -> Self {
        let shader_params = ShaderParams::new(scene.spheres.len() as u32);
        let compute_shader = device.create_shader_module(include_spirv!("compute.spv"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Ray Tracing Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Ray Tracing Bind group"),
            layout: &texture_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: TextureView(texture_view),
            }],
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
}

impl ShaderParams {
    fn new(num_spheres: u32) -> Self {
        Self {
            frame_count: 0,
            accumulated_frames: 0,
            num_spheres,
        }
    }

    pub fn increment_frame_count(&mut self) {
        self.frame_count += 1;
        self.accumulated_frames += 1;
    }

    pub fn reset_frame_accumulation(&mut self) {
        self.accumulated_frames = 0;
    }
}
