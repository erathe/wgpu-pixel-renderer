use instant::{Duration, Instant};
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::{
    constants::{
        parse_map, Position, Size, Translation, Types, MAP, SPRITE_SIZE, TILES, TILE_SIZE,
    },
    renderer::{Camera, Renderer, SpriteRenderer},
    sprite::Sprite,
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
    pub renderer: Renderer,
    sprite_renderer: SpriteRenderer,
    camera: Camera,
    time: Instant,
    time_since_last_frame: Duration,
    frames: i32,
    acc_time: Duration,
    pub entities: Vec<Sprite>,
    input: Input,
}

impl World {
    pub async fn new(window: Window) -> anyhow::Result<Self> {
        let time = Instant::now();
        let time_since_last_frame = Duration::from_millis(0);
        let acc_time = Duration::from_millis(0);
        let size = window.inner_size();
        let renderer = Renderer::new(&window).await?;
        let entities = Vec::new();
        let sprite_renderer =
            SpriteRenderer::new(&renderer.device, &renderer.config, &renderer.queue).await?;
        let camera = Camera::new(
            &renderer.device,
            size.height as f32,
            size.width as f32,
            1.0,
            (428., 100.),
        );
        Ok(Self {
            renderer,
            sprite_renderer,
            size,
            window,
            camera,
            frames: 0,
            time,
            time_since_last_frame,
            acc_time,
            entities,
            input: Input {
                up: false,
                left: false,
                right: false,
                down: false,
            },
        })
    }

    pub fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
        if let Some(size) = new_size {
            self.size = size;
        }
        self.renderer.resize(self.size);

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
            &self.sprite_renderer,
            &self.camera,
            self.entities.len() as u32,
        )?;
        Ok(())
    }
    pub fn update(&mut self) {
        let c_time = Instant::now();
        self.time_since_last_frame = c_time - self.time;
        self.time = c_time;

        self.frames += 1;
        self.acc_time += self.time_since_last_frame;
        if self.acc_time >= Duration::from_millis(1000) {
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

        self.sprite_renderer.draw_sprites(
            &self.entities,
            &self.renderer.device,
            self.entities.len() as u64,
        );
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::TouchpadMagnify { delta, .. } => {
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
                        _ => {}
                    },
                    ElementState::Released => match key {
                        VirtualKeyCode::D => self.input.right = false,
                        VirtualKeyCode::A => self.input.left = false,
                        VirtualKeyCode::W => self.input.up = false,
                        VirtualKeyCode::S => self.input.down = false,
                        _ => {}
                    },
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }

    fn spawn_sprite(&mut self, texture_origin: Position, translation: Translation, kind: Types) {
        let idx = self.entities.len();
        let sprite = Sprite::new(
            idx,
            kind,
            Size {
                width: SPRITE_SIZE,
                height: SPRITE_SIZE,
            },
            texture_origin,
            translation,
        );

        self.entities.push(sprite);
    }

    pub(crate) fn initialize(&mut self) {
        let map = parse_map(MAP);
        for ((x, y), tile) in map {
            self.spawn_sprite(
                tile,
                Translation {
                    position: Position {
                        x: x as f32 * TILE_SIZE,
                        y: y as f32 * TILE_SIZE,
                    },
                },
                Types::ENVIRONMENT,
            )
        }
        self.spawn_sprite(
            TILES.player_walk_down_1,
            Translation {
                position: Position {
                    x: 20. as f32 * TILE_SIZE,
                    y: 20. as f32 * TILE_SIZE,
                },
            },
            Types::PLAYER,
        );
    }

    fn move_player(&mut self) -> Option<(f32, f32)> {
        let delta_t = self.time_since_last_frame.as_millis() as f32 / 10.;
        if let Some(player) = self.entities.iter_mut().find(|e| e.kind == Types::PLAYER) {
            if self.input.left {
                player.translation.position.x -= 3.0 * delta_t;
            }
            if self.input.right {
                player.translation.position.x += 3.0 * delta_t;
            }
            if self.input.up {
                player.translation.position.y += 3.0 * delta_t;
            }
            if self.input.down {
                player.translation.position.y -= 3.0 * delta_t;
            }

            return Some((player.translation.position.x, player.translation.position.y));
        }

        None
    }
}
