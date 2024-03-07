use instant::Duration;
use wgpu::util::DeviceExt;

pub struct Animation<const N: usize> {
    time_since_last_frame: Duration,
    frame_duration: Duration,
    current_animation_index: usize,
    current_frame: usize,
    frames: [Vec<u32>; N],
}

impl<const N: usize> Animation<N> {
    pub fn new(device: &wgpu::Device, frames: [Vec<u32>; N], frame_duration: Duration) -> Self {
        let uniform = AnimationUniform::new(0);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animation Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("animation bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("animation Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            time_since_last_frame: Duration::from_millis(0),
            frame_duration,
            current_frame: 0,
            current_animation_index: 0,
            frames,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AnimationUniform {
    tile_index: u32,
}

impl AnimationUniform {
    pub fn new(tile_index: u32) -> Self {
        Self { tile_index }
    }
}
