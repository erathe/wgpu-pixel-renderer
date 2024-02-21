use crate::{resources::load_texture, texture};

pub struct TextureAtlas {
    pub texture: texture::Texture,
    width: u32,
    height: u32,
}

impl TextureAtlas {
    pub async fn new(
        path: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<Self> {
        let texture = load_texture(path, false, device, queue).await?;
        Ok(Self {
            width: texture.size.width,
            height: texture.size.height,
            texture,
        })
    }
}
