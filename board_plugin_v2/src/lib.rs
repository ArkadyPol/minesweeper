pub mod components;
pub mod resources;

use bevy::{log, platform::collections::HashMap, prelude::*, window::PrimaryWindow};

use resources::{BoardAssets, BoardOptions, BoardPosition, TileSize};

use components::{Coordinates, Neighbors};

pub struct BoardPluginV2<T, U> {
    pub running_state: T,
    pub not_pause: U,
}

impl<T: ComputedStates, U: States> Plugin for BoardPluginV2<T, U> {
    fn build(&self, app: &mut App) {
        // When the running states comes into the stack we load a board
        app.add_systems(OnEnter(self.running_state.clone()), Self::create_board);
        log::info!("Loaded Board Plugin");
    }
}

impl<T, U> BoardPluginV2<T, U> {
    /// System to generate the complete board
    pub fn create_board(
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        window: Query<&Window, With<PrimaryWindow>>,
        board_assets: Res<BoardAssets>,
    ) {
        let board_assets = board_assets.clone();
        let options = match board_options {
            None => BoardOptions::default(), // If no options is set we use the default one
            Some(o) => o.clone(),
        };

        let (width, height) = options.map_size;

        let tile_size = match options.tile_size {
            TileSize::Fixed(v) => v,
            TileSize::Adaptive { min, max } => {
                Self::adaptative_tile_size(window, (min, max), (width, height))
            }
        };

        // We deduce the size of the complete board
        let board_size = Vec2::new(width as f32 * tile_size, height as f32 * tile_size);
        log::info!("board size: {}", board_size);
        // We define the board anchor position (bottom left)
        let board_position = match options.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
            }
            BoardPosition::Custom(p) => p,
        };

        let mut coords_map = HashMap::new();

        Self::spawn_tiles(
            &mut commands,
            options.map_size,
            tile_size,
            options.tile_padding,
            &board_assets,
            &mut coords_map,
        );

        Self::assign_neighbors(&coords_map, &mut commands);

        commands.spawn((
            Name::new("Board"),
            Transform::from_translation(board_position),
            Visibility::default(),
            Children::spawn((
                Spawn((
                    Name::new("Background"),
                    Sprite {
                        color: board_assets.board_material.color,
                        custom_size: Some(board_size),
                        image: board_assets.board_material.texture.clone(),
                        ..Default::default()
                    },
                    Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                )),
                WithRelated::new(coords_map.into_values()),
            )),
        ));
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

    fn spawn_tiles(
        commands: &mut Commands,
        (width, height): (u16, u16),
        size: f32,
        padding: f32,
        board_assets: &BoardAssets,
        coords_map: &mut HashMap<Coordinates, Entity>,
    ) {
        for y in 0..height {
            for x in 0..width {
                let coordinates = Coordinates { x, y };
                let entity = commands
                    .spawn((
                        Name::new(format!("Tile ({}, {})", x, y)),
                        Sprite {
                            color: board_assets.tile_material.color,
                            custom_size: Some(Vec2::splat(size - padding)),
                            image: board_assets.tile_material.texture.clone(),
                            ..Default::default()
                        },
                        Transform::from_xyz(
                            (x as f32 * size) + (size / 2.),
                            (y as f32 * size) + (size / 2.),
                            1.,
                        ),
                        coordinates,
                    ))
                    .id();

                coords_map.insert(coordinates, entity);
            }
        }
    }

    fn assign_neighbors(coords_map: &HashMap<Coordinates, Entity>, commands: &mut Commands) {
        /// Delta coordinates for all 8 square neighbors
        const SQUARE_COORDINATES: [(i8, i8); 8] = [
            // Bottom left
            (-1, -1),
            // Bottom
            (0, -1),
            // Bottom right
            (1, -1),
            // Left
            (-1, 0),
            // Right
            (1, 0),
            // Top Left
            (-1, 1),
            // Top
            (0, 1),
            // Top right
            (1, 1),
        ];

        for (&coords, &entity) in coords_map {
            let neighbors = SQUARE_COORDINATES
                .map(|tuple| coords + tuple)
                .map(|c| coords_map.get(&c).copied());
            commands.entity(entity).insert(Neighbors { neighbors });
        }
    }
}
