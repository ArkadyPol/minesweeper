use bevy::{log, prelude::*};

#[cfg(feature = "simple_neighbors")]
use crate::components::Neighbors;
#[cfg(feature = "hierarchical_neighbors")]
use crate::components::{GridChildOf, GridMap};
#[cfg(all(feature = "hierarchical_neighbors", not(feature = "simple_neighbors")))]
use crate::find_neighbors;
use crate::{
    BoardOptions,
    components::{Bomb, BombNeighbor, Coordinates, TileCover, Uncover},
    events::{BoardCompletedEvent, BombExplosionEvent, PropagateUncoverEvent, TileTriggerEvent},
};
#[cfg(not(any(feature = "simple_neighbors", feature = "hierarchical_neighbors")))]
use crate::{SQUARE_COORDINATES, resources::Board};

pub fn trigger_event_handler(event: On<TileTriggerEvent>, mut commands: Commands) {
    commands.entity(event.0).insert(Uncover);
}

pub fn uncover_tiles(
    mut commands: Commands,
    children: Query<(Entity, &ChildOf), With<Uncover>>,
    children_query: Query<&Children>,
    #[cfg(not(any(feature = "simple_neighbors", feature = "hierarchical_neighbors")))] board: Res<
        Board,
    >,
    #[cfg(not(feature = "simple_neighbors"))] parents: Query<(
        Option<&Bomb>,
        Option<&BombNeighbor>,
        &Coordinates,
    )>,
    #[cfg(feature = "simple_neighbors")] parents: Query<(
        &Neighbors,
        Option<&Bomb>,
        Option<&BombNeighbor>,
    )>,
    cover_query: Query<(), (With<TileCover>, Without<Uncover>)>,
    board_options: Option<Res<BoardOptions>>,
    #[cfg(feature = "hierarchical_neighbors")] query_neighbors_2: Query<(&GridMap, &Coordinates)>,
    #[cfg(feature = "hierarchical_neighbors")] query_neighbor_of: Query<&GridChildOf>,
) {
    let options = match board_options {
        None => BoardOptions::default(), // If no options is set we use the default one
        Some(o) => o.clone(),
    };
    let bomb_count = options.bomb_count as usize;
    let cover_count = cover_query.count();
    let mut is_finished = false;

    // We iterate through tile covers to uncover
    for (entity, parent) in children.iter() {
        // we destroy the tile cover entity
        commands.entity(entity).despawn();

        let parent_entity = parent.parent();

        #[cfg(feature = "simple_neighbors")]
        let (neighbors, bomb, bomb_counter) = match parents.get(parent_entity) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{}", e);
                continue;
            }
        };

        #[cfg(not(feature = "simple_neighbors"))]
        let (bomb, bomb_counter, &coords) = match parents.get(parent_entity) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{}", e);
                continue;
            }
        };

        if bomb.is_some() {
            log::info!("Boom !");
            commands.trigger(BombExplosionEvent(parent_entity));
            return;
        }

        if cover_count == bomb_count && !is_finished {
            log::info!("Board completed");
            commands.trigger(BoardCompletedEvent);
            is_finished = true;
        }
        // If the tile is empty..
        if bomb_counter.is_none() {
            #[cfg(not(any(feature = "simple_neighbors", feature = "hierarchical_neighbors")))]
            for neighbor_entity in SQUARE_COORDINATES
                .map(|tuple| coords + tuple)
                .into_iter()
                .filter_map(|c| board.coords_map.get(&c).copied())
            {
                commands.trigger(PropagateUncoverEvent::new(neighbor_entity, &children_query));
            }
            #[cfg(feature = "simple_neighbors")]
            for neighbor_entity in neighbors.iter().flatten() {
                commands.trigger(PropagateUncoverEvent::new(
                    *neighbor_entity,
                    &children_query,
                ));
            }
            #[cfg(all(feature = "hierarchical_neighbors", not(feature = "simple_neighbors")))]
            for neighbor_entity in find_neighbors(
                parent_entity,
                coords,
                &query_neighbors_2,
                &query_neighbor_of,
            ) {
                commands.trigger(PropagateUncoverEvent::new(neighbor_entity, &children_query));
            }
        }
    }
}

pub fn on_uncover_handler(
    event: On<PropagateUncoverEvent>,
    cover_query: Query<(), (With<TileCover>, Without<Uncover>)>,
    mut commands: Commands,
) {
    if cover_query.get(event.entity).is_ok() {
        commands.entity(event.entity).insert(Uncover);
    }
}
