use bevy::{
    color::palettes::css::{DARK_GRAY, DARK_GREEN, GRAY},
    prelude::*,
};

use crate::{components::SettingsButtonAction, events::CreateGameEvent};

pub struct ButtonPosition {
    pub right: Val,
    pub bottom: Val,
}

pub fn button(
    label: &str,
    font: Handle<Font>,
    action: SettingsButtonAction,
    button_position: ButtonPosition,
) -> impl Bundle {
    (
        Node {
            width: px(250),
            padding: px(16).all(),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            right: button_position.right,
            bottom: button_position.bottom,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::from(GRAY)),
        Button,
        action,
        children![(
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    )
}

pub fn menu_action(
    interaction_query: Query<
        (&Interaction, &SettingsButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut create_game: MessageWriter<CreateGameEvent>,
) {
    for (interaction, action) in interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                SettingsButtonAction::Start => {
                    create_game.write(CreateGameEvent);
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
