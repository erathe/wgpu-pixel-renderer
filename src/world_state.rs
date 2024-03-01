use std::{collections::HashMap, rc::Rc, time::Instant};

use instant::Duration;

use crate::texture_atlas::TextureAtlas;

pub struct WorldState {
    frames: i32,
    acc_time: Duration,
    time: Instant,
    time_since_last_frame: Duration,
    texture_atlas: Rc<TextureAtlas>,
}

pub struct Resources<'world> {
    texture_atlases: HashMap<&'world str, Rc<TextureAtlas>>,
}

impl<'world> Resources<'world> {
    pub fn new() -> Self {
        Self {
            texture_atlases: HashMap::new(),
        }
    }
}
