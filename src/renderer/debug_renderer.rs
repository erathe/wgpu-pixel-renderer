use wgpu::{include_wgsl, util::DeviceExt};

use super::{
    pipeline_utils::{
        create_basic_sampler_bind_group, create_basic_sampler_bind_group_layout,
        create_render_pipeline,
    },
    texture::{self, Texture},
    Renderer,
};

pub struct DebugRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: Option<wgpu::BindGroup>,
}

impl DebugRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let texture_bind_group_layout =
            create_basic_sampler_bind_group_layout(&device, Some("debug_bg layout"));

        let shader = include_wgsl!("occluder_shader.wgsl");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            config.format,
            &[Vertex::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
            Some("debug render pipeline"),
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite renderer vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite renderer index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            texture_bind_group_layout,
            texture_bind_group: None,
        }
    }

    pub fn set_bind_group(&mut self, renderer: &Renderer, texture: &texture::Texture) {
        let bind_group = create_basic_sampler_bind_group(
            &renderer,
            &self.texture_bind_group_layout,
            &texture,
            Some("debug bg"),
        );

        self.texture_bind_group = Some(bind_group);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 1.0],
    }, // Top-left
    Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 1.0],
    }, // Top-right
    Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 0.0],
    }, // Bottom-left
    Vertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 0.0],
    }, // Bottom-right
];

const INDICES: &[u16] = &[2, 1, 0u16, 2, 3, 1];

pub(super) trait DebugTexture<'a> {
    fn draw_debug_texture(&mut self, debug_renderer: &'a DebugRenderer);
}

impl<'a, 'b> DebugTexture<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_debug_texture(&mut self, debug_renderer: &'b DebugRenderer) {
        if let Some(bind_group) = &debug_renderer.texture_bind_group {
            self.set_pipeline(&debug_renderer.pipeline);
            self.set_vertex_buffer(0, debug_renderer.vertex_buffer.slice(..));
            self.set_index_buffer(
                debug_renderer.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            self.set_bind_group(0, bind_group, &[]);
            self.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }
    }
}
