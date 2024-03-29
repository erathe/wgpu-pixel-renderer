use wgpu::{include_wgsl, util::DeviceExt};

use super::{pipeline_utils::create_render_pipeline, Camera, Texture};

pub struct LightNode {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pub output_texture: Texture,
}

impl LightNode {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sampler: &wgpu::Sampler,
        sprite_texture: &Texture,
        sdf_texture: &Texture,
    ) -> Self {
        let output_texture = Texture::create_2d_texture(
            device,
            config.width,
            config.height,
            config.format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            Some("light texture"),
        );
        let bind_group_layout = Self::get_bind_group_layout(device, Some("Light bg layout"));
        let bind_group = Self::get_bind_group(
            device,
            sampler,
            &bind_group_layout,
            &sprite_texture,
            &sdf_texture,
            Some("Light bg"),
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite renderer pipeline layout"),
            bind_group_layouts: &[
                &bind_group_layout,
                &Camera::create_bind_group_layout(device),
            ],
            push_constant_ranges: &[],
        });

        let shader = include_wgsl!("light.wgsl");

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
            bind_group,
            vertex_buffer,
            index_buffer,
            output_texture,
        }
    }

    fn get_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    fn get_bind_group(
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        layout: &wgpu::BindGroupLayout,
        sprite_texture: &Texture,
        sdf_texture: &Texture,
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sprite_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&sdf_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
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
pub(super) trait DrawIlluminatedScene<'a> {
    fn draw_illuminated_scene(&mut self, light_node: &'a LightNode, camera: &'a Camera);
}

impl<'a, 'b> DrawIlluminatedScene<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_illuminated_scene(&mut self, light_node: &'b LightNode, camera: &'b Camera) {
        self.set_pipeline(&light_node.pipeline);
        self.set_vertex_buffer(0, light_node.vertex_buffer.slice(..));
        self.set_index_buffer(light_node.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.set_bind_group(0, &light_node.bind_group, &[]);
        self.set_bind_group(1, &camera.bind_group(), &[]);
        self.draw_indexed(0..INDICES.len() as u32, 0, 0..1)
    }
}
