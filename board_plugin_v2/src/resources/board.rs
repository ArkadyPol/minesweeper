use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct Board {
    pub tile_size: f32,
    pub entity: Entity,
    pub tile_mark_observer: Entity,
}
