use bytemuck::NoUninit;
use winit::window::Window;

use super::{
    camera::Camera,
    debug_renderer::DebugTexture,
    output_renderer::DrawToScreen,
    sprite_renderer::{DrawSprite, SpriteRenderer},
    utils::to_linear_rgb,
    DebugRenderer, OutputRenderer,
};

pub struct Renderer {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub sampler: wgpu::Sampler,
    // sprite node
    // sdf node
    // lighting node
    // output node
}

impl Renderer {
    pub async fn new(window: &Window) -> anyhow::Result<Self> {
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

        Ok(Self {
            surface,
            device,
            queue,
            config,
            sampler,
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

    pub fn render() {
        todo!();
    }

    pub fn render_sprites_to_texture(
        &mut self,
        sprite_renderer: &SpriteRenderer,
        camera: &Camera,
        instances: u32,
    ) -> Result<(), wgpu::SurfaceError> {
        let view = &sprite_renderer.texture.view;

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

        pass.draw_sprites_instanced(sprite_renderer, camera, instances);
        drop(pass);

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn render_to_screen(
        &mut self,
        output_renderer: &OutputRenderer,
        debug_renderer: &DebugRenderer,
        show_debug_texture: bool,
    ) -> Result<(), wgpu::SurfaceError> {
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

        pass.draw_to_screen(output_renderer);
        if show_debug_texture {
            pass.draw_debug_texture(debug_renderer);
        }
        drop(pass);

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}
