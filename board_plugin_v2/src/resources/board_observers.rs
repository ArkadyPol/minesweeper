use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct BoardObservers {
    pub input_observer: Entity,
    pub tile_trigger_observer: Entity,
}
