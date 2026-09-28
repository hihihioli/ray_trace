use bytemuck::cast_slice;
use exr::prelude::read_first_rgba_layer_from_file;
use std::path::Path;
use wgpu::wgt::SamplerDescriptor;
use wgpu::{
    AddressMode, Device, Extent3d, FilterMode, MipmapFilterMode, Queue, Sampler,
    TexelCopyBufferLayout, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView,
};

pub struct Environment {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
}

impl Environment {
    pub fn new(device: &Device, queue: &Queue, path: impl AsRef<Path>, bin: Box<Path>) -> Self {
        let texture = create_exr_texture(&device, &queue, path);
        let texture_view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Environment Texture Sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            texture_view,
            sampler,
        }
    }
}

pub fn create_exr_texture(device: &Device, queue: &Queue, path: impl AsRef<Path>) -> Texture {
    let (pixel_data, width, height) = load_exr_rgba32(path);

    let texture_size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("EXR Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba32Float,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        cast_slice(&pixel_data),
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(16 * width),
            rows_per_image: Some(height),
        },
        texture_size,
    );

    texture
}

struct PixelBuffer {
    data: Vec<f32>,
    width: usize,
}

fn load_exr_rgba32(path: impl AsRef<Path>) -> (Vec<f32>, u32, u32) {
    let image = read_first_rgba_layer_from_file(
        path,
        |size, _| PixelBuffer {
            data: vec![0.0; size.width() * size.height() * 4],
            width: size.width(),
        },
        |pixels, position, (r, g, b, a): (f32, f32, f32, f32)| {
            let i = (position.y() * pixels.width + position.x()) * 4;

            pixels.data[i] = r;
            pixels.data[i + 1] = g;
            pixels.data[i + 2] = b;
            pixels.data[i + 3] = a;
        },
    )
    .expect("Failed to read EXR file");

    let width = image.layer_data.size.width() as u32;
    let height = image.layer_data.size.height() as u32;

    (image.layer_data.channel_data.pixels.data, width, height)
}
