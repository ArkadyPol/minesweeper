pub mod components;
pub mod events;
pub mod resources;
mod systems;
mod traits;

use bevy::{
    log,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    window::PrimaryWindow,
};
use rand::{rng, seq::SliceRandom};

use components::{
    Bomb, BombNeighbor, Center, Coordinates, EndMessage, LevelDown, LevelUp, NeighborOf, Neighbors,
    Neighbors2, TileCover, Uncover, VirtualCenter,
};
use events::{RestartGameEvent, TileMarkEvent};
use resources::{Board, BoardObservers};
use settings_plugin::resources::{BoardAssets, BoardOptions, BoardPosition, TileSize};
use systems::{
    end::{on_game_end, show_message, tick_count_down},
    input::input_handling,
    lose::uncover_tiles_on_lose,
    mark::mark_tiles,
    uncover::{on_uncover_handler, trigger_event_handler, uncover_tiles},
    win::uncover_bombs_on_win,
};

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
        .add_systems(OnEnter(self.not_pause.clone()), Self::init_observers)
        .add_systems(OnExit(self.not_pause.clone()), Self::cleanup_observers)
        // We handle uncovering even if the state is inactive
        .add_systems(
            Update,
            (uncover_tiles, show_message, tick_count_down)
                .run_if(in_state(self.running_state.clone())),
        )
        .add_systems(OnExit(self.running_state.clone()), Self::cleanup_board);
        app.add_message::<RestartGameEvent>();
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

        let virtual_centers = Self::assign_neighbors(&coords_map, &mut commands, options.map_size);

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
                            ..default()
                        },
                        Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                    )),
                    WithRelated::new(coords_map.into_values()),
                    WithRelated::new(virtual_centers),
                )),
            ))
            .id();

        let observers = vec![
            commands.add_observer(mark_tiles).id(),
            commands.add_observer(on_uncover_handler).id(),
            commands.add_observer(uncover_bombs_on_win).id(),
            commands.add_observer(uncover_tiles_on_lose).id(),
            commands.add_observer(on_game_end).id(),
        ];

        commands.insert_resource(Board {
            tile_size,
            entity: board_entity,
            observers,
            timer: None,
            end_message: "".into(),
        });
    }

    /// Places bombs and bomb neighbor tiles
    fn set_bombs(
        query: Query<(Entity, &Children), With<Coordinates>>,
        cover_query: Query<(), With<TileCover>>,
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        board_assets: Res<BoardAssets>,
        board: Res<Board>,
        query_neighbors_2: Query<(
            Option<&Neighbors2>,
            Option<&LevelDown>,
            &Coordinates,
            &Center,
            Has<VirtualCenter>,
        )>,
        query_neighbor_of: Query<(Option<&NeighborOf>, Option<&LevelUp>, &Coordinates)>,
    ) {
        let mut rng = rng();
        let options = match board_options {
            None => BoardOptions::default(), // If no options is set we use the default one
            Some(o) => o.clone(),
        };
        let bomb_count = options.bomb_count as usize;
        let padding = options.tile_padding;
        let size = board.tile_size;

        let mut entities: Vec<Entity> = query.iter().map(|(e, _)| e).collect();
        entities.shuffle(&mut rng);
        let mut bomb_entities = HashSet::new();

        for i in 0..bomb_count {
            if let Some(entity) = entities.get(i) {
                commands.entity(*entity).insert(Bomb).with_child((
                    Sprite {
                        color: board_assets.bomb_material.color,
                        image: board_assets.bomb_material.texture.clone(),
                        custom_size: Some(Vec2::splat(size - padding)),
                        ..default()
                    },
                    Transform::from_xyz(0., 0., 1.),
                ));
                bomb_entities.insert(*entity);
            }
        }

        let mut safe_start = None;

        for entity in entities.iter().skip(bomb_count).copied() {
            let count = find_neighbors(entity, &query_neighbors_2, &query_neighbor_of)
                .iter()
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
                let (_, children) = query.get(entity).unwrap();
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
                            ..default()
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
                                ..default()
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
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(0., 0., 1.),
        )
    }

    fn assign_neighbors(
        coords_map: &HashMap<Coordinates, Entity>,
        commands: &mut Commands,
        (width, height): (u16, u16),
    ) -> Vec<Entity> {
        for (&coords, &entity) in coords_map {
            let neighbors = SQUARE_COORDINATES
                .map(|tuple| coords + tuple)
                .map(|c| coords_map.get(&c).copied());
            commands.entity(entity).insert(Neighbors(neighbors));
        }

        let mut virtual_centers = Vec::new();

        let mut temp = coords_map.clone();
        let mut divisor: u16 = 3;

        while temp.len() > 1 {
            let mut new_map = HashMap::new();
            let level = divisor.ilog(3) as u8;

            for y in 0..height.div_ceil(divisor) {
                for x in 0..width.div_ceil(divisor) {
                    let center = Coordinates {
                        x: x * divisor + divisor / 2,
                        y: y * divisor + divisor / 2,
                    };

                    let mut center_entity = temp.get(&center).copied().unwrap_or_else(|| {
                        let entity = commands
                            .spawn((
                                Name::new(format!("Virtual Center ({}, {})", center.x, center.y)),
                                center,
                                VirtualCenter,
                            ))
                            .id();
                        virtual_centers.push(entity);
                        entity
                    });

                    if level > 1 {
                        let center_up = commands
                            .spawn((
                                Name::new(format!(
                                    "Level{} Center ({}, {})",
                                    level, center.x, center.y
                                )),
                                center,
                                VirtualCenter,
                            ))
                            .add_one_related::<LevelUp>(center_entity)
                            .id();
                        virtual_centers.push(center_up);
                        center_entity = center_up;
                    }

                    commands.entity(center_entity).insert(Center(level));

                    new_map.insert(center, center_entity);
                    let neighbors =
                        SQUARE_COORDINATES.map(|tuple| center + tuple * divisor as i32 / 3);

                    for coords in neighbors {
                        if let Some(&entity) = temp.get(&coords) {
                            commands.entity(entity).insert(NeighborOf(center_entity));
                        }
                    }
                }
            }

            temp = new_map;
            divisor *= 3;
        }

        virtual_centers
    }

    fn cleanup_board(
        board: Res<Board>,
        mut commands: Commands,
        end_message: Query<Entity, With<EndMessage>>,
    ) {
        commands.entity(board.entity).despawn();
        for &observer in &board.observers {
            commands.entity(observer).despawn();
        }
        commands.remove_resource::<Board>();

        if let Ok(end_message_entity) = end_message.single() {
            commands.entity(end_message_entity).despawn();
        }
    }

    fn init_observers(mut commands: Commands) {
        let input_observer = commands.add_observer(input_handling).id();
        let tile_trigger_observer = commands.add_observer(trigger_event_handler).id();
        commands.insert_resource(BoardObservers {
            input_observer,
            tile_trigger_observer,
        });
    }

    fn cleanup_observers(board_observers: Res<BoardObservers>, mut commands: Commands) {
        let BoardObservers {
            input_observer,
            tile_trigger_observer,
        } = *board_observers;
        commands.entity(input_observer).despawn();
        commands.entity(tile_trigger_observer).despawn();
        commands.remove_resource::<BoardObservers>();
    }

    fn check_neighbors(
        query_neighbors: Query<(Entity, &Neighbors)>,
        query_neighbors_2: Query<(
            Option<&Neighbors2>,
            Option<&LevelDown>,
            &Coordinates,
            &Center,
            Has<VirtualCenter>,
        )>,
        query_neighbor_of: Query<(Option<&NeighborOf>, Option<&LevelUp>, &Coordinates)>,
        query_coordinates: Query<&Coordinates>,
    ) {
        for (entity, neighbors) in query_neighbors {
            let neighbors: Vec<Entity> = neighbors.iter().flatten().copied().collect();
            let neighbors_2 = find_neighbors(entity, &query_neighbors_2, &query_neighbor_of);

            if neighbors != neighbors_2 {
                if let Ok(coords) = query_coordinates.get(entity) {
                    println!("--{}--", coords);
                }
                for i in 0..neighbors.len().max(neighbors_2.len()) {
                    let n_coords = neighbors
                        .get(i)
                        .and_then(|&e| query_coordinates.get(e).ok());
                    let n_coords_2 = neighbors_2
                        .get(i)
                        .and_then(|&e| query_coordinates.get(e).ok());
                    match (n_coords, n_coords_2) {
                        (Some(c1), Some(c2)) => println!("{} / {}", c1, c2),
                        (Some(c1), None) => println!("{} / None", c1),
                        (None, Some(c2)) => println!("None / {}", c2),
                        (None, None) => println!("None / None"),
                    }
                }
            }
        }
    }
}

pub fn find_neighbors(
    entity: Entity,
    query_neighbors: &Query<(
        Option<&Neighbors2>,
        Option<&LevelDown>,
        &Coordinates,
        &Center,
        Has<VirtualCenter>,
    )>,
    query_neighbor_of: &Query<(Option<&NeighborOf>, Option<&LevelUp>, &Coordinates)>,
) -> Vec<Entity> {
    if let Ok((Some(neighbors), _, _, _, _)) = query_neighbors.get(entity) {
        return neighbors.iter().collect();
    }

    if let Ok((Some(neighbor_of), _, coords)) = query_neighbor_of.get(entity) {
        let mut entities = Vec::new();
        let neighbors = SQUARE_COORDINATES.map(|tuple| *coords + tuple);

        let center_entity = neighbor_of.0;

        for neighbor in neighbors {
            if let Some(neighbor_entity) = find_coordinate(
                neighbor,
                center_entity,
                None,
                query_neighbors,
                query_neighbor_of,
            ) {
                entities.push(neighbor_entity);
            }
        }

        return entities;
    }

    vec![]
}

fn find_coordinate(
    coords: Coordinates,
    center_entity: Entity,
    previous_entity: Option<Entity>,
    query_neighbors: &Query<(
        Option<&Neighbors2>,
        Option<&LevelDown>,
        &Coordinates,
        &Center,
        Has<VirtualCenter>,
    )>,
    query_neighbor_of: &Query<(Option<&NeighborOf>, Option<&LevelUp>, &Coordinates)>,
) -> Option<Entity> {
    if let Ok((_, _, &center_coords, _, is_virtual)) = query_neighbors.get(center_entity) {
        if !is_virtual && coords == center_coords {
            return Some(center_entity);
        }
    }

    if let Ok((Some(neighbors), _, _, _, _)) = query_neighbors.get(center_entity) {
        for neighbor_entity in neighbors.iter() {
            if Some(neighbor_entity) == previous_entity {
                continue;
            }

            if let Ok((_, _, &n_coords)) = query_neighbor_of.get(neighbor_entity) {
                if coords == n_coords {
                    return Some(neighbor_entity);
                }
            }

            if let Ok((_, _, &n_coords, level, _)) = query_neighbors.get(neighbor_entity) {
                let bounds = IRect::from_center_size(n_coords.into(), level.get_size());
                if bounds.contains(coords.into()) {
                    return find_coordinate(
                        coords,
                        neighbor_entity,
                        Some(center_entity),
                        query_neighbors,
                        query_neighbor_of,
                    );
                }
            }
        }
    }

    if let Ok((neighbor_of, level_up, _)) = query_neighbor_of.get(center_entity) {
        let Some(parent_entity) = neighbor_of.map(|n| n.0).or_else(|| level_up.map(|l| l.0)) else {
            return None;
        };
        if let Ok((Some(neighbors), _, _, _, _)) = query_neighbors.get(parent_entity) {
            for neighbor_entity in neighbors.iter() {
                if neighbor_entity == center_entity {
                    continue;
                }

                if let Ok((_, _, &n_coords, level, _)) = query_neighbors.get(neighbor_entity) {
                    let bounds = IRect::from_center_size(n_coords.into(), level.get_size());
                    if bounds.contains(coords.into()) {
                        return find_coordinate(
                            coords,
                            neighbor_entity,
                            Some(center_entity),
                            query_neighbors,
                            query_neighbor_of,
                        );
                    }
                }
            }
        }

        if let Ok((_, Some(level_down), _, _, _)) = query_neighbors.get(parent_entity) {
            let child_entity = level_down.entity();
            if child_entity == center_entity {
                return None;
            }

            if let Ok((_, _, &n_coords, level, _)) = query_neighbors.get(child_entity) {
                let bounds = IRect::from_center_size(n_coords.into(), level.get_size());
                if bounds.contains(coords.into()) {
                    return find_coordinate(
                        coords,
                        child_entity,
                        Some(center_entity),
                        query_neighbors,
                        query_neighbor_of,
                    );
                }
            }
        }

        if Some(parent_entity) == previous_entity {
            return None;
        }

        return find_coordinate(
            coords,
            parent_entity,
            Some(center_entity),
            query_neighbors,
            query_neighbor_of,
        );
    }

    None
}

/// Delta coordinates for all 8 square neighbors
const SQUARE_COORDINATES: [IVec2; 8] = [
    // Bottom left
    IVec2::new(-1, -1),
    // Bottom
    IVec2::new(0, -1),
    // Bottom right
    IVec2::new(1, -1),
    // Left
    IVec2::new(-1, 0),
    // Right
    IVec2::new(1, 0),
    // Top Left
    IVec2::new(-1, 1),
    // Top
    IVec2::new(0, 1),
    // Top right
    IVec2::new(1, 1),
];
