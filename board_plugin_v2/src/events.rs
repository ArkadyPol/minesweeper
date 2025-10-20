use bevy::{
    ecs::{query::QueryData, relationship::DescendantIter, traversal::Traversal},
    prelude::*,
};

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

#[derive(QueryData)]
pub struct ChildrenTraversal;
impl Traversal<PropagateUncoverEvent> for ChildrenTraversal {
    fn traverse(_item: Self::Item<'_, '_>, event: &PropagateUncoverEvent) -> Option<Entity> {
        if event.entity == event.original {
            event.descendants.get(0).copied()
        } else {
            let pos = event.descendants.iter().position(|&e| e == event.entity)?;
            event.descendants.get(pos + 1).copied()
        }
    }
}

#[derive(Debug, Clone, EntityEvent)]
#[entity_event(propagate = ChildrenTraversal, auto_propagate)]
pub struct PropagateUncoverEvent {
    pub entity: Entity,
    original: Entity,
    descendants: Vec<Entity>,
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
