use instant::Duration;
use wgpu::util::DeviceExt;

use crate::utils::{PipelineData, UniformData};

pub struct Animation<const N: usize> {
    time_since_last_frame: Duration,
    frame_duration: Duration,
    current_animation_index: usize,
    current_frame: usize,
    frames: [Vec<u32>; N],
    uniform: UniformData<AnimationUniform>,
    pipeline_data: PipelineData,
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
            uniform: UniformData::new(uniform, buffer),
            pipeline_data: PipelineData::new(bind_group, bind_group_layout),
        }
    }

    pub fn increment_frame(&mut self, delta: Duration) -> Option<&UniformData<AnimationUniform>> {
        self.time_since_last_frame += delta;
        if self.time_since_last_frame >= self.frame_duration {
            // update current frame
            self.current_frame =
                (self.current_frame + 1) % self.frames[self.current_animation_index].len();

            // set corresponding tile index on uniform
            self.uniform.uniform.tile_index =
                self.frames[self.current_animation_index][self.current_frame];

            self.time_since_last_frame = Duration::from_millis(0);

            // return uniform so state can write it to the queue
            return Some(&self.uniform);
        }
        None
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.pipeline_data.bind_group()
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.pipeline_data.bind_group_layout()
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
