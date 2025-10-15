use bevy::prelude::*;

use super::common::{field, label};

pub fn map_size_row((width, height): (u16, u16)) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(16),
            ..default()
        },
        children![
            label("Map size"),
            field("Width", width as i32),
            field("Height", height as i32)
        ],
    )
}
