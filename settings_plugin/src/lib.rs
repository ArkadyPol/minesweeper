pub mod events;
pub mod resources;

use bevy::{
    color::palettes::css::{DARK_GRAY, GRAY},
    log,
    prelude::*,
};
use ron::ser::{PrettyConfig, to_string_pretty};
use std::fs;

use events::CreateGameEvent;
use resources::{BoardAssets, BoardOptions, SpriteMaterial};

pub struct SettingsPlugin<T> {
    pub running_state: T,
}

impl<T: States> Plugin for SettingsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(self.running_state.clone()), Self::setup_board);
        app.add_message::<CreateGameEvent>();
    }
}

impl<T> SettingsPlugin<T> {
    fn setup_board(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut create_game: MessageWriter<CreateGameEvent>,
    ) {
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

        fs::write(
            "board_options.ron",
            to_string_pretty(&board_options, PrettyConfig::default()).unwrap(),
        )
        .expect("Error saving settings");

        create_game.write(CreateGameEvent);
    }
}
