use bevy::{
    ecs::relationship::RelatedSpawner,
    prelude::*,
    ui_widgets::{RadioGroup, observe},
};
use ron::to_string;

use crate::{
    components::TextInput, events::ChangeInput, input_value::InputValue, resources::TileSize,
};

use super::{
    common::{field, label, select_button},
    position_row::{button_group_update, on_value_change},
};

pub fn tile_size_row(tile_size: &TileSize, controls: [Entity; 2]) -> impl Bundle {
    let tile_size = tile_size.clone();

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
            Spawn(label("Tile size")),
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
                    let is_adaptive = matches!(&tile_size, TileSize::Adaptive { .. });
                    let is_fixed = matches!(&tile_size, TileSize::Fixed { .. });

                    select_button(sub, "Adaptive", is_adaptive, controls[0]);
                    select_button(sub, "Fixed", is_fixed, controls[1]);
                })),
                observe(button_group_update),
                observe(on_value_change),
            )),
            WithRelated::new(controls),
        )),
    )
}

fn controls_view(caption: &str, selected: bool, vec: Vec<(String, f32)>) -> impl Bundle {
    (
        Name::new(caption.to_string()),
        Node {
            width: percent(60.0),
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
        Children::spawn(SpawnIter(
            vec.into_iter().map(|(label, value)| field(label, value)),
        )),
        observe(on_change_input),
    )
}

fn on_change_input(
    mut change: On<ChangeInput>,
    names: Query<(&Name, &Children)>,
    labels: Query<&Name, With<Label>>,
    inputs: Query<&TextInput>,
) {
    let (variant, children) = names.get(change.entity).unwrap();
    let mut min = 0.0;
    let mut max = 0.0;
    let mut value = 0.0;

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
                "Min" => {
                    if let InputValue::Float(v) = input_value {
                        min = v;
                    }
                }
                "Max" => {
                    if let InputValue::Float(v) = input_value {
                        max = v;
                    }
                }
                "Value" => {
                    if let InputValue::Float(v) = input_value {
                        value = v;
                    }
                }
                _ => {}
            }
        }
    }

    let tile_size = match variant.as_str() {
        "Adaptive" => TileSize::Adaptive { min, max },
        "Fixed" => TileSize::Fixed(value),
        _ => unreachable!(),
    };

    change.label = Some("Tile size".into());
    change.value = InputValue::from(to_string(&tile_size).unwrap());
}

pub fn spawn_tile_size_controls(tile_size: &TileSize, commands: &mut Commands) -> [Entity; 2] {
    let (is_adaptive, adaptive_vec) = match &tile_size {
        TileSize::Adaptive { min, max } => {
            (true, vec![("Min".to_string(), *min), ("Max".into(), *max)])
        }
        _ => (false, vec![("Min".into(), 10.0), ("Max".into(), 50.0)]),
    };
    let (is_fixed, fixed_vec) = match &tile_size {
        TileSize::Fixed(value) => (true, vec![("Value".to_string(), *value)]),
        _ => (false, vec![("Value".into(), 30.0)]),
    };

    let adaptive_view = controls_view("Adaptive", is_adaptive, adaptive_vec);
    let fixed_view = controls_view("Fixed", is_fixed, fixed_vec);

    [
        commands.spawn(adaptive_view).id(),
        commands.spawn(fixed_view).id(),
    ]
}
