use wgpu::{include_wgsl, util::DeviceExt};

use super::{
    camera::Camera, pipeline_utils::create_render_pipeline, texture_atlas::TextureAtlas, Texture,
};

pub struct SpriteNode {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    sampler_bind_group: wgpu::BindGroup,
    texture_atlas_bind_group: wgpu::BindGroup,
    lights_bind_group: wgpu::BindGroup,
    lights_buffer: wgpu::Buffer,
    pub texture: Texture,
}

// TODO: clean up the whole lights buffer stuff, pretty hacky right now
impl SpriteNode {
    pub async fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        sampler: &wgpu::Sampler,
        sdf_texture: &Texture,
    ) -> anyhow::Result<Self> {
        // Texture atlas
        let texture_atlas = TextureAtlas::new("test_texture-sheet.png", device, &queue).await?;
        let texture = Texture::create_2d_texture(
            device,
            config.width,
            config.height,
            config.format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            Some("sprite texture"),
        );

        // Layouts
        let texture_atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture atlas bind group"),
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

        let lights_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("lights bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let sampler_bind_group_layout =
            Self::get_bind_group_layout(device, Some("sprite sampler bg layout"));
        // let sampler_bind_group_layout =
        //     &create_basic_sampler_bind_group_layout(device, Some("Sprite basic sampler bg layout"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite renderer pipeline layout"),
            bind_group_layouts: &[
                &sampler_bind_group_layout,
                &texture_atlas_bind_group_layout,
                &Camera::create_bind_group_layout(device),
                &lights_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // Shader
        let shader = include_wgsl!("texture_atlas_shader.wgsl");

        // Pipeline
        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            config.format,
            &[Vertex::desc(), SpriteInstance::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
            Some("sprite renderer pipeline"),
        );

        // Buffers
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

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: 1_200_000 * std::mem::size_of::<SpriteInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let texture_atlas_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Buffer"),
            contents: bytemuck::cast_slice(&[TextureAtlasUniform {
                size: [texture_atlas.width as f32, texture_atlas.height as f32],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lights Buffer"),
            size: 100 * std::mem::size_of::<Light>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        // Bind groups
        // let sampler_bind_group = create_basic_sampler_bind_group(
        //     &device,
        //     &sampler,
        //     &sampler_bind_group_layout,
        //     &texture_atlas.texture,
        //     Some("sprite renderer sampler bind group"),
        // );

        let sampler_bind_group = Self::get_bind_group(
            &device,
            &sampler,
            &sampler_bind_group_layout,
            &texture_atlas.texture,
            &sdf_texture,
            Some("sprite bg"),
        );
        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lights bind group"),
            layout: &lights_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_buffer.as_entire_binding(),
            }],
        });
        let texture_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite renderer texture atlas Bind Group"),
            layout: &texture_atlas_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: texture_atlas_buffer.as_entire_binding(),
            }],
        });

        Ok(Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            sampler_bind_group,
            texture_atlas_bind_group,
            lights_bind_group,
            lights_buffer,
            texture,
        })
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
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
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
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&sdf_texture.view),
                },
            ],
        })
    }

    pub fn draw_sprites(
        &mut self,
        sprites: &[SpriteInstance],
        queue: &wgpu::Queue,
        // offset_base: u64,
    ) {
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&sprites));
        // self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Instance Buffer"),
        //     contents: bytemuck::cast_slice(&sprites),
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        // });
    }

    // TODO: This should be generalized
    pub fn draw_lights(&mut self, lights: &[Light], queue: &wgpu::Queue) {
        queue.write_buffer(&self.lights_buffer, 0, bytemuck::cast_slice(&lights));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    pub position: [f32; 2],
    pub intensity: f32,
    pub falloff: f32,
    pub color: [f32; 3],
    pub frequency: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureAtlasUniform {
    size: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Wrapped2D([f32; 2]);

impl Wrapped2D {
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    pub fn width(&self) -> f32 {
        self.0[0]
    }

    pub fn y(&self) -> f32 {
        self.0[1]
    }

    pub fn height(&self) -> f32 {
        self.0[1]
    }

    pub fn set_delta_x(&mut self, delta: f32) {
        self.0[0] += delta;
    }
    pub fn set_delta_y(&mut self, delta: f32) {
        self.0[1] += delta;
    }

    fn new(init: [f32; 2]) -> Self {
        Self(init)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    pub size: Wrapped2D,
    pub texture_origin: Wrapped2D,
    pub translation: Wrapped2D,
}

impl SpriteInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32x2, 4 => Float32x2];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn new(size: [f32; 2], texture_origin: [f32; 2], translation: [f32; 2]) -> Self {
        Self {
            size: Wrapped2D::new(size),
            texture_origin: Wrapped2D::new(texture_origin),
            translation: Wrapped2D::new(translation),
        }
    }

    // fn raw(sprite: &Sprite) -> Self {
    //     SpriteInstance {
    //         size: [sprite.size.width, sprite.size.height],
    //         texture_origin: [sprite.texture_origin.x, sprite.texture_origin.y],
    //         translation: [sprite.translation.position.x, sprite.translation.position.y],
    //     }
    // }
}

//TODO: Move this to somewhere general
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
        position: [-0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // Top-left
    Vertex {
        position: [0.5, 0.5],
        tex_coords: [1.0, 0.0],
    }, // Top-right
    Vertex {
        position: [-0.5, -0.5],
        tex_coords: [0.0, 1.0],
    }, // Bottom-left
    Vertex {
        position: [0.5, -0.5],
        tex_coords: [1.0, 1.0],
    }, // Bottom-right
];

const INDICES: &[u16] = &[2, 1, 0u16, 2, 3, 1];

pub(super) trait DrawSprite<'a> {
    fn draw_sprites_instanced(
        &mut self,
        sprite_renderer: &'a SpriteNode,
        camera: &'a Camera,
        instances: u32,
    );
}

impl<'a, 'b> DrawSprite<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_sprites_instanced(
        &mut self,
        sprite_renderer: &'b SpriteNode,
        camera: &'b Camera,
        instances: u32,
    ) {
        self.set_pipeline(&sprite_renderer.pipeline);
        self.set_vertex_buffer(0, sprite_renderer.vertex_buffer.slice(..));
        self.set_vertex_buffer(1, sprite_renderer.instance_buffer.slice(..));
        self.set_index_buffer(
            sprite_renderer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        self.set_bind_group(0, &sprite_renderer.sampler_bind_group, &[]);
        self.set_bind_group(1, &sprite_renderer.texture_atlas_bind_group, &[]);
        self.set_bind_group(2, &camera.bind_group(), &[]);
        self.set_bind_group(3, &sprite_renderer.lights_bind_group, &[]);
        self.draw_indexed(0..INDICES.len() as u32, 0, 0..instances)
    }
}
