use bevy::{prelude::*, ui_widgets::observe};

use crate::{events::ChangeInput, input_value::InputValue};

use super::{label, text_input};

pub fn field(
    label_txt: impl Into<String> + Clone,
    init_value: impl Into<InputValue>,
) -> impl Bundle {
    (
        Name::new("Field"),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        children![label(label_txt), text_input(init_value)],
        observe(on_change_input),
    )
}

fn on_change_input(
    mut change: On<ChangeInput>,
    children_query: Query<&Children>,
    label_query: Query<&Name, With<Label>>,
) {
    let children = children_query.get(change.entity).unwrap();
    for &child in children {
        if let Ok(name) = label_query.get(child) {
            change.label = Some(name.into());
        }
    }
}
