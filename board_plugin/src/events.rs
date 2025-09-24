use bevy::prelude::Message;

use crate::components::Coordinates;

#[derive(Debug, Copy, Clone, Message)]
pub struct TileTriggerEvent(pub Coordinates);
#[derive(Debug, Copy, Clone, Message)]
pub struct BoardCompletedEvent;
#[derive(Debug, Copy, Clone, Message)]
pub struct BombExplosionEvent;
#[derive(Debug, Copy, Clone, Message)]
pub struct TileMarkEvent(pub Coordinates);
