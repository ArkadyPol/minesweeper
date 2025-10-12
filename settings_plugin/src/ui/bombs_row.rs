use bevy::{ecs::relationship::RelatedSpawner, prelude::*};

use super::{
    common::{label, text_input},
    root::on_change_input,
};

pub fn bombs_row(font: Handle<Font>, bomb_count: u16) -> impl Bundle {
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
            // Bombs
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                },
                Children::spawn(SpawnWith(move |sub: &mut RelatedSpawner<ChildOf>| {
                    sub.spawn(label(font.clone(), "Bombs"));
                    text_input(sub, font.clone(), bomb_count as i32);
                })),
            ))
            .observe(on_change_input);
        })),
    )
}
