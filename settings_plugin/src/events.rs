use bevy::prelude::*;

use crate::input_value::InputValue;

#[derive(Debug, Copy, Clone, Message)]
pub struct CreateGameEvent;
#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct LostFocusEvent(pub Entity);
#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct SetCursorPosEvent {
    pub entity: Entity,
    pub cursor_pos: usize,
}
#[derive(Debug, Clone, EntityEvent)]
#[entity_event(auto_propagate)]
pub struct ChangeInput {
    pub entity: Entity,
    pub value: InputValue,
    pub label: Option<String>,
}

#[derive(Debug, Clone, EntityEvent)]
pub struct BackOriginalInput {
    pub entity: Entity,
    pub value: InputValue,
}
