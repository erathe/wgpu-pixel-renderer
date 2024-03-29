use bytemuck::NoUninit;
use winit::window::Window;

use super::{
    camera::Camera,
    debug_node::DebugTexture,
    light_node::{DrawIlluminatedScene, LightNode},
    output_node::DrawToScreen,
    sprite_node::DrawSprite,
    utils::to_linear_rgb,
    DebugNode, OutputNode, SDFPipeline, SpriteInstance, SpriteNode, Texture,
};

pub struct Renderer {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub sampler: wgpu::Sampler,
    sprite_node: SpriteNode,
    sdf_node: SDFPipeline,
    output_node: OutputNode,
    light_node: LightNode,
    debug_node: DebugNode,
    // sprite node
    // sdf node
    // lighting node
    // output node
}

impl Renderer {
    pub async fn new(
        window: &Window,
        occluder_data: Vec<f32>,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::all_webgpu_mask()
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                        | wgpu::Features::TEXTURE_BINDING_ARRAY,
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = wgpu::TextureFormat::Rgba16Float;
        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     .copied()
        //     .filter(|f| f.is_srgb())
        //     .next()
        //     .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let texture = Texture::from_data(&device, &queue, &occluder_data, width, height);

        let sprite_node = SpriteNode::new(&device, &config, &queue, &sampler).await?;
        let sdf_node = SDFPipeline::new(&device, texture);
        let mut debug_node = DebugNode::new(&device, &config);
        debug_node.set_bind_group(&device, &sampler, &sdf_node.output_texture);
        let light_node = LightNode::new(
            &device,
            &config,
            &sampler,
            &sprite_node.texture,
            &sdf_node.output_texture,
        );
        let output_node = OutputNode::new(&device, &config, &sampler, &sprite_node.texture);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            sampler,
            sprite_node,
            sdf_node,
            light_node,
            output_node,
            debug_node,
        })
    }

    pub(super) fn write_buffer<A: NoUninit>(&mut self, buffer: &wgpu::Buffer, data: A) {
        self.queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn draw_sprites(&mut self, sprites: &[SpriteInstance]) {
        self.sprite_node.draw_sprites(sprites, &self.queue);
    }

    pub fn render(
        &mut self,
        camera: &Camera,
        instances: u32,
        show_debug_texture: bool,
    ) -> Result<(), wgpu::SurfaceError> {
        self.sdf_node.compute_pass(&self.device, &self.queue);
        self.render_sprites_to_texture(camera, instances)?;
        // self.render_illuminated_scene(camera)?;
        self.render_to_screen(show_debug_texture)?;

        Ok(())
    }

    pub fn render_sprites_to_texture(
        &mut self,
        camera: &Camera,
        instances: u32,
    ) -> Result<(), wgpu::SurfaceError> {
        let view = &self.sprite_node.texture.view;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let clear_color = to_linear_rgb(0x0F0F26);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Base::pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color[0] as f64,
                        g: clear_color[1] as f64,
                        b: clear_color[2] as f64,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.draw_sprites_instanced(&self.sprite_node, camera, instances);
        drop(pass);

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn render_illuminated_scene(&mut self, camera: &Camera) -> Result<(), wgpu::SurfaceError> {
        let view = &self.light_node.output_texture.view;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let clear_color = to_linear_rgb(0x0F0F26);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Base::pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color[0] as f64,
                        g: clear_color[1] as f64,
                        b: clear_color[2] as f64,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.draw_illuminated_scene(&self.light_node, camera);
        drop(pass);

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn render_to_screen(&mut self, show_debug_texture: bool) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let clear_color = to_linear_rgb(0x0F0F26);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Base::pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color[0] as f64,
                        g: clear_color[1] as f64,
                        b: clear_color[2] as f64,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.draw_to_screen(&self.output_node);
        if show_debug_texture {
            pass.draw_debug_texture(&self.debug_node);
        }
        drop(pass);

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}
