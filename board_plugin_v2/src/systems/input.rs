use bevy::{log, prelude::*};

use crate::{
    components::{Coordinates, Flag, TileCover},
    events::{TileMarkEvent, TileTriggerEvent},
};

pub fn input_handling(
    click: On<Pointer<Click>>,
    tile_query: Query<&Coordinates>,
    cover_query: Query<&ChildOf, With<TileCover>>,
    flag_query: Query<(), With<Flag>>,
    mut commands: Commands,
) {
    if let Ok(parent) = cover_query.get(click.entity) {
        if let Ok(&coordinates) = tile_query.get(parent.parent()) {
            let original = click.original_event_target();
            let is_flag = flag_query.get(original).is_ok();
            match click.button {
                PointerButton::Primary => {
                    log::info!("Trying to uncover tile on {}", coordinates);
                    if !is_flag {
                        commands.trigger(TileTriggerEvent(click.entity));
                    }
                }
                PointerButton::Secondary => {
                    log::info!("Trying to mark tile on {}", coordinates);
                    commands.trigger(TileMarkEvent {
                        entity: click.entity,
                        mark: !is_flag,
                    });
                }
                _ => (),
            }
        }
    }
}
