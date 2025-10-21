use bevy::{log, prelude::*, ui_widgets::UiWidgetsPlugins};
use board_plugin_v2::events::RestartGameEvent;
use main_menu_plugin::{MainMenuPlugin, events::LoadSettingsEvent};
use settings_plugin::{
    SettingsPlugin,
    events::{BackToMenuEvent, CreateGameEvent},
};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    MainMenu,
    Settings,
    InGame {
        paused: bool,
    },
    Out,
}

impl AppState {
    fn start_game() -> Self {
        Self::InGame { paused: false }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct InGame;

impl ComputedStates for InGame {
    type SourceStates = AppState;

    fn compute(sources: AppState) -> Option<Self> {
        match sources {
            AppState::InGame { .. } => Some(InGame),
            _ => None,
        }
    }
}

fn main() {
    let mut app = App::new();
    // Bevy default plugins with window setup
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mine Sweeper!".to_string(),
                resolution: (700, 800).into(),
                ..default()
            }),
            ..default()
        }),
        UiWidgetsPlugins,
    ));
    app.insert_resource(SpritePickingSettings {
        picking_mode: SpritePickingMode::BoundingBox,
        ..default()
    });
    app.init_state::<AppState>();
    app.add_computed_state::<InGame>();
    #[cfg(feature = "board_v1")]
    {
        use board_plugin::BoardPlugin;

        app.add_plugins(BoardPlugin {
            running_state: InGame,
            not_pause: AppState::start_game(),
        });
    }

    #[cfg(feature = "board_v2")]
    {
        use board_plugin_v2::BoardPluginV2;

        app.add_plugins(BoardPluginV2 {
            running_state: InGame,
            not_pause: AppState::start_game(),
        });
    }

    app.add_plugins((
        MainMenuPlugin {
            running_state: AppState::MainMenu,
        },
        SettingsPlugin {
            running_state: AppState::Settings,
        },
    ));

    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    {
        use bevy_inspector_egui::bevy_egui::EguiPlugin;
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
    }
    // Startup system (cameras) & board
    app.add_systems(Startup, camera_setup);
    // State handling
    app.add_systems(
        Update,
        (
            handle_state_game_events,
            handle_restart_game_event,
            state_handler,
        )
            .chain(),
    );

    // Run the app
    app.run();
}

fn camera_setup(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn(Camera2d);
}

fn state_handler(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut events: MessageWriter<CreateGameEvent>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        log::debug!("clearing detected");
        if let AppState::InGame { .. } = state.get() {
            log::info!("clearing game");
            next_state.set(AppState::MainMenu);
        }
    }
    if keys.just_pressed(KeyCode::KeyG) {
        log::debug!("loading detected");
        if state.get() == &AppState::Out {
            log::info!("loading game");
            next_state.set(AppState::start_game());
        }
        if let AppState::InGame { .. } = state.get() {
            log::info!("reloading game");
            next_state.set(AppState::Out);
            events.write(CreateGameEvent);
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        log::debug!("pause toggle detected");
        if let AppState::InGame { paused } = state.get() {
            if *paused {
                log::info!("resuming game");
            } else {
                log::info!("pausing game");
            }

            next_state.set(AppState::InGame { paused: !paused });
        }
    }
}

fn handle_state_game_events(
    mut create_game_reader: MessageReader<CreateGameEvent>,
    mut load_settings_reader: MessageReader<LoadSettingsEvent>,
    mut back_to_menu_reader: MessageReader<BackToMenuEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _ev in create_game_reader.read() {
        log::info!("loading game from event");
        next_state.set(AppState::start_game());
    }
    for _ev in load_settings_reader.read() {
        log::info!("loading settings from event");
        next_state.set(AppState::Settings);
    }
    for _ev in back_to_menu_reader.read() {
        log::info!("back to menu");
        next_state.set(AppState::MainMenu);
    }
}

fn handle_restart_game_event(
    mut create_game_writer: MessageWriter<CreateGameEvent>,
    mut restart_game_reader: MessageReader<RestartGameEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _ev in restart_game_reader.read() {
        log::info!("restart game");
        next_state.set(AppState::Out);
        create_game_writer.write(CreateGameEvent);
    }
}
