use crate::{
    constants::{Position, Size, Translation, Types},
    renderer::{IntoSpriteInstance, SpriteInstance},
};

pub struct Sprite {
    idx: usize,
    pub kind: Types,
    pub size: Size,
    pub texture_origin: Position,
    pub translation: Translation,
}

impl Sprite {
    pub fn new(
        idx: usize,
        kind: Types,
        size: Size,
        texture_origin: Position,
        translation: Translation,
    ) -> Self {
        Self {
            idx,
            kind,
            size,
            texture_origin,
            translation,
        }
    }
}

impl IntoSpriteInstance for Sprite {
    fn into_sprite_instance(&self) -> SpriteInstance {
        SpriteInstance::new(
            [self.size.width, self.size.height],
            [self.texture_origin.x, self.texture_origin.y],
            [self.translation.position.x, self.translation.position.y],
        )
    }
}
