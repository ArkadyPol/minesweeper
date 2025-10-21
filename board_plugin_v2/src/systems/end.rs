use bevy::{color::palettes::css::GRAY, log, prelude::*, ui_widgets::observe};

use crate::{
    components::{CountdownTimer, EndMessage},
    events::{CountdownEvent, GameEndEvent, RestartGameEvent},
    resources::Board,
};

pub fn on_game_end(event: On<GameEndEvent>, mut board: ResMut<Board>) {
    log::info!("{}", event.message);
    board.timer = Some(Timer::from_seconds(2.0, TimerMode::Once));
    board.end_message = event.message.clone();
}

pub fn show_message(
    mut commands: Commands,
    mut board: ResMut<Board>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    children: Query<&Children>,
) {
    let Some(timer) = &mut board.timer else {
        return;
    };

    timer.tick(time.delta());

    if !timer.just_finished() {
        return;
    }

    board.timer = None;

    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    let remaining = 20;

    let entity = commands
        .spawn((
            Name::new("End message"),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            EndMessage,
            children![(
                Node {
                    width: px(500),
                    padding: px(12).all(),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    column_gap: px(6),
                    ..default()
                },
                BackgroundColor(Color::from(GRAY)),
                children![
                    (
                        Text::new(format!("{}", board.end_message)),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ),
                    (
                        Text::new(format!("Game restarts in {} seconds", remaining)),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        observe(on_count_down_text)
                    ),
                    (
                        Text::new("Press the key C to exit to the main menu"),
                        TextFont {
                            font,
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    )
                ]
            )],
        ))
        .observe(on_count_down)
        .id();

    commands.trigger(CountdownEvent::new(entity, remaining, &children));
}

fn on_count_down(
    event: On<CountdownEvent>,
    mut commands: Commands,
    mut restart_game_writer: MessageWriter<RestartGameEvent>,
) {
    log::info!("restart in {} seconds", event.remaining);

    if event.remaining > 0 {
        commands.entity(event.entity).insert(CountdownTimer {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            remaining: event.remaining - 1,
        });
    } else {
        commands.entity(event.entity).remove::<CountdownTimer>();
        restart_game_writer.write(RestartGameEvent);
    }
}

fn on_count_down_text(event: On<CountdownEvent>, mut query: Query<&mut Text>) {
    let mut text = query.get_mut(event.entity).unwrap();
    text.0 = format!("Game restarts in {} seconds", event.remaining);
}

pub fn tick_count_down(
    mut commands: Commands,
    mut countdown_timer: Query<(Entity, &mut CountdownTimer)>,
    children: Query<&Children>,
    time: Res<Time>,
) {
    if let Ok((entity, mut countdown_timer)) = countdown_timer.single_mut() {
        countdown_timer.timer.tick(time.delta());
        if countdown_timer.timer.just_finished() {
            commands.trigger(CountdownEvent::new(
                entity,
                countdown_timer.remaining,
                &children,
            ));
        }
    }
}
