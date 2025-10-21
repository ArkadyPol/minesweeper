use bevy::{ecs::relationship::DescendantIter, prelude::*};

use crate::traits::{DescendantsTraversal, HasDescendants};

#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct TileTriggerEvent(pub Entity);
#[derive(Debug, Copy, Clone, Event)]
pub struct BoardCompletedEvent;
#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct BombExplosionEvent(pub Entity);
#[derive(Debug, Copy, Clone, EntityEvent)]
pub struct TileMarkEvent {
    pub entity: Entity,
    pub mark: bool,
}

#[derive(Debug, Clone, EntityEvent)]
#[entity_event(propagate = DescendantsTraversal, auto_propagate)]
pub struct PropagateUncoverEvent {
    pub entity: Entity,
    original: Entity,
    descendants: Vec<Entity>,
}

impl HasDescendants for PropagateUncoverEvent {
    fn entity(&self) -> Entity {
        self.entity
    }
    fn original(&self) -> Entity {
        self.original
    }
    fn descendants(&self) -> &[Entity] {
        &self.descendants
    }
}

impl PropagateUncoverEvent {
    pub fn new(entity: Entity, children_query: &Query<&Children>) -> Self {
        let descendants = DescendantIter::new(children_query, entity).collect();
        Self {
            entity,
            original: entity,
            descendants,
        }
    }
}

#[derive(Debug, Clone, Event)]
pub struct GameEndEvent {
    pub message: String,
}

#[derive(Debug, Clone, EntityEvent)]
#[entity_event(propagate = DescendantsTraversal, auto_propagate)]
pub struct CountdownEvent {
    pub entity: Entity,
    pub remaining: u8,
    original: Entity,
    descendants: Vec<Entity>,
}

impl CountdownEvent {
    pub fn new(entity: Entity, remaining: u8, children_query: &Query<&Children>) -> Self {
        let descendants = DescendantIter::new(children_query, entity).collect();
        Self {
            entity,
            remaining,
            original: entity,
            descendants,
        }
    }
}

impl HasDescendants for CountdownEvent {
    fn entity(&self) -> Entity {
        self.entity
    }
    fn original(&self) -> Entity {
        self.original
    }
    fn descendants(&self) -> &[Entity] {
        &self.descendants
    }
}

#[derive(Debug, Copy, Clone, Message)]
pub struct RestartGameEvent;
