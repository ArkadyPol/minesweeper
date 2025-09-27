use crate::{Board, BoardAssets, TileMarkEvent, components::Flag};
use bevy::prelude::*;

pub fn mark_tiles(
    mut commands: Commands,
    board: Res<Board>,
    board_assets: Res<BoardAssets>,
    mut tile_mark_event_rdr: MessageReader<TileMarkEvent>,
) {
    for &TileMarkEvent(entity, mark) in tile_mark_event_rdr.read() {
        if mark {
            commands.entity(entity).with_child((
                Name::new("Flag"),
                Sprite {
                    custom_size: Some(Vec2::splat(board.tile_size)),
                    color: board_assets.flag_material.color,
                    image: board_assets.flag_material.texture.clone(),
                    ..Default::default()
                },
                Transform::from_xyz(0., 0., 1.),
                Pickable::default(),
                Flag,
            ));
        } else {
            commands.entity(entity).despawn_children();
        }
    }
}
