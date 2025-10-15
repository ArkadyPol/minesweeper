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
    safe_start_row::safe_start_row,
    tile_padding_row::tile_padding_row,
};

pub fn create_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board: Res<BoardOptions>,
) {
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    let font_observer = commands
        .add_observer(
            move |event: On<Add, TextFont>, mut query: Query<&mut TextFont>| {
                let mut text_font = query.get_mut(event.entity).unwrap();
                text_font.font = font.clone();
            },
        )
        .id();

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
                map_size_row(board.map_size),
                bombs_row(board.bomb_count),
                tile_padding_row(board.tile_padding),
                safe_start_row(board.safe_start),
                button(
                    "Start",
                    SettingsButtonAction::Start,
                    ButtonPosition {
                        right: px(50),
                        bottom: px(50),
                        ..default()
                    },
                ),
                button(
                    "Back to Menu",
                    SettingsButtonAction::BackToMenu,
                    ButtonPosition {
                        left: px(50),
                        bottom: px(50),
                        ..default()
                    },
                ),
            ],
        ))
        .observe(focus_handler)
        .observe(on_change_labeled_input);

    commands.entity(font_observer).despawn();

    log::info!("Settings menu initialized");
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
            "Tile padding" => {
                if let InputValue::Float(tile_padding) = change.value {
                    return board.set_tile_padding(tile_padding);
                }
            }
            "Safe start" => {
                if let InputValue::Bool(safe_start) = change.value {
                    board.safe_start = safe_start;
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
            "Tile padding" => InputValue::from(board.tile_padding),
            _ => unreachable!(),
        };
        commands.trigger(BackOriginalInput {
            entity: change.original_event_target(),
            value,
        })
    }

    log::info!("Updated BoardOptions: {:?}", *board);
}
