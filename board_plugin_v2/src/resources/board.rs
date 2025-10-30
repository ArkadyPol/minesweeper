#[cfg(not(any(feature = "simple_neighbors", feature = "hierarchical_neighbors")))]
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

#[cfg(not(any(feature = "simple_neighbors", feature = "hierarchical_neighbors")))]
use crate::components::Coordinates;

#[derive(Debug, Resource)]
pub struct Board {
    pub tile_size: f32,
    pub entity: Entity,
    pub observers: Vec<Entity>,
    pub timer: Option<Timer>,
    pub end_message: String,
    #[cfg(not(any(feature = "simple_neighbors", feature = "hierarchical_neighbors")))]
    pub coords_map: HashMap<Coordinates, Entity>,
}
