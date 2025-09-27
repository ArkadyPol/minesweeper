use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Message)]
pub struct TileTriggerEvent(pub Entity);
#[derive(Debug, Copy, Clone, Message)]
pub struct BoardCompletedEvent;
#[derive(Debug, Copy, Clone, Message)]
pub struct BombExplosionEvent;
#[derive(Debug, Copy, Clone, Message)]
pub struct TileMarkEvent(pub Entity, pub bool);
