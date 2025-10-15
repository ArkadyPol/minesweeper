use bevy::prelude::*;

use super::common::field;

pub fn bombs_row(bomb_count: u16) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(16),
            ..default()
        },
        children![field("Bombs", bomb_count as i32)],
    )
}
