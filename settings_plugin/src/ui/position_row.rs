use bevy::{
    color::palettes::css::{BLUE, GRAY},
    ecs::relationship::RelatedSpawner,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioButton, RadioGroup, ValueChange, observe},
};

use crate::resources::BoardPosition;

use super::common::{field, label, select_button};

pub fn position_row(pos: &BoardPosition) -> impl Bundle {
    let pos = pos.clone();
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
            label("Board position"),
            (
                Name::new("Button Group"),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                },
                RadioGroup,
                Children::spawn(SpawnWith(move |sub: &mut RelatedSpawner<'_, ChildOf>| {
                    select_button(
                        sub,
                        "Centered",
                        matches!(pos, BoardPosition::Centered { .. }),
                    );
                    select_button(sub, "Custom", matches!(pos, BoardPosition::Custom { .. }));
                })),
                observe(button_group_update),
            )
        ],
    )
}

pub fn button_group_update(
    value_change: On<ValueChange<Entity>>,
    query: Query<&Children, With<RadioGroup>>,
    mut checked: Query<(Has<Checked>, &mut BackgroundColor), With<RadioButton>>,
    mut commands: Commands,
) {
    let children = query.get(value_change.source).unwrap();
    for &child in children {
        let (checked, mut background) = checked.get_mut(child).unwrap();
        if checked {
            commands.entity(child).remove::<Checked>();
            *background = BackgroundColor(Color::from(GRAY));
        }
        if child == value_change.value {
            commands.entity(child).insert(Checked);
            *background = BackgroundColor(Color::from(BLUE));
        }
    }
}
