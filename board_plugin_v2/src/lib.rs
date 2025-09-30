pub mod components;
mod events;
pub mod resources;
mod systems;

use bevy::{
    log,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    window::PrimaryWindow,
};
use rand::{rng, seq::SliceRandom};

use components::{Bomb, BombNeighbor, Coordinates, Neighbors, TileCover, Uncover};
use events::{BoardCompletedEvent, BombExplosionEvent, TileMarkEvent, TileTriggerEvent};
use resources::Board;
use settings_plugin::resources::{BoardAssets, BoardOptions, BoardPosition, TileSize};

pub struct BoardPluginV2<T, U> {
    pub running_state: T,
    pub not_pause: U,
}

impl<T: ComputedStates, U: States> Plugin for BoardPluginV2<T, U> {
    fn build(&self, app: &mut App) {
        // When the running states comes into the stack we load a board
        app.add_systems(
            OnEnter(self.running_state.clone()),
            (Self::create_board, Self::set_bombs).chain(),
        )
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
            (systems::uncover::uncover_tiles, systems::mark::mark_tiles)
                .run_if(in_state(self.running_state.clone())),
        )
        .add_systems(OnExit(self.running_state.clone()), Self::cleanup_board);
        app.add_message::<TileTriggerEvent>();
        app.add_message::<BoardCompletedEvent>();
        app.add_message::<BombExplosionEvent>();
        app.add_message::<TileMarkEvent>();
        log::info!("Loaded Board Plugin");
        #[cfg(feature = "debug")]
        {
            // registering custom component to be able to edit it in inspector
            app.register_type::<Coordinates>();
            app.register_type::<BombNeighbor>();
            app.register_type::<Bomb>();
            app.register_type::<Uncover>();
            app.register_type::<Neighbors>();
            app.register_type::<TileCover>();
        }
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

        let board_entity = commands
            .spawn((
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
            ))
            .id();

        commands.insert_resource(Board {
            tile_size,
            entity: board_entity,
        });
    }

    /// Places bombs and bomb neighbor tiles
    fn set_bombs(
        query: Query<(Entity, &Neighbors, &Children), With<Coordinates>>,
        cover_query: Query<(), With<TileCover>>,
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        board_assets: Res<BoardAssets>,
        board: Res<Board>,
    ) {
        let mut rng = rng();
        let options = match board_options {
            None => BoardOptions::default(), // If no options is set we use the default one
            Some(o) => o.clone(),
        };
        let bomb_count = options.bomb_count as usize;
        let padding = options.tile_padding;
        let size = board.tile_size;

        let mut entities: Vec<(Entity, &Neighbors)> =
            query.iter().map(|(e, n, _)| (e, n)).collect();
        entities.shuffle(&mut rng);
        let mut bomb_entities = HashSet::new();

        for i in 0..bomb_count {
            if let Some((entity, _)) = entities.get(i) {
                commands.entity(*entity).insert(Bomb).with_child((
                    Sprite {
                        color: board_assets.bomb_material.color,
                        image: board_assets.bomb_material.texture.clone(),
                        custom_size: Some(Vec2::splat(size - padding)),
                        ..Default::default()
                    },
                    Transform::from_xyz(0., 0., 1.),
                ));
                bomb_entities.insert(*entity);
            }
        }

        let mut safe_start = None;

        for (entity, neighbors) in entities.iter().skip(bomb_count).copied() {
            let count = neighbors
                .neighbors
                .iter()
                .flatten()
                .filter(|&e| bomb_entities.contains(e))
                .count() as u8;

            if count > 0 {
                commands
                    .entity(entity)
                    .insert(BombNeighbor { count })
                    .with_child(Self::bomb_count_text_bundle(
                        count,
                        &board_assets,
                        (size - padding) * 0.5,
                    ));
            } else if safe_start.is_none() {
                safe_start = Some(entity);
            }
        }
        if options.safe_start {
            if let Some(entity) = safe_start {
                let (_, _, children) = query.get(entity).unwrap();
                for &child in children {
                    if cover_query.get(child).is_ok() {
                        commands.entity(child).insert(Uncover);
                    }
                }
            }
        }
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
                        // We add the cover sprites
                        children![(
                            Name::new("Tile Cover"),
                            Sprite {
                                custom_size: Some(Vec2::splat(size - padding)),
                                color: board_assets.covered_tile_material.color,
                                image: board_assets.covered_tile_material.texture.clone(),
                                ..Default::default()
                            },
                            Transform::from_xyz(0., 0., 2.),
                            Pickable::default(),
                            TileCover,
                        )],
                    ))
                    .id();

                coords_map.insert(coordinates, entity);
            }
        }
    }

    /// Generates the bomb counter text 2D Bundle for a given value
    fn bomb_count_text_bundle(count: u8, board_assets: &BoardAssets, size: f32) -> impl Bundle {
        // We retrieve the text and the correct color
        let color = board_assets.bomb_counter_color(count);
        // We generate a text bundle
        (
            Text2d::new(count.to_string()),
            TextFont {
                font: board_assets.bomb_counter_font.clone(),
                font_size: size,
                ..Default::default()
            },
            TextColor(color),
            Transform::from_xyz(0., 0., 1.),
        )
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

    fn cleanup_board(board: Res<Board>, mut commands: Commands) {
        commands.entity(board.entity).despawn();
        commands.remove_resource::<Board>();
    }
}
