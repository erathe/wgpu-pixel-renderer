mod animation;
mod constants;
mod entity;
mod renderer;
mod utils;
mod world;
mod world_state;

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use world::World;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(2048, 1600))
        .build(&event_loop)
        .unwrap();

    let Ok(mut world) = World::new(window).await else {
        panic!("could not initialize world")
    };

    world.initialize_map();
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
