use crate::{Board, BoardAssets, TileMarkEvent, components::Flag};
use bevy::prelude::*;

pub fn mark_tiles(
    event: On<TileMarkEvent>,
    mut commands: Commands,
    board: Res<Board>,
    board_assets: Res<BoardAssets>,
) {
    if event.mark {
        commands.entity(event.entity).with_child((
            Name::new("Flag"),
            Sprite {
                custom_size: Some(Vec2::splat(board.tile_size)),
                color: board_assets.flag_material.color,
                image: board_assets.flag_material.texture.clone(),
                ..default()
            },
            Transform::from_xyz(0., 0., 1.),
            Pickable::default(),
            Flag,
        ));
    } else {
        commands.entity(event.entity).despawn_children();
    }
}
