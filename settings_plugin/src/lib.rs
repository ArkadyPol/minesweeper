pub mod events;
pub mod resources;

use bevy::{
    color::palettes::css::{DARK_GRAY, GRAY},
    prelude::*,
};

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

        create_game.write(CreateGameEvent);
    }
}
