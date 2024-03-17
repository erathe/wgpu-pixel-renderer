use wgpu::{include_wgsl, util::DeviceExt};

use super::{
    pipeline_utils::{
        create_basic_sampler_bind_group, create_basic_sampler_bind_group_layout,
        create_render_pipeline,
    },
    texture::Texture,
};

pub struct OccluderRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: Option<wgpu::BindGroup>,
}

impl OccluderRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let texture_bind_group_layout =
            create_basic_sampler_bind_group_layout(&device, Some("Occluder bg layout"));

        let shader = include_wgsl!("occluder_shader.wgsl");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Occluder pipeline layout"),
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
            Some("occluder render pipeline"),
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

    pub fn set_bind_group(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        occluder_data: Vec<u8>,
        width: u32,
        height: u32,
    ) {
        let texture = Texture::from_data(device, queue, &occluder_data, width, height);
        let bind_group = create_basic_sampler_bind_group(
            device,
            &self.texture_bind_group_layout,
            &texture,
            Some("occluder bg"),
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

pub(super) trait DrawOccluder<'a> {
    fn draw_occluder(&mut self, occluder_renderer: &'a OccluderRenderer);
}

impl<'a, 'b> DrawOccluder<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_occluder(&mut self, occluder_renderer: &'b OccluderRenderer) {
        if let Some(bind_group) = &occluder_renderer.texture_bind_group {
            self.set_pipeline(&occluder_renderer.pipeline);
            self.set_vertex_buffer(0, occluder_renderer.vertex_buffer.slice(..));
            self.set_index_buffer(
                occluder_renderer.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            self.set_bind_group(0, bind_group, &[]);
            self.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }
    }
}
