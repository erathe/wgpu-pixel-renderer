use instant::Duration;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{
    animation::Animation,
    camera::Camera,
    pipeline_utils::{
        create_basic_sampler_bind_group, create_basic_sampler_bind_group_layout,
        create_render_pipeline,
    },
    resources::load_texture,
    texture_atlas::TextureAtlas,
    Vertex, INDICES, VERTICES,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct AtlasUniform {
    texture_atlas_size: [u32; 2],
    tile_index: u32,
    _padding: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    pub p: [f32; 2],
}

impl SpriteInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![2 => Float32x2];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn new(p: [f32; 2]) -> Self {
        Self { p }
    }
}

pub struct Sprite {
    texture_atlas: std::rc::Weak<TextureAtlas>,
    pipeline: wgpu::RenderPipeline,
    sampler_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pub atlas_bind_group: wgpu::BindGroup,
    pub animation: Animation<1>,
}

impl Sprite {
    pub async fn new(
        device: &wgpu::Device,
        texture_atlas: &std::rc::Rc<TextureAtlas>,
        config: &wgpu::SurfaceConfiguration,
        camera: &Camera,
        texture_atlas_size: [u32; 2],
        initial_tile: u32,
    ) -> anyhow::Result<Self> {
        let animation = Animation::new(
            &device,
            [Vec::from([0, 1, 2, 3])],
            Duration::from_millis(200),
        );

        let weak_atlas = std::rc::Rc::downgrade(&texture_atlas);

        let bind_group_layout =
            create_basic_sampler_bind_group_layout(device, Some("character::bg_layout"));

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("atlas bind group"),
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

        let shader = include_wgsl!("texture_atlas_shader.wgsl");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("character::pipeline_layout"),
            bind_group_layouts: &[
                &bind_group_layout,
                &atlas_bind_group_layout,
                camera.bind_group_layout(),
                animation.bind_group_layout(),
            ],
            // bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let atlas_uniform = AtlasUniform {
            texture_atlas_size,
            tile_index: initial_tile,
            _padding: 0,
        };

        let atlas_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Buffer"),
            contents: bytemuck::cast_slice(&[atlas_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Bind Group"),
            layout: &atlas_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: atlas_buffer.as_entire_binding(),
            }],
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let Some(texture) = weak_atlas.upgrade() else {
            panic!("coult not read texture");
        };

        let sampler_bind_group = create_basic_sampler_bind_group(
            device,
            &bind_group_layout,
            &texture.texture,
            Some("character::sampler_bind_group"),
        );

        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            config.format,
            &[Vertex::desc(), SpriteInstance::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
            Some("character::pipeline"),
        );

        Ok(Self {
            pipeline,
            sampler_bind_group,
            texture_atlas: weak_atlas,
            vertex_buffer,
            index_buffer,
            atlas_bind_group,
            animation,
        })
    }

    // pub fn draw(
    //     &self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     output: &wgpu::TextureView,
    //     camera: &Camera,
    // ) {
    //     let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    //         label: Some("Render Encoder"),
    //     });
    //     let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //         label: Some("Base::pass"),
    //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //             view: &output,
    //             resolve_target: None,
    //             ops: wgpu::Operations {
    //                 load: wgpu::LoadOp::Clear(wgpu::Color {
    //                     r: 0.1,
    //                     g: 0.2,
    //                     b: 0.3,
    //                     a: 1.0,
    //                 }),
    //                 store: wgpu::StoreOp::Store,
    //             },
    //         })],
    //         depth_stencil_attachment: None,
    //         timestamp_writes: None,
    //         occlusion_query_set: None,
    //     });

    //     pass.set_pipeline(&self.pipeline);
    //     pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    //     pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    //     pass.set_bind_group(0, &self.sampler_bind_group, &[]);
    //     pass.set_bind_group(1, &self.atlas_bind_group, &[]);
    //     pass.set_bind_group(2, &camera.bind_group(), &[]);
    //     pass.set_bind_group(3, &self.animation.bind_group(), &[]);
    //     pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

    //     drop(pass);
    //     queue.submit(Some(encoder.finish()));
    // }

    pub fn draw_instanced(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output: &wgpu::TextureView,
        camera: &Camera,
        instances: usize,
        instance_buffer: &wgpu::Buffer,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Base::pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_bind_group(0, &self.sampler_bind_group, &[]);
        pass.set_bind_group(1, &self.atlas_bind_group, &[]);
        pass.set_bind_group(2, &camera.bind_group(), &[]);
        pass.set_bind_group(3, &self.animation.bind_group(), &[]);
        pass.draw_indexed(0..INDICES.len() as u32, 0, 0..instances as _);

        drop(pass);
        queue.submit(Some(encoder.finish()));
    }
}
