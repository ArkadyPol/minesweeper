mod bounds;
pub mod components;
mod events;
pub mod resources;
mod systems;

use bevy::{
    color::palettes::css::{DARK_GRAY, GRAY, GREEN, ORANGE, PURPLE, YELLOW},
    ecs::relationship::RelatedSpawnerCommands,
    log,
    platform::collections::HashMap,
    prelude::*,
    window::PrimaryWindow,
};
use bounds::Bounds2;
use components::{Bomb, BombNeighbor, Coordinates, Uncover};
use events::TileTriggerEvent;
use resources::{Board, BoardOptions, BoardPosition, TileSize, tile::Tile, tile_map::TileMap};

pub struct BoardPlugin<T, U> {
    pub running_state: T,
    pub not_pause: U,
}

impl<T: ComputedStates, U: States> Plugin for BoardPlugin<T, U> {
    fn build(&self, app: &mut App) {
        // When the running states comes into the stack we load a board
        app.add_systems(OnEnter(self.running_state.clone()), Self::create_board)
            // We handle input and trigger events only if the state is active
            .add_systems(
                Update,
                (
                    systems::input::input_handling,
                    systems::uncover::trigger_event_handler,
                )
                    .run_if(in_state(self.not_pause.clone())),
            )
            // We handle uncovering even if the state is inactive
            .add_systems(
                Update,
                systems::uncover::uncover_tiles.run_if(in_state(self.running_state.clone())),
            )
            .add_systems(OnExit(self.running_state.clone()), Self::cleanup_board);
        app.add_event::<TileTriggerEvent>();
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

impl<T, U> BoardPlugin<T, U> {
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

        let mut covered_tiles =
            HashMap::with_capacity((tile_map.width() * tile_map.height()).into());
        let mut safe_start = None;

        let board_entity = commands
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
                    Color::from(DARK_GRAY),
                    &mut covered_tiles,
                    &mut safe_start,
                );
            })
            .id();

        if options.safe_start {
            if let Some(entity) = safe_start {
                commands.entity(entity).insert(Uncover);
            }
        }

        commands.insert_resource(Board {
            tile_map,
            bounds: Bounds2 {
                position: board_position.xy(),
                size: board_size,
            },
            tile_size,
            covered_tiles,
            entity: board_entity,
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
        covered_tile_color: Color,
        covered_tiles: &mut HashMap<Coordinates, Entity>,
        safe_start_entity: &mut Option<Entity>,
    ) {
        // Tiles
        for (y, line) in tile_map.iter().enumerate() {
            for (x, tile) in line.iter().enumerate() {
                let coordinates = Coordinates {
                    x: x as u16,
                    y: y as u16,
                };

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
                    coordinates,
                ));

                // We add the cover sprites
                cmd.with_children(|parent| {
                    let entity = parent
                        .spawn((
                            Name::new("Tile Cover"),
                            Sprite {
                                custom_size: Some(Vec2::splat(size - padding)),
                                color: covered_tile_color,
                                ..Default::default()
                            },
                            Transform::from_xyz(0., 0., 2.),
                        ))
                        .id();
                    covered_tiles.insert(coordinates, entity);
                    if safe_start_entity.is_none() && *tile == Tile::Empty {
                        *safe_start_entity = Some(entity);
                    }
                });

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

    fn cleanup_board(board: Res<Board>, mut commands: Commands) {
        commands.entity(board.entity).despawn();
        commands.remove_resource::<Board>();
    }
}
