use bevy::{
    color::palettes::css::{BLUE, GRAY},
    ecs::relationship::RelatedSpawner,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioButton, RadioGroup, ValueChange, observe},
};
use ron::to_string;

use crate::{
    components::{Controlled, Controls, TextInput},
    events::ChangeInput,
    input_value::InputValue,
    resources::BoardPosition,
};

use super::common::{field, label, select_button};

pub fn position_row(pos: &BoardPosition, controls: [Entity; 2]) -> impl Bundle {
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
        Children::spawn((
            Spawn(label("Board position")),
            Spawn((
                Name::new("Button Group"),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                },
                RadioGroup,
                Children::spawn(SpawnWith(move |sub: &mut RelatedSpawner<'_, ChildOf>| {
                    let is_centered = matches!(&pos, BoardPosition::Centered { .. });
                    let is_custom = matches!(&pos, BoardPosition::Custom { .. });

                    select_button(sub, "Centered", is_centered, controls[0]);
                    select_button(sub, "Custom", is_custom, controls[1]);
                })),
                observe(button_group_update),
                observe(on_value_change),
            )),
            WithRelated::new(controls),
        )),
    )
}

fn controls_view(caption: &str, selected: bool, vec: Vec3) -> impl Bundle {
    let is_centered = caption == "Centered";

    (
        Name::new(caption.to_string()),
        Node {
            width: percent(65.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(16),
            display: if selected {
                Display::Flex
            } else {
                Display::None
            },
            ..default()
        },
        Children::spawn(SpawnWith(
            move |parent: &mut RelatedSpawner<'_, ChildOf>| {
                if is_centered {
                    parent.spawn(label("Offset"));
                }
                parent.spawn(field("X", vec.x));
                parent.spawn(field("Y", vec.y));
                parent.spawn(field("Z", vec.z));
            },
        )),
        observe(on_change_input),
    )
}

pub fn button_group_update(
    value_change: On<ValueChange<Entity>>,
    query: Query<&Children, With<RadioGroup>>,
    mut buttons: Query<(Has<Checked>, &mut BackgroundColor, &Controlled), With<RadioButton>>,
    mut controls: Query<&mut Node, With<Controls>>,
    mut commands: Commands,
) {
    let children = query.get(value_change.source).unwrap();
    for &child in children {
        let (is_checked, mut background, controlled) = buttons.get_mut(child).unwrap();
        let mut node = controls.get_mut(controlled[0]).unwrap();
        if child == value_change.value {
            commands.entity(child).insert(Checked);
            *background = BackgroundColor(Color::from(BLUE));
            node.display = Display::Flex;
        } else if is_checked {
            commands.entity(child).remove::<Checked>();
            *background = BackgroundColor(Color::from(GRAY));
            node.display = Display::None;
        }
    }
}

pub fn on_value_change(
    value_change: On<ValueChange<Entity>>,
    buttons: Query<&Controlled, With<RadioButton>>,
    mut commands: Commands,
) {
    let controlled = buttons.get(value_change.value).unwrap();
    let node_entity = controlled[0];

    commands.trigger(ChangeInput {
        entity: node_entity,
        value: InputValue::from(0.0),
        label: None,
    });
}

fn on_change_input(
    mut change: On<ChangeInput>,
    names: Query<(&Name, &Children)>,
    labels: Query<&Name, With<Label>>,
    inputs: Query<&TextInput>,
) {
    let (variant, children) = names.get(change.entity).unwrap();
    let mut vec = Vec3::default();

    for &child in children {
        // Field
        if let Ok((_name, children)) = names.get(child) {
            let mut label = String::new();
            let mut input_value = InputValue::Float(0.0);

            for &child in children {
                // Label
                if let Ok(name) = labels.get(child) {
                    label = name.into();
                }
                // Text Input
                if let Ok(input) = inputs.get(child) {
                    input_value = input.value.clone();
                }
            }

            match label.as_str() {
                "X" => {
                    if let InputValue::Float(x) = input_value {
                        vec.x = x;
                    }
                }
                "Y" => {
                    if let InputValue::Float(y) = input_value {
                        vec.y = y;
                    }
                }
                "Z" => {
                    if let InputValue::Float(z) = input_value {
                        vec.z = z;
                    }
                }
                _ => {}
            }
        }
    }

    let board_pos = match variant.as_str() {
        "Centered" => BoardPosition::Centered { offset: vec },
        "Custom" => BoardPosition::Custom(vec),
        _ => unreachable!(),
    };

    change.label = Some("Board position".into());
    change.value = InputValue::from(to_string(&board_pos).unwrap());
}

pub fn spawn_board_pos_controls(pos: &BoardPosition, commands: &mut Commands) -> [Entity; 2] {
    let (is_centered, centered_vec) = match &pos {
        BoardPosition::Centered { offset } => (true, *offset),
        _ => (false, Vec3::default()),
    };
    let (is_custom, custom_vec) = match &pos {
        BoardPosition::Custom(vec) => (true, *vec),
        _ => (false, Vec3::default()),
    };

    let centered_view = controls_view("Centered", is_centered, centered_vec);
    let custom_view = controls_view("Custom", is_custom, custom_vec);

    [
        commands.spawn(centered_view).id(),
        commands.spawn(custom_view).id(),
    ]
}
