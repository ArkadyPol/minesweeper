use bevy::{log, prelude::*};

use crate::{
    BoardOptions,
    components::{Bomb, BombNeighbor, Neighbors, TileCover, Uncover},
    events::{BoardCompletedEvent, BombExplosionEvent, TileTriggerEvent},
};

pub fn trigger_event_handler(event: On<TileTriggerEvent>, mut commands: Commands) {
    commands.entity(event.0).insert(Uncover);
}

pub fn uncover_tiles(
    mut commands: Commands,
    children: Query<(Entity, &ChildOf), With<Uncover>>,
    parents: Query<(&Neighbors, &Children, Option<&Bomb>, Option<&BombNeighbor>)>,
    cover_query: Query<(), (With<TileCover>, Without<Uncover>)>,
    board_options: Option<Res<BoardOptions>>,
) {
    let options = match board_options {
        None => BoardOptions::default(), // If no options is set we use the default one
        Some(o) => o.clone(),
    };
    let bomb_count = options.bomb_count as usize;

    // We iterate through tile covers to uncover
    for (entity, parent) in children.iter() {
        // we destroy the tile cover entity
        commands.entity(entity).despawn();

        let (neighbors, _, bomb, bomb_counter) = match parents.get(parent.0) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{}", e);
                continue;
            }
        };

        if cover_query.count() == bomb_count {
            log::info!("Board completed");
            commands.trigger(BoardCompletedEvent);
        }

        if bomb.is_some() {
            log::info!("Boom !");
            commands.trigger(BombExplosionEvent);
        }
        // If the tile is empty..
        else if bomb_counter.is_none() {
            // .. We propagate the uncovering by adding the `Uncover` component to adjacent tiles
            // which will then be removed next frame
            for neighbor_entity in neighbors.neighbors.iter().flatten() {
                if let Ok((_, children, _, _)) = parents.get(*neighbor_entity) {
                    for &child in children {
                        if cover_query.get(child).is_ok() {
                            commands.entity(child).insert(Uncover);
                        }
                    }
                }
            }
        }
    }
}
