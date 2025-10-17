use bevy::{
    color::palettes::css::{BLUE, GRAY},
    ecs::relationship::RelatedSpawner,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioButton, RadioGroup, ValueChange, observe},
};

use crate::{
    components::{BoardPositionRow, Controlled, Controls},
    resources::BoardPosition,
};

use super::common::{field, label, select_button};

pub fn position_row(pos: &BoardPosition) -> impl Bundle {
    let pos = pos.clone();

    let node = Node {
        width: percent(65.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceEvenly,
        column_gap: px(16),
        ..default()
    };

    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: px(16),
            ..default()
        },
        BoardPositionRow,
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
                    let is_centered = matches!(pos, BoardPosition::Centered { .. });
                    select_button(
                        sub,
                        "Centered",
                        is_centered,
                        (
                            Name::new("Centered"),
                            Node {
                                display: if is_centered {
                                    Display::Flex
                                } else {
                                    Display::None
                                },
                                ..node.clone()
                            },
                            children![
                                label("Offset"),
                                field("X", 0.0),
                                field("Y", 0.0),
                                field("Z", 0.0),
                            ],
                        ),
                    );
                    let is_custom = matches!(pos, BoardPosition::Custom { .. });
                    select_button(
                        sub,
                        "Custom",
                        is_custom,
                        (
                            Name::new("Custom"),
                            Node {
                                display: if is_custom {
                                    Display::Flex
                                } else {
                                    Display::None
                                },
                                ..node
                            },
                            children![field("X", 0.0), field("Y", 0.0), field("Z", 0.0),],
                        ),
                    );
                })),
                observe(button_group_update),
            ),
        ],
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

pub fn bind_controls_to_board_pos(
    event: On<Add, Controls>,
    query_ctrl: Query<&Controls>,
    parents: Query<&ChildOf>,
    board_pos: Query<(), With<BoardPositionRow>>,
    mut commands: Commands,
) {
    let button_entity = query_ctrl.get(event.entity).unwrap().0;

    let mut row_entity = None;

    for parent in parents.iter_ancestors(button_entity) {
        if board_pos.get(parent).is_ok() {
            row_entity = Some(parent);
            break;
        }
    }

    let Some(row_entity) = row_entity else {
        return;
    };

    commands.entity(event.entity).insert(ChildOf(row_entity));
}
