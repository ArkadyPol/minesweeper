use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Message)]
pub struct CreateGameEvent;
#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct LostFocusEvent(pub Entity);
#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct SetCursorPosEvent {
    pub entity: Entity,
    pub cursor_pos: usize,
}
