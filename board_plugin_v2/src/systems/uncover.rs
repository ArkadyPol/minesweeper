use bevy::{log, prelude::*};

use crate::{
    components::{Bomb, BombNeighbor, Neighbors, TileCover, Uncover},
    events::{BoardCompletedEvent, BombExplosionEvent, TileTriggerEvent},
    resources::BoardOptions,
};

pub fn trigger_event_handler(
    mut commands: Commands,
    mut tile_trigger_evr: MessageReader<TileTriggerEvent>,
) {
    for TileTriggerEvent(entity) in tile_trigger_evr.read() {
        commands.entity(*entity).insert(Uncover);
    }
}

pub fn uncover_tiles(
    mut commands: Commands,
    children: Query<(Entity, &ChildOf), With<Uncover>>,
    parents: Query<(&Neighbors, &Children, Option<&Bomb>, Option<&BombNeighbor>)>,
    cover_query: Query<(), (With<TileCover>, Without<Uncover>)>,
    board_options: Option<Res<BoardOptions>>,
    mut board_completed_event_wr: MessageWriter<BoardCompletedEvent>,
    mut bomb_explosion_event_wr: MessageWriter<BombExplosionEvent>,
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
            board_completed_event_wr.write(BoardCompletedEvent);
        }

        if bomb.is_some() {
            log::info!("Boom !");
            bomb_explosion_event_wr.write(BombExplosionEvent);
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
