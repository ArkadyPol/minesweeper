use bevy::color::palettes::css::{DARK_GRAY, GRAY};
use bevy::{log, prelude::*};
#[cfg(feature = "board_v1")]
use board_plugin::resources::{BoardAssets, BoardOptions, SpriteMaterial};
#[cfg(feature = "board_v2")]
use board_plugin_v2::resources::{BoardAssets, BoardOptions, SpriteMaterial};
use events::CreateGameEvent;

mod events;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    InGame {
        paused: bool,
    },
    #[default]
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
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Mine Sweeper!".to_string(),
            resolution: (700, 800).into(),
            ..Default::default()
        }),
        ..Default::default()
    }));
    app.insert_resource(SpritePickingSettings {
        picking_mode: SpritePickingMode::BoundingBox,
        ..Default::default()
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

    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    {
        use bevy_inspector_egui::bevy_egui::EguiPlugin;
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
    }
    // Startup system (cameras) & board
    app.add_systems(Startup, (camera_setup, setup_board));
    // State handling
    app.add_systems(Update, (handle_create_game_event, state_handler).chain());

    app.add_message::<CreateGameEvent>();
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
            next_state.set(AppState::Out);
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

fn handle_create_game_event(
    mut reader: MessageReader<CreateGameEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _ev in reader.read() {
        log::info!("loading game from event");
        next_state.set(AppState::start_game());
    }
}

fn setup_board(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
) {
    // Board plugin options
    commands.insert_resource(BoardOptions {
        map_size: (20, 20),
        bomb_count: 40,
        tile_padding: 1.,
        safe_start: true,
        ..Default::default()
    });
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
    // Plugin activation
    next_state.set(AppState::start_game());
}
