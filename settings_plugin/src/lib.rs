pub mod components;
pub mod events;
pub mod resources;

use bevy::{
    color::palettes::css::{DARK_GRAY, DARK_GREEN, GRAY, GREEN},
    ecs::relationship::RelatedSpawner,
    input::keyboard::Key,
    log,
    prelude::*,
    text::ComputedTextBlock,
};
use ron::ser::{PrettyConfig, to_string_pretty};
use std::fs;

use components::{CursorTimer, InputValue, SettingsButtonAction, SettingsUIRoot, TextInput};
use events::{BackOriginalInput, ChangeInput, CreateGameEvent, LostFocusEvent, SetCursorPosEvent};
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
    fn create_menu(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        board: Res<BoardOptions>,
    ) {
        let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
        let font_2 = font.clone();

        let map_size_x = board.map_size.0;
        let map_size_y = board.map_size.1;

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
                    Self::button(
                        "Start",
                        font.clone(),
                        SettingsButtonAction::Start,
                        ButtonPosition {
                            right: px(50),
                            bottom: px(50),
                        },
                    ),
                    (
                        Node {
                            width: percent(100.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceEvenly,
                            column_gap: px(16),
                            ..default()
                        },
                        Children::spawn(SpawnWith(move |row: &mut RelatedSpawner<ChildOf>| {
                            // Map size
                            row.spawn(Self::label(font.clone(), "Map size"));
                            // Width
                            row.spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(8),
                                    ..default()
                                },
                                Children::spawn(SpawnWith(
                                    move |sub: &mut RelatedSpawner<ChildOf>| {
                                        sub.spawn(Self::label(font.clone(), "Width"));
                                        Self::text_input(sub, font.clone(), map_size_x as i32);
                                    },
                                )),
                            ))
                            .observe(on_change_input);
                            // Height
                            row.spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(8),
                                    ..default()
                                },
                                Children::spawn(SpawnWith(
                                    move |sub: &mut RelatedSpawner<ChildOf>| {
                                        sub.spawn(Self::label(font_2.clone(), "Height"));
                                        Self::text_input(sub, font_2.clone(), map_size_y as i32);
                                    },
                                )),
                            ))
                            .observe(on_change_input);
                        }))
                    )
                ],
            ))
            .observe(focus_handler)
            .observe(on_change_labeled_input);

        log::info!("Settings menu initialized");
    }
    fn setup_board(mut commands: Commands, asset_server: Res<AssetServer>) {
        // Board plugin options
        let board_options: BoardOptions = fs::read_to_string("board_options.ron")
            .map(|s| ron::from_str(&s).unwrap())
            .unwrap();

        log::info!("{:?}", board_options);

        commands.insert_resource(board_options);

        // Board assets
        commands.insert_resource(BoardAssets {
            label: "Default".to_string(),
            board_material: SpriteMaterial {
                color: Color::WHITE,
                ..default()
            },
            tile_material: SpriteMaterial {
                color: Color::from(DARK_GRAY),
                ..default()
            },
            covered_tile_material: SpriteMaterial {
                color: Color::from(GRAY),
                ..default()
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

    fn text_input(
        parent: &mut RelatedSpawner<ChildOf>,
        font: Handle<Font>,
        value: impl Into<InputValue>,
    ) {
        let value: InputValue = value.into();

        parent
            .spawn((
                Name::new("Text Input"),
                Node {
                    width: px(150),
                    padding: px(6).all(),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: px(4).all(),
                    ..default()
                },
                BackgroundColor(Color::from(GRAY)),
                TextInput {
                    value: value.clone(),
                    ..default()
                },
                Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                    parent
                        .spawn(Self::ui_text(font.clone(), value))
                        .observe(on_click_text);
                })),
            ))
            .observe(on_lost_focus_handler)
            .observe(on_set_cursor_pos)
            .observe(on_back_original_input);
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

    fn ui_text(font: Handle<Font>, value: impl Into<String>) -> impl Bundle {
        (
            Text::new(value),
            TextFont {
                font: font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        )
    }

    fn label(font: Handle<Font>, value: impl Into<String> + Clone) -> impl Bundle {
        (
            Name::new(value.clone().into()),
            Label,
            Self::ui_text(font.clone(), value),
        )
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
    mut commands: Commands,
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
    } else {
        commands.trigger(ChangeInput {
            entity: event.0,
            value: input.value.clone(),
            label: None,
        });
    }
}

fn on_back_original_input(
    event: On<BackOriginalInput>,
    mut inputs: Query<(&mut TextInput, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    let (mut input, children) = inputs.get_mut(event.entity).unwrap();

    let text_entity = find_text_child_entity(children, &text_query);
    let mut text = text_query.get_mut(text_entity).unwrap();

    input.value = event.value.clone();
    text.0 = input.value.as_string();
    let chars: Vec<char> = text.0.chars().collect();
    input.cursor_pos = input.cursor_pos.min(chars.len());
}

fn on_click_text(
    click: On<Pointer<Click>>,
    text_query: Query<(&ChildOf, &Text, &ComputedTextBlock)>,
    mut commands: Commands,
) {
    let (parent, text, computed_text) = text_query.get(click.entity).unwrap();
    let buffer = computed_text.buffer();
    if let Some(position) = click.hit.position {
        if let (Some(x), Some(y)) = buffer.size() {
            let local_x = (position.x + 0.5) * x;
            let local_y = (position.y + 0.5) * y;

            if let Some(cursor) = buffer.hit(local_x, local_y) {
                let cursor_pos = text
                    .0
                    .char_indices()
                    .position(|(b, _)| b == cursor.index)
                    .unwrap_or_else(|| text.0.chars().count());

                log::info!(
                    "local {:?} cursor {:?} cursor_pos {}",
                    (local_x, local_y),
                    cursor,
                    cursor_pos
                );

                commands.trigger(SetCursorPosEvent {
                    entity: parent.parent(),
                    cursor_pos,
                });
            }
        }
    }
}

fn on_set_cursor_pos(
    event: On<SetCursorPosEvent>,
    mut inputs: Query<(&mut TextInput, &Children)>,
    mut text_query: Query<&mut Text>,
    mut timer: Query<&mut CursorTimer, With<SettingsUIRoot>>,
) {
    let (mut input, children) = inputs.get_mut(event.entity).unwrap();

    let mut timer = timer.single_mut().unwrap();
    timer.0.reset();

    let text_entity = find_text_child_entity(children, &text_query);
    let mut text = text_query.get_mut(text_entity).unwrap();

    let mut chars: Vec<char> = text.0.chars().collect();

    if input.is_cursor_inserted {
        chars.remove(input.cursor_pos);
    }

    let new_pos = event.cursor_pos.min(chars.len());
    chars.insert(new_pos, '|');
    input.cursor_pos = new_pos;
    input.is_cursor_inserted = true;

    text.0 = chars.into_iter().collect();
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
            _ => {}
        }
        Ok(())
    };

    if let Err(err) = res() {
        log::error!("{}", err);
        let value = match label.as_str() {
            "Width" => InputValue::from(board.map_size.0 as i32),
            "Height" => InputValue::from(board.map_size.1 as i32),
            _ => unreachable!(),
        };
        commands.trigger(BackOriginalInput {
            entity: change.original_event_target(),
            value,
        })
    }

    log::info!("Updated BoardOptions: {:?}", *board);
}

fn find_text_child_entity(children: &Children, text_query: &Query<&mut Text>) -> Entity {
    children
        .iter()
        .find(|child| text_query.get(*child).is_ok())
        .expect("TextInput must have a Text child")
}
