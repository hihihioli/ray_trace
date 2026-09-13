use wgpu::{include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device, FragmentState, PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, TextureSampleType, TextureView, TextureViewDimension, VertexState};

pub struct BlitResources {
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl BlitResources {
    pub fn new(device: &Device, format: TextureFormat, texture_view: &TextureView) -> Self {
        let shader = device.create_shader_module(include_wgsl!("blit.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor{
            label: Some("Blitter Bind Group Layout"),
            entries: &[BindGroupLayoutEntry{
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float {
                        filterable: false,
                    },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor{
            label: Some("Blit Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets:  &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("Blit Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry{
                binding: 0,
                resource: BindingResource::TextureView(texture_view),
            }],
        });

        Self {
            pipeline,
            bind_group,
            bind_group_layout,
        }
    }
}