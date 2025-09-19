mod bounds;
pub mod components;
pub mod resources;
mod systems;

use bevy::{
    color::palettes::css::{GRAY, GREEN, ORANGE, PURPLE, YELLOW},
    ecs::relationship::RelatedSpawnerCommands,
    log,
    prelude::*,
    window::PrimaryWindow,
};
use bounds::Bounds2;
use components::{Bomb, BombNeighbor, Coordinates, Uncover};
use resources::{Board, BoardOptions, BoardPosition, TileSize, tile::Tile, tile_map::TileMap};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::create_board);
        app.add_systems(Update, systems::input::input_handling);
        log::info!("Loaded Board Plugin");
        #[cfg(feature = "debug")]
        {
            // registering custom component to be able to edit it in inspector
            app.register_type::<Coordinates>();
            app.register_type::<BombNeighbor>();
            app.register_type::<Bomb>();
            app.register_type::<Uncover>();
        }
    }
}

impl BoardPlugin {
    /// System to generate the complete board
    pub fn create_board(
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        window: Query<&Window, With<PrimaryWindow>>,
        asset_server: Res<AssetServer>,
    ) {
        let font: Handle<Font> = asset_server.load("fonts/pixeled.ttf");
        let bomb_image: Handle<Image> = asset_server.load("sprites/bomb.png");

        let options = match board_options {
            None => BoardOptions::default(), // If no options is set we use the default one
            Some(o) => o.clone(),
        };
        // Tilemap generation
        let mut tile_map = TileMap::empty(options.map_size.0, options.map_size.1);
        tile_map.set_bombs(options.bomb_count);
        #[cfg(feature = "debug")]
        // Tilemap debugging
        log::info!("{}", tile_map.console_output());

        let tile_size = match options.tile_size {
            TileSize::Fixed(v) => v,
            TileSize::Adaptive { min, max } => Self::adaptative_tile_size(
                window,
                (min, max),
                (tile_map.width(), tile_map.height()),
            ),
        };

        // We deduce the size of the complete board
        let board_size = Vec2::new(
            tile_map.width() as f32 * tile_size,
            tile_map.height() as f32 * tile_size,
        );
        log::info!("board size: {}", board_size);
        // We define the board anchor position (bottom left)
        let board_position = match options.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
            }
            BoardPosition::Custom(p) => p,
        };

        commands
            .spawn((
                Name::new("Board"),
                Transform::from_translation(board_position),
                Visibility::default(),
            ))
            .with_children(|parent| {
                // We spawn the board background sprite at the center of the board, since the sprite pivot is centered
                parent.spawn((
                    Name::new("Background"),
                    Sprite {
                        color: Color::WHITE,
                        custom_size: Some(board_size),
                        ..Default::default()
                    },
                    Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                ));

                Self::spawn_tiles(
                    parent,
                    &tile_map,
                    tile_size,
                    options.tile_padding,
                    Color::from(GRAY),
                    bomb_image,
                    font,
                );
            });

        commands.insert_resource(Board {
            tile_map,
            bounds: Bounds2 {
                position: board_position.xy(),
                size: board_size,
            },
            tile_size,
        });
    }

    /// Computes a tile size that matches the window according to the tile map size
    fn adaptative_tile_size(
        window: Query<&Window, With<PrimaryWindow>>,
        (min, max): (f32, f32),      // Tile size constraints
        (width, height): (u16, u16), // Tile map dimensions
    ) -> f32 {
        let window = window.single().unwrap();
        let max_width = window.width() / width as f32;
        let max_heigth = window.height() / height as f32;
        max_width.min(max_heigth).clamp(min, max)
    }

    /// Generates the bomb counter text 2D Bundle for a given value
    fn bomb_count_text_bundle(count: u8, font: Handle<Font>, size: f32) -> impl Bundle {
        // We retrieve the text and the correct color
        let (text, color) = (
            count.to_string(),
            match count {
                1 => Color::WHITE,
                2 => Color::from(GREEN),
                3 => Color::from(YELLOW),
                4 => Color::from(ORANGE),
                _ => Color::from(PURPLE),
            },
        );
        // We generate a text bundle
        (
            Text2d::new(text),
            TextFont {
                font,
                font_size: size,
                ..Default::default()
            },
            TextColor(color),
            Transform::from_xyz(0., 0., 1.),
        )
    }

    fn spawn_tiles(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        tile_map: &TileMap,
        size: f32,
        padding: f32,
        color: Color,
        bomb_image: Handle<Image>,
        font: Handle<Font>,
    ) {
        // Tiles
        for (y, line) in tile_map.iter().enumerate() {
            for (x, tile) in line.iter().enumerate() {
                let mut cmd = parent.spawn((
                    Name::new(format!("Tile ({}, {})", x, y)),
                    Sprite {
                        color,
                        custom_size: Some(Vec2::splat(size - padding as f32)),
                        ..Default::default()
                    },
                    Transform::from_xyz(
                        (x as f32 * size) + (size / 2.),
                        (y as f32 * size) + (size / 2.),
                        1.,
                    ),
                    Coordinates {
                        x: x as u16,
                        y: y as u16,
                    },
                ));

                match tile {
                    // If the tile is a bomb we add the matching component and a sprite child
                    Tile::Bomb => {
                        cmd.insert(Bomb);
                        cmd.with_children(|parent| {
                            parent.spawn((
                                Sprite {
                                    image: bomb_image.clone(),
                                    custom_size: Some(Vec2::splat(size - padding)),
                                    ..Default::default()
                                },
                                Transform::from_xyz(0., 0., 1.),
                            ));
                        });
                    }
                    // If the tile is a bomb neighbour we add the matching component and a text child
                    Tile::BombNeighbor(v) => {
                        cmd.insert(BombNeighbor { count: *v });
                        cmd.with_children(|parent| {
                            parent.spawn(Self::bomb_count_text_bundle(
                                *v,
                                font.clone(),
                                (size - padding) * 0.5,
                            ));
                        });
                    }
                    Tile::Empty => (),
                }
            }
        }
    }
}
