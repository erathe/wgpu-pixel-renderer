use wgpu::{include_wgsl, util::DeviceExt};

use super::{
    pipeline_utils::{
        create_basic_sampler_bind_group, create_basic_sampler_bind_group_layout,
        create_render_pipeline,
    },
    Texture,
};

pub struct OutputNode {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    sampler_bind_group: wgpu::BindGroup,
}

impl OutputNode {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sampler: &wgpu::Sampler,
        target: &Texture,
    ) -> Self {
        let sampler_bind_group_layout =
            create_basic_sampler_bind_group_layout(&device, Some("output bg layout"));

        let sampler_bind_group = create_basic_sampler_bind_group(
            &device,
            &sampler,
            &sampler_bind_group_layout,
            &target,
            Some("output bg"),
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite renderer pipeline layout"),
            bind_group_layouts: &[&sampler_bind_group_layout],
            push_constant_ranges: &[],
        });

        // buffers
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

        let shader = include_wgsl!("output.wgsl");

        let pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            config.format,
            &[Vertex::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
            Some("sprite renderer pipeline"),
        );

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            sampler_bind_group,
        }
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
        tex_coords: [0.0, 0.0],
    }, // Top-left
    Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0],
    }, // Top-right
    Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0],
    }, // Bottom-left
    Vertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0],
    }, // Bottom-right
];

const INDICES: &[u16] = &[2, 1, 0u16, 2, 3, 1];

pub(super) trait DrawToScreen<'a> {
    fn draw_to_screen(&mut self, output_renderer: &'a OutputNode);
}

impl<'a, 'b> DrawToScreen<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_to_screen(&mut self, output_renderer: &'b OutputNode) {
        self.set_pipeline(&output_renderer.pipeline);
        self.set_vertex_buffer(0, output_renderer.vertex_buffer.slice(..));
        self.set_index_buffer(
            output_renderer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        self.set_bind_group(0, &output_renderer.sampler_bind_group, &[]);
        self.draw_indexed(0..INDICES.len() as u32, 0, 0..1)
    }
}
