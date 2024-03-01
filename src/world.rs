use instant::{Duration, Instant};
use rand::{thread_rng, Rng};
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::{
    camera::{self, Camera},
    pipeline_utils::{
        create_basic_sampler_bind_group, create_basic_sampler_bind_group_layout,
        create_render_pipeline,
    },
    renderer::Renderer,
    texture_atlas::TextureAtlas,
    world_state::Resources,
};

pub struct World<'world> {
    window: winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,
    pub renderer: Renderer,
    sprite_renderer: SpriteRenderer,
    resources: Resources<'world>,
    camera: Camera,
    time: Instant,
    time_since_last_frame: Duration,
    frames: i32,
    acc_time: Duration,
}

impl<'world> World<'world> {
    pub async fn new(window: Window) -> anyhow::Result<Self> {
        let time = Instant::now();
        let time_since_last_frame = Duration::from_millis(0);
        let acc_time = Duration::from_millis(0);
        let size = window.inner_size();
        let renderer = Renderer::new(&window).await?;
        let sprite_renderer =
            SpriteRenderer::new(&renderer.device, &renderer.config, &renderer.queue).await?;
        let camera = Camera::new(&renderer.device, size.height as f32, size.width as f32, 1.0);
        let resources = Resources::new();
        Ok(Self {
            renderer,
            sprite_renderer,
            size,
            resources,
            window,
            camera,
            frames: 0,
            time,
            time_since_last_frame,
            acc_time,
        })
    }

    pub fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
        if let Some(size) = new_size {
            self.size = size;
        }
        self.renderer.resize(self.size)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(&self.sprite_renderer, &self.camera)?;
        Ok(())
    }
    pub fn update(&mut self) {
        let c_time = Instant::now();
        self.time_since_last_frame = c_time - self.time;
        self.time = c_time;

        self.frames += 1;
        self.acc_time += self.time_since_last_frame;
        if self.acc_time >= Duration::from_millis(1000) {
            println!("entities: {}", self.sprite_renderer.instances.len());
            println!("FPS: {}", self.frames);
            self.acc_time = Duration::from_millis(0);
            self.frames = 0;
        }

        // if let Some(uniform_data) = self
        //     .sprite
        //     .animation
        //     .increment_frame(self.time_since_last_frame)
        // {
        //     update_uniform(&self.queue, uniform_data.buffer(), uniform_data.uniform);
        // };

        // let delta_t = self.time_since_last_frame.as_millis() as f32 / 10.;

        // if self.input.left {
        //     self.instances[15].p[0] -= 3.0 * delta_t;
        // }
        // if self.input.right {
        //     self.instances[15].p[0] += 3.0 * delta_t;
        // }
        // if self.input.up {
        //     self.instances[15].p[1] += 3.0 * delta_t;
        // }
        // if self.input.down {
        //     self.instances[15].p[1] -= 3.0 * delta_t;
        // }

        // if self.input.left | self.input.right | self.input.up | self.input.down {
        //     self.queue.write_buffer(
        //         &self.instance_buffer,
        //         0,
        //         bytemuck::cast_slice(&self.instances),
        //     );
        // }
        let mut new_instances = Vec::new();
        for _ in 0..100 {
            let mut rng = thread_rng();
            let x: f32 = rng.gen_range(-400.0..=400.);
            let y: f32 = rng.gen_range(-300.0..=300.);
            new_instances.push(SpriteInstance {
                size: [16.0, 16.0],
                texture_origin: [32.0, 0.0],
                translation: [x, y],
            });
        }
        self.sprite_renderer
            .add_sprites(new_instances, &self.renderer.queue);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // WindowEvent::TouchpadMagnify { delta, .. } => {
            //     self.camera.update_view_projection(
            //         self.config.height as f32,
            //         self.config.width as f32,
            //         *delta as f32 * 5.,
            //     );
            //     self.queue.write_buffer(
            //         &self.camera.uniform.buffer,
            //         0,
            //         bytemuck::cast_slice(&[self.camera.get_uniform().uniform]),
            //     );
            //     true
            // }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => {
                match *state {
                    // ElementState::Pressed => match key {
                    //     VirtualKeyCode::A => self.input.left = true,
                    //     VirtualKeyCode::D => self.input.right = true,
                    //     VirtualKeyCode::W => self.input.up = true,
                    //     VirtualKeyCode::S => self.input.down = true,
                    //     _ => {}
                    // },
                    ElementState::Released => match key {
                        // VirtualKeyCode::D => {
                        //     let mut rng = thread_rng();
                        //     let x: f32 = rng.gen_range(-300.0..=300.);
                        //     let y: f32 = rng.gen_range(-300.0..=300.);
                        //     self.sprite_renderer.add_sprite(
                        //         SpriteInstance {
                        //             size: [16.0, 16.0],
                        //             texture_origin: [32.0, 0.0],
                        //             translation: [x, y],
                        //         },
                        //         &self.renderer.device,
                        //     )
                        // }
                        // VirtualKeyCode::A => self.input.left = false,
                        // VirtualKeyCode::W => self.input.up = false,
                        // VirtualKeyCode::S => self.input.down = false,
                        _ => {}
                    },
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }
}

pub struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instances: Vec<SpriteInstance>,
    instance_buffer: wgpu::Buffer,
    sampler_bind_group: wgpu::BindGroup,
    texture_atlas_bind_group: wgpu::BindGroup,
}

impl SpriteRenderer {
    pub async fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<Self> {
        // Texture atlas
        let texture_atlas = TextureAtlas::new("character.png", &device, &queue).await?;

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

        let sampler_bind_group_layout =
            &create_basic_sampler_bind_group_layout(device, Some("Sprite basic sampler bg layout"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite renderer pipeline layout"),
            bind_group_layouts: &[
                &sampler_bind_group_layout,
                &texture_atlas_bind_group_layout,
                &Camera::create_bind_group_layout(device),
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

        let instances = Vec::with_capacity(8_000_000);
        // let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Instance Buffer"),
        //     contents: bytemuck::cast_slice(&instances),
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        // });

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

        // Bind groups
        let sampler_bind_group = create_basic_sampler_bind_group(
            device,
            &sampler_bind_group_layout,
            &texture_atlas.texture,
            Some("sprite renderer sampler bind group"),
        );

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
            instances,
            instance_buffer,
            sampler_bind_group,
            texture_atlas_bind_group,
        })
    }

    pub fn add_sprites(&mut self, sprites: Vec<SpriteInstance>, queue: &wgpu::Queue) {
        let offset = std::mem::size_of::<SpriteInstance>() as u64 * self.instances.len() as u64;

        queue.write_buffer(
            &self.instance_buffer,
            offset,
            bytemuck::cast_slice(&sprites),
        );
        self.instances.extend(sprites);
        // self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Instance Buffer"),
        //     contents: bytemuck::cast_slice(&self.instances),
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        // });
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureAtlasUniform {
    size: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    size: [f32; 2],
    texture_origin: [f32; 2],
    translation: [f32; 2],
}

impl SpriteInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32x2, 4 => Float32x2];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn new(size: [f32; 2], texture_origin: [f32; 2], translation: [f32; 2]) -> Self {
        Self {
            size,
            texture_origin,
            translation,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Position {
    x: f32,
    y: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Size {
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Translation {
    positon: Position,
}

struct Sprite {}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
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

// const VERTICES: &[Vertex] = &[
//     Vertex {
//         position: [-32., 32.],
//         tex_coords: [0.0, 0.0],
//     }, // Top-left
//     Vertex {
//         position: [32., 32.],
//         tex_coords: [1.0, 0.0],
//     }, // Top-right
//     Vertex {
//         position: [-32., -32.],
//         tex_coords: [0.0, 1.0],
//     }, // Bottom-left
//     Vertex {
//         position: [32., -32.],
//         tex_coords: [1.0, 1.0],
//     }, // Bottom-right
// ];

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-16., 16.],
        tex_coords: [0.0, 0.0],
    }, // Top-left
    Vertex {
        position: [16., 16.],
        tex_coords: [1.0, 0.0],
    }, // Top-right
    Vertex {
        position: [-16., -16.],
        tex_coords: [0.0, 1.0],
    }, // Bottom-left
    Vertex {
        position: [16., -16.],
        tex_coords: [1.0, 1.0],
    }, // Bottom-right
];
const INDICES: &[u16] = &[2, 1, 0u16, 2, 3, 1];

pub trait DrawSprite<'a> {
    fn draw_sprites_instanced(&mut self, sprite_renderer: &'a SpriteRenderer, camera: &'a Camera);
}

impl<'a, 'b> DrawSprite<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_sprites_instanced(&mut self, sprite_renderer: &'b SpriteRenderer, camera: &'b Camera) {
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
        self.draw_indexed(
            0..INDICES.len() as u32,
            0,
            0..sprite_renderer.instances.len() as _,
        )
    }
}
