use bevy::{
    ecs::{query::QueryData, traversal::Traversal},
    prelude::*,
};

pub trait HasDescendants {
    fn entity(&self) -> Entity;
    fn original(&self) -> Entity;
    fn descendants(&self) -> &[Entity];
}

#[derive(QueryData)]
pub struct DescendantsTraversal;

impl<T: HasDescendants> Traversal<T> for DescendantsTraversal {
    fn traverse(_item: Self::Item<'_, '_>, event: &T) -> Option<Entity> {
        let descendants = event.descendants();
        let current = event.entity();
        let original = event.original();

        if current == original {
            descendants.get(0).copied()
        } else {
            let pos = descendants.iter().position(|&e| e == current)?;
            descendants.get(pos + 1).copied()
        }
    }
}
