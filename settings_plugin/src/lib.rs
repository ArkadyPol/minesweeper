mod components;
pub mod events;
mod input_value;
pub mod resources;
mod ui;

use bevy::{
    color::palettes::css::{DARK_GRAY, GRAY},
    log,
    prelude::*,
};
use ron::ser::{PrettyConfig, to_string_pretty};
use std::fs;

use components::SettingsUIRoot;
use events::CreateGameEvent;
use resources::{BoardAssets, BoardOptions, SpriteMaterial};
use ui::{
    common::{change_background_color, in_focus_cursor, keyboard_handler, menu_action},
    root::create_menu,
};

pub struct SettingsPlugin<T> {
    pub running_state: T,
}

impl<T: States> Plugin for SettingsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(self.running_state.clone()),
            (Self::setup_board, create_menu).chain(),
        )
        .add_systems(
            Update,
            (
                (change_background_color, menu_action).chain(),
                keyboard_handler,
                in_focus_cursor,
            )
                .run_if(in_state(self.running_state.clone())),
        )
        .add_systems(OnExit(self.running_state.clone()), Self::cleanup_menu);
        app.add_message::<CreateGameEvent>();
    }
}

impl<T> SettingsPlugin<T> {
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
}
