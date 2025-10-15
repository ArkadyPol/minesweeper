use bevy::{
    color::palettes::css::{GRAY, GREEN},
    ecs::relationship::RelatedSpawner,
    prelude::*,
    ui::Checked,
    ui_widgets::{Checkbox, ValueChange, checkbox_self_update, observe},
};

use crate::events::ChangeInput;

use super::common::label;

pub fn safe_start_row(safe_start: bool) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(16),
            ..default()
        },
        Children::spawn(SpawnWith(
            move |parent: &mut RelatedSpawner<'_, ChildOf>| {
                let mut cmd = parent.spawn((
                    Name::new("Checkbox"),
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                        ..default()
                    },
                    Checkbox,
                    children![
                        (
                            Node {
                                width: px(16),
                                height: px(16),
                                border: px(2).all(),
                                ..default()
                            },
                            BorderColor::all(Color::BLACK),
                            if safe_start {
                                BackgroundColor(Color::from(GREEN))
                            } else {
                                BackgroundColor(Color::from(GRAY))
                            },
                        ),
                        label("Safe start")
                    ],
                    observe(checkbox_self_update),
                    observe(on_value_change),
                ));

                if safe_start {
                    cmd.insert(Checked);
                }
            },
        )),
    )
}

fn on_value_change(
    value_change: On<ValueChange<bool>>,
    query: Query<&Children, With<Checkbox>>,
    mut backgrounds: Query<&mut BackgroundColor, Without<Label>>,
    label_query: Query<&Name, With<Label>>,
    mut commands: Commands,
) {
    let children = query.get(value_change.source).unwrap();

    let inner = children[0];
    let label = children[1];

    let mut background = backgrounds.get_mut(inner).unwrap();
    let name = label_query.get(label).unwrap();

    *background = if value_change.value {
        BackgroundColor(Color::from(GREEN))
    } else {
        BackgroundColor(Color::from(GRAY))
    };

    commands.trigger(ChangeInput {
        entity: value_change.source,
        value: value_change.value.into(),
        label: Some(name.into()),
    });
}
