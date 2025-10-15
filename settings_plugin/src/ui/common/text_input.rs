use bevy::{
    color::palettes::css::GRAY, input::keyboard::Key, log, prelude::*, text::ComputedTextBlock,
    ui_widgets::observe,
};

use crate::{
    components::{CursorTimer, SettingsUIRoot, TextInput},
    events::{BackOriginalInput, ChangeInput, LostFocusEvent, SetCursorPosEvent},
    input_value::InputValue,
};

use super::text;

pub fn text_input(value: impl Into<InputValue>) -> impl Bundle {
    let value: InputValue = value.into();

    (
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
        children![(text(24.0, value), observe(on_click_text))],
        observe(on_lost_focus_handler),
        observe(on_set_cursor_pos),
        observe(on_back_original_input),
    )
}

pub fn keyboard_handler(
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

pub fn in_focus_cursor(
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

fn find_text_child_entity(children: &Children, text_query: &Query<&mut Text>) -> Entity {
    children
        .iter()
        .find(|child| text_query.get(*child).is_ok())
        .expect("TextInput must have a Text child")
}
