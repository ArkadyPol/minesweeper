use bevy::prelude::*;

#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct TileTriggerEvent(pub Entity);
#[derive(Debug, Copy, Clone, Event)]
pub struct BoardCompletedEvent;
#[derive(Debug, Copy, Clone, Event)]
pub struct BombExplosionEvent;
#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct TileMarkEvent {
    pub entity: Entity,
    pub mark: bool,
}

#[derive(Debug, Copy, Clone, EntityEvent)]

pub struct PropagateUncoverEvent(pub Entity);
