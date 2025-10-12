use bevy::{ecs::relationship::RelatedSpawner, prelude::*};

use super::common::field;

pub fn tile_padding_row(font: Handle<Font>, tile_padding: f32) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(16),
            ..default()
        },
        Children::spawn(SpawnWith(move |row: &mut RelatedSpawner<ChildOf>| {
            // Tile padding
            field(row, font.clone(), "Tile padding", tile_padding);
        })),
    )
}
