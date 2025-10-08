pub mod components;
pub mod events;
pub mod resources;

use bevy::{
    color::palettes::css::{DARK_GRAY, DARK_GREEN, GRAY, GREEN},
    input::keyboard::Key,
    log,
    prelude::*,
};
use ron::ser::{PrettyConfig, to_string_pretty};
use std::fs;

use components::{CursorTimer, InputValue, SettingsButtonAction, SettingsUIRoot, TextInput};
use events::{CreateGameEvent, LostFocusEvent};
use resources::{BoardAssets, BoardOptions, SpriteMaterial};

pub struct SettingsPlugin<T> {
    pub running_state: T,
}

struct ButtonPosition {
    right: Val,
    bottom: Val,
}

impl<T: States> Plugin for SettingsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(self.running_state.clone()),
            (Self::setup_board, Self::create_menu).chain(),
        )
        .add_systems(
            Update,
            (
                (Self::change_background_color, Self::menu_action).chain(),
                Self::keyboard_handler,
                Self::in_focus_cursor,
            )
                .run_if(in_state(self.running_state.clone())),
        )
        .add_systems(OnExit(self.running_state.clone()), Self::cleanup_menu);
        app.add_message::<CreateGameEvent>();
    }
}

impl<T> SettingsPlugin<T> {
    fn create_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
        let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
        let inputs = vec![
            Self::text_input(font.clone(), 20.1f32, &mut commands),
            Self::text_input(font.clone(), 20, &mut commands),
            Self::text_input(font.clone(), "abc".to_string(), &mut commands),
        ];
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
                    ..Default::default()
                },
                SettingsUIRoot,
                CursorTimer::default(),
                children![Self::button(
                    "Start",
                    font.clone(),
                    SettingsButtonAction::Start,
                    ButtonPosition {
                        right: px(50),
                        bottom: px(50),
                    }
                ),],
            ))
            .observe(focus_handler)
            .add_children(&inputs);

        log::info!("Settings menu initialized");
    }
    fn setup_board(mut commands: Commands, asset_server: Res<AssetServer>) {
        // Board plugin options
        let board_options: BoardOptions = fs::read_to_string("board_options.ron")
            .map(|s| ron::from_str(&s).unwrap())
            .unwrap();

        log::info!("{:?}", board_options);

        commands.insert_resource(board_options.clone());

        // Board assets
        commands.insert_resource(BoardAssets {
            label: "Default".to_string(),
            board_material: SpriteMaterial {
                color: Color::WHITE,
                ..Default::default()
            },
            tile_material: SpriteMaterial {
                color: Color::from(DARK_GRAY),
                ..Default::default()
            },
            covered_tile_material: SpriteMaterial {
                color: Color::from(GRAY),
                ..Default::default()
            },
            bomb_counter_font: asset_server.load("fonts/pixeled.ttf"),
            bomb_counter_colors: BoardAssets::default_colors(),
            flag_material: SpriteMaterial {
                texture: asset_server.load("sprites/flag.png"),
                color: Color::WHITE,
            },
            bomb_material: SpriteMaterial {
                texture: asset_server.load("sprites/bomb.png"),
                color: Color::WHITE,
            },
        });
    }

    fn button(
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
                ..Default::default()
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

    fn text_input(
        font: Handle<Font>,
        value: impl Into<InputValue>,
        commands: &mut Commands,
    ) -> Entity {
        let value: InputValue = value.into();
        commands
            .spawn((
                Name::new("Text Input"),
                Node {
                    width: px(150),
                    padding: px(6).all(),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: px(4).all(),
                    ..Default::default()
                },
                BackgroundColor(Color::from(GRAY)),
                TextInput {
                    value: value.clone(),
                    ..Default::default()
                },
                children![(
                    Text::new(value),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ))
            .observe(on_lost_focus_handler)
            .id()
    }

    fn menu_action(
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

    fn change_background_color(
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

    fn cleanup_menu(
        mut commands: Commands,
        query: Query<Entity, With<SettingsUIRoot>>,
        board_options: Res<BoardOptions>,
    ) {
        let entity = query.single().unwrap();
        commands.entity(entity).despawn();
        log::info!("Settings menu closed");

        fs::write(
            "board_options.ron",
            to_string_pretty(&board_options.into_inner(), PrettyConfig::default()).unwrap(),
        )
        .expect("Error saving settings");
    }

    fn keyboard_handler(
        inputs: Query<(Entity, &mut TextInput, &Children)>,
        keys: Res<ButtonInput<Key>>,
        mut text_query: Query<&mut Text>,
        mut commands: Commands,
        mut timer: Query<&mut CursorTimer, With<SettingsUIRoot>>,
    ) {
        let mut timer = timer.single_mut().unwrap();

        for (entity, mut input, children) in inputs {
            if !input.focused {
                continue;
            }

            let text_entity = find_text_child_entity(children, &text_query);
            let mut text = text_query.get_mut(text_entity).unwrap();

            for key in keys.get_just_pressed() {
                timer.0.reset();

                let mut chars: Vec<char> = text.0.chars().collect();

                if !input.is_cursor_inserted {
                    chars.insert(input.cursor_pos, '|');
                    input.is_cursor_inserted = true;
                }

                match key {
                    Key::Character(s) => {
                        for c in s.chars() {
                            chars.insert(input.cursor_pos, c);
                            input.cursor_pos += 1;
                        }
                    }
                    Key::Space => {
                        chars.insert(input.cursor_pos, ' ');
                        input.cursor_pos += 1;
                    }
                    Key::ArrowLeft => {
                        if input.cursor_pos > 0 {
                            chars.remove(input.cursor_pos);
                            chars.insert(input.cursor_pos - 1, '|');
                            input.cursor_pos -= 1;
                        }
                    }
                    Key::ArrowRight => {
                        if input.cursor_pos < chars.len() - 1 {
                            chars.remove(input.cursor_pos);
                            chars.insert(input.cursor_pos + 1, '|');
                            input.cursor_pos += 1;
                        }
                    }
                    Key::Backspace => {
                        if input.cursor_pos > 0 {
                            chars.remove(input.cursor_pos - 1);
                            input.cursor_pos -= 1;
                        }
                    }
                    Key::Delete => {
                        if input.cursor_pos < chars.len() - 1 {
                            chars.remove(input.cursor_pos);
                        }
                    }
                    Key::Enter => {
                        commands.trigger(LostFocusEvent(entity));
                    }
                    _ => {}
                }

                text.0 = chars.into_iter().collect();
            }
        }
    }

    fn in_focus_cursor(
        inputs: Query<(&mut TextInput, &Children)>,
        mut text_query: Query<&mut Text>,
        mut timer: Query<&mut CursorTimer, With<SettingsUIRoot>>,
        time: Res<Time>,
    ) {
        let mut timer = timer.single_mut().unwrap();

        timer.0.tick(time.delta());

        if !timer.0.just_finished() {
            return;
        }

        for (mut input, children) in inputs {
            if !input.focused {
                continue;
            }
            let text_entity = find_text_child_entity(children, &text_query);
            let mut text = text_query.get_mut(text_entity).unwrap();
            let mut chars: Vec<char> = text.0.chars().collect();

            if input.is_cursor_inserted {
                chars.remove(input.cursor_pos);
            } else {
                chars.insert(input.cursor_pos, '|');
            }

            text.0 = chars.into_iter().collect();

            input.is_cursor_inserted = !input.is_cursor_inserted;
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

fn on_lost_focus_handler(
    event: On<LostFocusEvent>,
    mut inputs: Query<(&mut TextInput, &Children, &mut BorderColor)>,
    mut text_query: Query<&mut Text>,
) {
    let (mut input, children, mut border) = inputs.get_mut(event.0).unwrap();
    input.focused = false;
    *border = BorderColor::all(Color::NONE);

    let text_entity = find_text_child_entity(children, &text_query);
    let mut text = text_query.get_mut(text_entity).unwrap();

    if input.is_cursor_inserted {
        let mut chars: Vec<char> = text.0.chars().collect();
        chars.remove(input.cursor_pos);
        input.is_cursor_inserted = false;
        text.0 = chars.into_iter().collect();
    }

    if let Err(err) = input.value.parse_and_mut(&text.0) {
        log::error!("{}", err);
        text.0 = input.value.as_string();
        let chars: Vec<char> = text.0.chars().collect();
        input.cursor_pos = input.cursor_pos.min(chars.len());
    }
}

fn find_text_child_entity(children: &Children, text_query: &Query<&mut Text>) -> Entity {
    children
        .iter()
        .find(|child| text_query.get(*child).is_ok())
        .expect("TextInput must have a Text child")
}
