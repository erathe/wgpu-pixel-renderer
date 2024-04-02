use std::collections::HashMap;

use instant::{Duration, Instant};
use rand::Rng;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::{
    constants::{parse_map, Position, Translation, Types, MAP, SPRITE_SIZE, TILES, TILE_SIZE},
    entity::Entity,
    renderer::{Camera, Light, Renderer, SpriteInstance},
    utils::Incrementor,
};

struct Input {
    up: bool,
    left: bool,
    right: bool,
    down: bool,
}
pub struct World {
    window: winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,
    id_generator: Incrementor,
    pub renderer: Renderer,
    camera: Camera,
    time: Instant,
    time_since_last_frame: Duration,
    time_tot: Duration,
    frames: i32,
    acc_time: Duration,
    sprite_instances: Vec<SpriteInstance>,
    instance_map: HashMap<usize, usize>,
    pub map: HashMap<(usize, usize), Position>,
    pub entities: Vec<Entity>,
    input: Input,
    debug_texture: bool,
    lights: Vec<Light>,
}

impl World {
    pub async fn new(window: Window) -> anyhow::Result<Self> {
        let time = Instant::now();
        let time_since_last_frame = Duration::from_millis(0);
        let time_tot = Duration::from_millis(0);
        let acc_time = Duration::from_millis(0);
        let size = window.inner_size();
        let (map, occluder_data, width, height) = parse_map(MAP);
        let renderer = Renderer::new(&window, occluder_data, width, height).await?;
        let sprite_instances =
            Vec::with_capacity(std::mem::size_of::<SpriteInstance>() * 1_000_000);
        let instance_map = HashMap::new();
        let entities = Vec::new();
        let camera = Camera::new(
            &renderer.device,
            size.height as f32,
            size.width as f32,
            1.0,
            (0., 0.),
        );
        let id_generator = Incrementor::new();

        let lights = Vec::from([
            Light {
                position: [1200., 920.],
                intensity: 3.,
                falloff: 0.4,
                color: [1., 1., 1.],
                frequency: 2.,
            },
            Light {
                position: [300., 900.],
                intensity: 3.,
                falloff: 0.2,
                color: [0.7, 0.3, 0.1],
                frequency: 7.,
            },
            Light {
                position: [150., 500.],
                intensity: 3.,
                falloff: 0.2,
                color: [0.4, 0.2, 0.8],
                frequency: 10.,
            },
            Light {
                position: [300., 200.],
                intensity: 3.,
                falloff: 0.4,
                color: [0.3, 0.2, 0.8],
                frequency: 1.,
            },
            Light {
                position: [500., 550.],
                intensity: 3.,
                falloff: 0.4,
                color: [0.98, 0.34, 0.13],
                frequency: 7.,
            },
            Light {
                position: [1000., 550.],
                intensity: 3.,
                falloff: 0.2,
                color: [1., 0.5, 0.3],
                frequency: 4.,
            },
            Light {
                position: [1400., 550.],
                intensity: 4.,
                falloff: 0.4,
                color: [0., 0.5, 0.3],
                frequency: 3.,
            },
            Light {
                position: [1800., 350.],
                intensity: 2.,
                falloff: 0.4,
                color: [0.4, 0.8, 0.1],
                frequency: 7.,
            },
            Light {
                position: [1500., 150.],
                intensity: 3.,
                falloff: 0.4,
                color: [0.7, 0.3, 0.1],
                frequency: 1.,
            },
            Light {
                position: [1800., 950.],
                intensity: 3.,
                falloff: 0.4,
                color: [0.1, 1.0, 0.5],
                frequency: 9.,
            },
        ]);
        Ok(Self {
            renderer,
            instance_map,
            id_generator,
            size,
            map,
            window,
            lights,
            camera,
            frames: 0,
            time,
            time_tot,
            time_since_last_frame,
            sprite_instances,
            acc_time,
            entities,
            input: Input {
                up: false,
                left: false,
                right: false,
                down: false,
            },
            debug_texture: false,
        })
    }

    pub fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
        if let Some(size) = new_size {
            self.size = size;
        }
        self.renderer.resize(self.size);

        // TODO: move into renderer
        self.camera
            .update_view_projection(self.size.height as f32, self.size.width as f32, 0.);
        self.renderer.queue.write_buffer(
            &self.camera.uniform.buffer,
            0,
            bytemuck::cast_slice(&[self.camera.get_uniform().uniform]),
        );
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(
            &self.camera,
            self.sprite_instances.len() as u32,
            self.debug_texture,
        )?;
        Ok(())
    }
    pub fn update(&mut self) {
        let c_time = Instant::now();
        self.time_since_last_frame = c_time - self.time;
        self.time = c_time;
        self.time_tot += self.time_since_last_frame;

        self.frames += 1;
        self.acc_time += self.time_since_last_frame;
        if self.acc_time >= Duration::from_millis(1000) {
            println!("entities: {}", self.entities.len());
            println!("FPS: {}", self.frames);
            self.acc_time = Duration::from_millis(0);
            self.frames = 0;
        }

        if let Some(position) = self.move_player() {
            // TODO: abstract this
            self.camera.move_camera(position);
            self.renderer.queue.write_buffer(
                &self.camera.uniform.buffer,
                0,
                bytemuck::cast_slice(&[self.camera.get_uniform().uniform]),
            );
        };

        self.move_lights();

        // let mut rng = rand::thread_rng();
        // let mut rng_y = rand::thread_rng();
        // for _ in 0..1 {
        //     self.spawn_sprite(
        //         TILES.player_walk_down_2,
        //         Translation {
        //             position: Position {
        //                 x: rng.gen_range(0..=1600) as f32,
        //                 y: rng_y.gen_range(0..=1200) as f32,
        //             },
        //         },
        //         Types::PLAYER,
        //     );
        // }

        //currently we just generate sdf for the whole map. This is put here
        //to support when I start generating sdfs based on what the camera sees instead
        self.renderer.draw_sprites(
            &self.sprite_instances,
            // self.entities.len() as u64,
        );

        self.renderer.draw_lights(&self.lights);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::TouchpadMagnify { delta, .. } => {
                //TODO: fix this
                self.camera.update_view_projection(
                    self.size.height as f32,
                    self.size.width as f32,
                    *delta as f32 * 5.,
                );
                self.renderer.queue.write_buffer(
                    &self.camera.uniform.buffer,
                    0,
                    bytemuck::cast_slice(&[self.camera.get_uniform().uniform]),
                );
                true
            }
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
                    ElementState::Pressed => match key {
                        VirtualKeyCode::A => self.input.left = true,
                        VirtualKeyCode::D => self.input.right = true,
                        VirtualKeyCode::W => self.input.up = true,
                        VirtualKeyCode::S => self.input.down = true,
                        VirtualKeyCode::U => self.debug_texture = true,
                        _ => {}
                    },
                    ElementState::Released => match key {
                        VirtualKeyCode::D => self.input.right = false,
                        VirtualKeyCode::A => self.input.left = false,
                        VirtualKeyCode::W => self.input.up = false,
                        VirtualKeyCode::S => self.input.down = false,
                        VirtualKeyCode::U => self.debug_texture = false,
                        _ => {}
                    },
                }
                true
            }
            _ => false,
        }
    }

    fn spawn_sprite(&mut self, texture_origin: &Position, translation: Translation, kind: Types) {
        let Some(id) = self.id_generator.next() else {
            panic!("could not generate id for entity");
        };
        let sprite = Entity::new(id, kind);
        let instance_id = self.sprite_instances.len();
        self.sprite_instances.push(SpriteInstance::new(
            [SPRITE_SIZE, SPRITE_SIZE],
            [texture_origin.x, texture_origin.y],
            [translation.position.x, translation.position.y],
        ));

        self.instance_map.insert(id, instance_id);

        self.entities.push(sprite);
    }

    pub(crate) fn initialize_map(&mut self) {
        //TODO: clean up and fix parsing
        let map = self.map.clone();
        for ((x, y), tile) in map {
            self.spawn_sprite(
                &tile,
                Translation {
                    position: Position {
                        x: (x * TILE_SIZE) as f32,
                        y: (y * TILE_SIZE) as f32,
                    },
                },
                Types::ENVIRONMENT,
            )
        }
        // TODO: This guy should also occlude
        self.spawn_sprite(
            &TILES.player_walk_down_1,
            Translation {
                position: Position {
                    x: (20 * TILE_SIZE) as f32,
                    y: (20 * TILE_SIZE) as f32,
                },
            },
            Types::PLAYER,
        );
    }

    fn move_lights(&mut self) {
        let acc_t = self.time_tot;
        self.lights.iter_mut().for_each(|l| {
            let new_x = 0.4 * (l.frequency * acc_t.as_secs_f32()).sin();
            let new_y = 0.4 * (l.frequency * acc_t.as_secs_f32()).cos();
            l.position[0] += new_x;
            l.position[1] += new_y;
            l.intensity += 0.3 * (acc_t.as_secs_f32() * l.frequency).sin();
        });
    }

    fn move_player(&mut self) -> Option<(f32, f32)> {
        let delta_t = self.time_since_last_frame.as_millis() as f32 / 10.;
        let mut v = None;
        self.entities
            .iter_mut()
            .filter(|e| e.kind == Types::PLAYER)
            .enumerate()
            .for_each(|(id, e)| {
                if let Some(instance) = self.sprite_instances.get_mut(e.id) {
                    // let mut rng = rand::thread_rng();
                    // let t = rng.gen_range(0..=3)
                    if self.input.left {
                        instance.translation.set_delta_x(-3.0 * delta_t);
                    }
                    if self.input.right {
                        instance.translation.set_delta_x(3.0 * delta_t);
                    }
                    if self.input.up {
                        instance.translation.set_delta_y(3.0 * delta_t);
                    }
                    if self.input.down {
                        instance.translation.set_delta_y(-3.0 * delta_t);
                    }

                    if id == 0 {
                        v = Some((instance.translation.x(), instance.translation.y()))
                    }
                }
            });

        return v;
    }
}
