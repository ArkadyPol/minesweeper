use bevy::{color::palettes::css::GREEN, log, prelude::*};

use crate::{
    components::{CursorTimer, SettingsButtonAction, SettingsUIRoot, TextInput},
    events::{BackOriginalInput, ChangeInput, LostFocusEvent},
    input_value::InputValue,
    resources::BoardOptions,
};

use super::{
    bombs_row::bombs_row,
    common::{ButtonPosition, button},
    map_size_row::map_size_row,
};

pub fn create_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board: Res<BoardOptions>,
) {
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            Name::new("Settings UI Root"),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(50),
                ..default()
            },
            SettingsUIRoot,
            CursorTimer::default(),
            children![
                button(
                    "Start",
                    font.clone(),
                    SettingsButtonAction::Start,
                    ButtonPosition {
                        right: px(50),
                        bottom: px(50),
                    },
                ),
                map_size_row(font.clone(), board.map_size),
                bombs_row(font.clone(), board.bomb_count),
            ],
        ))
        .observe(focus_handler)
        .observe(on_change_labeled_input);

    log::info!("Settings menu initialized");
}

pub fn on_change_input(
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

fn focus_handler(
    click: On<Pointer<Click>>,
    inputs: Query<(Entity, &mut TextInput, &mut BorderColor)>,
    texts: Query<&ChildOf, With<Text>>,
    mut commands: Commands,
) {
    let original = click.original_event_target();
    for (entity, mut input, mut border) in inputs {
        if original == entity
            || texts
                .get(original)
                .is_ok_and(|parent| parent.parent() == entity)
        {
            input.focused = true;
            *border = BorderColor::all(GREEN);
        } else {
            if input.focused == true {
                commands.trigger(LostFocusEvent(entity));
            }
        }
    }
}

fn on_change_labeled_input(
    change: On<ChangeInput>,
    mut board: ResMut<BoardOptions>,
    mut commands: Commands,
) {
    log::info!("{:?}", change.event());

    let label = match &change.label {
        Some(name) => name,
        None => return,
    };

    let mut res = || {
        match label.as_str() {
            "Width" => {
                if let InputValue::Int(val) = change.value {
                    let width = u16::try_from(val).map_err(|e| e.to_string())?;
                    return board.set_width(width);
                }
            }
            "Height" => {
                if let InputValue::Int(val) = change.value {
                    let height = u16::try_from(val).map_err(|e| e.to_string())?;
                    return board.set_height(height);
                }
            }
            "Bombs" => {
                if let InputValue::Int(val) = change.value {
                    let bombs = u16::try_from(val).map_err(|e| e.to_string())?;
                    return board.set_bomb_count(bombs);
                }
            }
            _ => {}
        }
        Ok(())
    };

    if let Err(err) = res() {
        log::error!("{}", err);
        let value = match label.as_str() {
            "Width" => InputValue::from(board.map_size.0 as i32),
            "Height" => InputValue::from(board.map_size.1 as i32),
            "Bombs" => InputValue::from(board.bomb_count as i32),
            _ => unreachable!(),
        };
        commands.trigger(BackOriginalInput {
            entity: change.original_event_target(),
            value,
        })
    }

    log::info!("Updated BoardOptions: {:?}", *board);
}
