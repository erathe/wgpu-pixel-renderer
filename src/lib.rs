mod animation;
mod camera;
mod pipeline_utils;
mod renderer;
mod resources;
mod sprite;
mod texture;
mod texture_atlas;
mod utils;
mod world;
mod world_state;

use std::rc::Rc;

use bytemuck::NoUninit;
use camera::Camera;
use instant::{Duration, Instant};
use renderer::Renderer;
use sprite::{Sprite, SpriteInstance};
use texture_atlas::TextureAtlas;
use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use world::World;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let Ok(mut world) = World::new(window).await else {
        panic!("could not initialize world")
    };

    // let Ok(mut state) = State::new().await else {
    //     panic!("could not initialize state")
    // };

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } if window_id == world.window().id() => {
            if !world.input(&event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        world.resize(Some(physical_size));
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        world.resize(Some(*new_inner_size))
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == world.window().id() => {
            world.update();
            match world.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => world.resize(None),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            world.window().request_redraw();
        }
        _ => {}
    })
}

struct Input {
    up: bool,
    left: bool,
    right: bool,
    down: bool,
}

struct State {
    frames: i32,
    acc_time: Duration,
    camera: Camera,
    sprite: Sprite,
    instances: Vec<SpriteInstance>,
    instance_buffer: wgpu::Buffer,
    time: Instant,
    time_since_last_frame: Duration,
    input: Input,
}

impl State {
    // async fn new() -> anyhow::Result<Self> {
    //     let time = Instant::now();
    //     let time_since_last_frame = Duration::from_millis(0);

    //     let camera = Camera::new(&device, config.height as f32, config.width as f32, 1.);

    //     let instances = (0..3)
    //         .flat_map(|y| {
    //             (0..10).map(move |x| {
    //                 SpriteInstance::new([-400. + (x as f32 * 64.), -100. + (y as f32 * 64.)])
    //             })
    //         })
    //         .collect::<Vec<_>>();
    //     let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: Some("Instance Buffer"),
    //         contents: bytemuck::cast_slice(&instances),
    //         usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    //     });
    //     let Ok(sprite) = Sprite::new(&device, &texture_atlas, &config, &camera, [4, 4], 0).await
    //     else {
    //         panic!("nooo");
    //     };

    //     let input = Input {
    //         up: false,
    //         down: false,
    //         left: false,
    //         right: false,
    //     };

    //     Ok(Self {
    //         frames: 0,
    //         acc_time: Duration::from_millis(0),
    //         camera,
    //         sprite,
    //         instances,
    //         instance_buffer,
    //         input,
    //         time,
    //         time_since_last_frame,
    //     })
    // }

    // pub fn window(&self) -> &Window {
    //     &self.window
    // }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // if new_size.width > 0 && new_size.height > 0 {
        //     self.size = new_size;
        //     self.config.width = new_size.width;
        //     self.config.height = new_size.height;
        //     self.surface.configure(&self.device, &self.config);
        // }
        // self.camera
        //     .update_view_projection(self.config.height as f32, self.config.width as f32, 0.);
        // self.queue.write_buffer(
        //     &self.camera.uniform.buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.camera.get_uniform().uniform]),
        // );
    }

    // fn input(&mut self, event: &WindowEvent) -> bool {
    //     match event {
    //         // WindowEvent::TouchpadMagnify { delta, .. } => {
    //         //     self.camera.update_view_projection(
    //         //         self.config.height as f32,
    //         //         self.config.width as f32,
    //         //         *delta as f32 * 5.,
    //         //     );
    //         //     self.queue.write_buffer(
    //         //         &self.camera.uniform.buffer,
    //         //         0,
    //         //         bytemuck::cast_slice(&[self.camera.get_uniform().uniform]),
    //         //     );
    //         //     true
    //         // }
    //         WindowEvent::KeyboardInput {
    //             input:
    //                 KeyboardInput {
    //                     state,
    //                     virtual_keycode: Some(key),
    //                     ..
    //                 },
    //             ..
    //         } => {
    //             match *state {
    //                 ElementState::Pressed => match key {
    //                     VirtualKeyCode::A => self.input.left = true,
    //                     VirtualKeyCode::D => self.input.right = true,
    //                     VirtualKeyCode::W => self.input.up = true,
    //                     VirtualKeyCode::S => self.input.down = true,
    //                     _ => {}
    //                 },
    //                 ElementState::Released => match key {
    //                     VirtualKeyCode::D => self.input.right = false,
    //                     VirtualKeyCode::A => self.input.left = false,
    //                     VirtualKeyCode::W => self.input.up = false,
    //                     VirtualKeyCode::S => self.input.down = false,
    //                     _ => {}
    //                 },
    //             }
    //             true
    //         }
    //         _ => false,
    //     }
    // }

    fn update(&mut self) {
        // let c_time = Instant::now();
        // self.time_since_last_frame = c_time - self.time;
        // self.time = c_time;

        // self.frames += 1;
        // self.acc_time += self.time_since_last_frame;
        // if self.acc_time >= Duration::from_millis(1000) {
        //     println!("FPS: {}", self.frames);
        //     self.acc_time = Duration::from_millis(0);
        //     self.frames = 0;
        // }

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
    }

    // fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    //     let output = self.surface.get_current_texture()?;
    //     let view = output
    //         .texture
    //         .create_view(&wgpu::TextureViewDescriptor::default());

    //     self.sprite.draw_instanced(
    //         &self.device,
    //         &self.queue,
    //         &view,
    //         &self.camera,
    //         self.instances.len(),
    //         &self.instance_buffer,
    //     );

    //     output.present();

    //     Ok(())
    // }
}

pub fn update_uniform<A: NoUninit>(queue: &wgpu::Queue, buffer: &wgpu::Buffer, data: A) {
    queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
}

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

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-32., 32.],
        tex_coords: [0.0, 0.0],
    }, // Top-left
    Vertex {
        position: [32., 32.],
        tex_coords: [1.0, 0.0],
    }, // Top-right
    Vertex {
        position: [-32., -32.],
        tex_coords: [0.0, 1.0],
    }, // Bottom-left
    Vertex {
        position: [32., -32.],
        tex_coords: [1.0, 1.0],
    }, // Bottom-right
];

pub const INDICES: &[u16] = &[2, 1, 0u16, 2, 3, 1];
