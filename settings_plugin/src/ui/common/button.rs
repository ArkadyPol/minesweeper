use bevy::{
    color::palettes::css::{DARK_GRAY, DARK_GREEN, GRAY},
    prelude::*,
};

use crate::{
    components::SettingsButtonAction,
    events::{BackToMenuEvent, CreateGameEvent},
};

use super::text;

#[derive(Debug, Default)]
pub struct ButtonPosition {
    pub right: Val,
    pub bottom: Val,
    pub left: Val,
}

pub fn button(
    label: &str,
    action: SettingsButtonAction,
    button_position: ButtonPosition,
) -> impl Bundle {
    (
        Node {
            width: px(250),
            padding: px(16).all(),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            left: button_position.left,
            right: button_position.right,
            bottom: button_position.bottom,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::from(GRAY)),
        Button,
        action,
        children![text(32.0, label)],
    )
}

pub fn menu_action(
    interaction_query: Query<
        (&Interaction, &SettingsButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut create_game: MessageWriter<CreateGameEvent>,
    mut back_to_menu: MessageWriter<BackToMenuEvent>,
) {
    for (interaction, action) in interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                SettingsButtonAction::Start => {
                    create_game.write(CreateGameEvent);
                }
                SettingsButtonAction::BackToMenu => {
                    back_to_menu.write(BackToMenuEvent);
                }
            }
        }
    }
}

pub fn change_background_color(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = Color::from(DARK_GREEN).into();
            }
            Interaction::Hovered => {
                *background_color = Color::from(DARK_GRAY).into();
            }
            Interaction::None => {
                *background_color = Color::from(GRAY).into();
            }
        }
    }
}
