use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Message)]
pub struct CreateGameEvent;
#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct LostFocusEvent(pub Entity);
