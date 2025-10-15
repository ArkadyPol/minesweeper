use bevy::prelude::*;

use super::common::field;

pub fn tile_padding_row(tile_padding: f32) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(16),
            ..default()
        },
        children![field("Tile padding", tile_padding)],
    )
}
