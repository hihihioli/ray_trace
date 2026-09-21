use crate::blit::BlitResources;
use crate::compute::ComputeResources;
use std::sync::Arc;
use wgpu::BindingResource::TextureView;
use wgpu::{BindGroupDescriptor, BindGroupEntry, CommandEncoderDescriptor, ComputePassDescriptor, CurrentSurfaceTexture::{Lost, Occluded, Outdated, Suboptimal, Success, Timeout, Validation}, Device, DeviceDescriptor, Extent3d, Instance, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureViewDescriptor};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct GpuState {
    // This is controlled by wgpu
    pub window: Arc<Window>,
    size: PhysicalSize<u32>,

    instance: Instance,
    surface: Surface<'static>,

    device: Device,
    queue: Queue,

    config: SurfaceConfiguration,

    out_texture: Texture,
    compute_resources: ComputeResources,
    blit_resources: BlitResources,
}

impl GpuState {
    pub async fn new(window: Arc<Window>) -> Self {
        // initialize everything
        let size = window.inner_size();

        let instance = Instance::default(); // this interfaces into wgpu. everything stems from here

        // surface is created by wgpu using our winit window
        let surface = instance.create_surface(window.clone()).unwrap(); // this is what we render to

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface), // make sure the chosen gpu is compatible with our window
                ..Default::default()
            })
            .await
            .unwrap(); // this represents the underlying gpu and driver
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .unwrap(); // a connection to the gpu and what we submit commands to
        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap(); // get the config
        config.present_mode = PresentMode::AutoVsync;
        println!("{:?}",config.format);
        
        println!("Adapter info {:?}", adapter.get_info());

        surface.configure(&device, &config);

        let out_texture = create_output_texture(&device,&size);
        let texture_view = out_texture.create_view(&Default::default());

        let compute_resources = ComputeResources::new(&device, &texture_view);

        let blit_resources = BlitResources::new(&device, config.format, &texture_view);


        Self {
            window,
            size,
            instance,
            surface,
            device,
            queue,
            config,
            out_texture,
            compute_resources,
            blit_resources,
        }
    }

    pub fn configure_surface(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let size = PhysicalSize::new(new_size.width.max(1), new_size.height.max(1));
        self.config.width = size.width;
        self.config.height = size.height;
        self.size = size;

        self.configure_surface(); //reconfigure the surface with the new dimensions

        // Recreate output texture
        let out_texture = create_output_texture(&self.device,&self.size);
        let view = out_texture.create_view(&Default::default());

        self.out_texture = out_texture;
        self.compute_resources.texture_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Ray Tracing Bind group"),
            layout: &self.compute_resources.texture_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: TextureView(&view),
            }],
        });
        self.blit_resources.bind_group = self.device.create_bind_group(&BindGroupDescriptor{
            label: Some("Blit Bind Group"),
            layout: &self.blit_resources.bind_group_layout,
            entries: &[BindGroupEntry{
                binding: 0,
                resource: TextureView(&view),
            }],
        });

        self.window.request_redraw();
    }

    pub fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Success(frame) => frame,      //the frame
            Timeout | Occluded => return, // try again later
            Suboptimal(frame) => {
                // reconfigure surface
                drop(frame);
                self.configure_surface();
                return;
            }
            Outdated => {
                self.configure_surface();
                return;
            } // reconfigure the surface
            Lost => {
                // recreate the surface
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.configure_surface();
                return;
            }
            Validation => {
                panic!("Surface validation error")
            }
        };

        let frame_view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Ray Tracing Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_resources.pipeline);
            cpass.set_bind_group(0, &self.compute_resources.texture_bind_group, &[]);
            cpass.set_bind_group(1, &self.compute_resources.uniform_bind_group, &[]);

            cpass.dispatch_workgroups(
                self.size.width.div_ceil(8),
                self.size.height.div_ceil(8),
                1
            );
        }

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor{
                label: Some("Blit Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment{
                    view: &frame_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Default::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rpass.set_pipeline(&self.blit_resources.pipeline);
            rpass.set_bind_group(0,&self.blit_resources.bind_group, &[]);

            rpass.draw(0..3, 0..1)
        }

        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        self.queue.present(frame);

        self.compute_resources.shader_params.increment_frame_count();
        self.compute_resources.update_uniform_buffer(&self.queue);
    }
}

fn create_output_texture(device: &Device, size: &PhysicalSize<u32>) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Ray Traced HDR Output"),
        size: Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}
