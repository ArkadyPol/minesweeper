use bevy::{log, prelude::*};
use board_plugin::BoardPlugin;
use board_plugin::resources::BoardOptions;

use events::CreateGameEvent;

mod events;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States)]
pub enum AppState {
    InGame { paused: bool },
    Out,
}

impl Default for AppState {
    fn default() -> Self {
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
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Mine Sweeper!".to_string(),
            resolution: (700., 800.).into(),
            ..Default::default()
        }),
        ..Default::default()
    }));
    // Board plugin options
    app.insert_resource(BoardOptions {
        map_size: (20, 20),
        bomb_count: 40,
        tile_padding: 3.0,
        safe_start: true,
        ..Default::default()
    });
    app.init_state::<AppState>();
    app.add_computed_state::<InGame>();
    app.add_plugins(BoardPlugin {
        running_state: InGame,
        not_pause: AppState::default(),
    });
    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    {
        use bevy_inspector_egui::bevy_egui::EguiPlugin;
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
    }
    // Startup system (cameras)
    app.add_systems(Startup, camera_setup);
    // State handling
    app.add_systems(Update, (handle_create_game_event, state_handler).chain());

    app.add_event::<CreateGameEvent>();
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
    mut events: EventWriter<CreateGameEvent>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        log::debug!("clearing detected");
        if let AppState::InGame { .. } = state.get() {
            log::info!("clearing game");
            next_state.set(AppState::Out);
        }
    }
    if keys.just_pressed(KeyCode::KeyG) {
        log::debug!("loading detected");
        if state.get() == &AppState::Out {
            log::info!("loading game");
            next_state.set(AppState::default());
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

fn handle_create_game_event(
    mut reader: EventReader<CreateGameEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _ev in reader.read() {
        log::info!("loading game from event");
        next_state.set(AppState::default());
    }
}
