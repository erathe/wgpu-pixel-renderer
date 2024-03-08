use crate::constants::Types;

pub struct Entity {
    pub id: usize,
    pub kind: Types,
}

impl Entity {
    pub fn new(id: usize, kind: Types) -> Self {
        Self { id, kind }
    }
}
