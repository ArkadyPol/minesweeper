use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct Board {
    pub tile_size: f32,
    pub entity: Entity,
    pub observers: Vec<Entity>,
    pub timer: Option<Timer>,
    pub end_message: String,
}
