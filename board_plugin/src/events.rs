use bevy::ecs::event::Event;

use crate::components::Coordinates;

#[derive(Debug, Copy, Clone, Event)]
pub struct TileTriggerEvent(pub Coordinates);
#[derive(Debug, Copy, Clone, Event)]
pub struct BoardCompletedEvent;
#[derive(Debug, Copy, Clone, Event)]
pub struct BombExplosionEvent;
#[derive(Debug, Copy, Clone, Event)]
pub struct TileMarkEvent(pub Coordinates);
