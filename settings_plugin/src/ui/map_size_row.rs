use bevy::{ecs::relationship::RelatedSpawner, prelude::*};

use super::{
    common::{label, text_input},
    root::on_change_input,
};

pub fn map_size_row(font: Handle<Font>, (width, height): (u16, u16)) -> impl Bundle {
    let font_2 = font.clone();
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
            // Map size
            row.spawn(label(font.clone(), "Map size"));
            // Width
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                },
                Children::spawn(SpawnWith(move |sub: &mut RelatedSpawner<ChildOf>| {
                    sub.spawn(label(font.clone(), "Width"));
                    text_input(sub, font.clone(), width as i32);
                })),
            ))
            .observe(on_change_input);
            // Height
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                },
                Children::spawn(SpawnWith(move |sub: &mut RelatedSpawner<ChildOf>| {
                    sub.spawn(label(font_2.clone(), "Height"));
                    text_input(sub, font_2.clone(), height as i32);
                })),
            ))
            .observe(on_change_input);
        })),
    )
}
