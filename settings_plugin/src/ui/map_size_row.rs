use bevy::prelude::*;

use super::common::{field, label};

pub fn map_size_row(font: Handle<Font>, (width, height): (u16, u16)) -> impl Bundle {
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
            label(font.clone(), "Map size"),
            field(font.clone(), "Width", width as i32),
            field(font.clone(), "Height", height as i32)
        ],
    )
}
