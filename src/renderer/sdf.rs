use wgpu::util::DeviceExt;

use super::{texture::Texture, Renderer};

pub struct SDFPipeline<'a> {
    pipeline: wgpu::ComputePipeline,
    pub texture_a: Texture,
    pub texture_b: Texture,
    final_texture: Option<&'a Texture>,
}

impl<'a> SDFPipeline<'a> {
    pub fn new(device: &wgpu::Device, seed_texture: Texture) -> Self {
        let texture_a = seed_texture;

        let texture_b = Texture::create_2d_texture(
            device,
            texture_a.size.width,
            texture_a.size.height,
            wgpu::TextureFormat::R32Float,
            wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            Some("sdf texture b"),
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SDF shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sdf.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SDF Pipeline layout"),
            bind_group_layouts: &[
                &Self::get_texture_bind_group_layout(device),
                &Self::get_uniform_bind_group_layout(device),
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("SDF Compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            texture_a,
            texture_b,
            final_texture: None,
        }
    }

    pub fn compute_pass(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32) {
        let bind_group_a = Self::get_texture_bind_group(device, &self.texture_a, &self.texture_b);
        let bind_group_b = Self::get_texture_bind_group(device, &self.texture_b, &self.texture_a);

        let num_passes = x.ilog2();
        let mut uniform = Params {
            texture_size: [x, y],
            jump_distance: 2_u32.pow(num_passes - 1),
            run: 0,
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SDF Jump buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let buffer_bind_group = Self::get_jump_buffer_bind_group(device, &buffer);

        for i in 0..num_passes {
            let bind_group = if i % 2 == 0 {
                &bind_group_a
            } else {
                &bind_group_b
            };

            uniform.jump_distance = 2_u32.pow(num_passes - i - 1);
            uniform.run = i as i32;
            // println!("{}", uniform.jump_distance);

            queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&[uniform]));

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut compute_pass =
                    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.set_bind_group(1, &buffer_bind_group, &[]);
                compute_pass.dispatch_workgroups((x + 15) / 16, (y + 15) / 16, 1);
            }

            queue.submit(Some(encoder.finish()));
        }
    }

    fn get_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SDF bind group layout"),
            entries: &[
                // This is the sdf texture that we will write our SDF to
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        })
    }

    fn get_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SDF bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn get_texture_bind_group(
        device: &wgpu::Device,
        src_texture: &Texture,
        dest_texture: &Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf bind group"),
            layout: &Self::get_texture_bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dest_texture.view),
                },
            ],
        })
    }

    fn get_jump_buffer_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf jump buffer bind group"),
            layout: &Self::get_uniform_bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Params {
    texture_size: [u32; 2],
    jump_distance: u32,
    run: i32,
}
