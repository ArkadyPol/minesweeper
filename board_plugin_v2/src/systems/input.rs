use bevy::{log, prelude::*};

use crate::{
    components::{Coordinates, Flag, TileCover},
    events::{TileMarkEvent, TileTriggerEvent},
};

pub fn input_handling(
    mut events: MessageReader<Pointer<Click>>,
    cover_query: Query<(Entity, &ChildOf), With<TileCover>>,
    tile_query: Query<&Coordinates>,
    flag_query: Query<&ChildOf, With<Flag>>,
    mut tile_trigger_ewr: MessageWriter<TileTriggerEvent>,
    mut tile_mark_ewr: MessageWriter<TileMarkEvent>,
) {
    for event in events.read() {
        if let Ok((entity, parent)) = cover_query.get(event.entity) {
            if let Ok(&coordinates) = tile_query.get(parent.parent()) {
                match event.button {
                    PointerButton::Primary => {
                        log::info!("Trying to uncover tile on {}", coordinates);
                        tile_trigger_ewr.write(TileTriggerEvent(entity));
                    }
                    PointerButton::Secondary => {
                        log::info!("Trying to mark tile on {}", coordinates);
                        tile_mark_ewr.write(TileMarkEvent(entity, true));
                    }
                    _ => (),
                }
            }
        }
        if let Ok(parent) = flag_query.get(event.entity) {
            if let Ok((entity, parent)) = cover_query.get(parent.parent()) {
                if let Ok(&coordinates) = tile_query.get(parent.parent()) {
                    match event.button {
                        PointerButton::Secondary => {
                            log::info!("Trying to unmark tile on {}", coordinates);
                            tile_mark_ewr.write(TileMarkEvent(entity, false));
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}
