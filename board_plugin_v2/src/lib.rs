pub mod components;
pub mod events;
pub mod resources;
mod systems;
mod traits;

use std::time::Instant;

use bevy::{
    log,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    window::PrimaryWindow,
};
use rand::{rng, seq::SliceRandom};
#[cfg(feature = "hierarchical_neighbors")]
use smallvec::{SmallVec, smallvec};

use components::{Bomb, BombNeighbor, Coordinates, EndMessage, TileCover, Uncover};
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

#[cfg(feature = "simple_neighbors")]
use components::Neighbors;

#[cfg(feature = "hierarchical_neighbors")]
use components::{Center, GridChildOf, GridMap};

pub struct BoardPluginV2<T, U> {
    pub running_state: T,
    pub not_pause: U,
}

impl<T: ComputedStates, U: States> Plugin for BoardPluginV2<T, U> {
    fn build(&self, app: &mut App) {
        // When the running states comes into the stack we load a board
        app.add_systems(
            OnEnter(self.running_state.clone()),
            (
                Self::create_board,
                Self::set_bombs,
                #[cfg(all(feature = "simple_neighbors", feature = "hierarchical_neighbors"))]
                Self::check_neighbors,
            )
                .chain(),
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

        #[cfg(all(feature = "simple_neighbors", not(feature = "hierarchical_neighbors")))]
        Self::assign_neighbors(&coords_map, &mut commands);

        #[cfg(feature = "hierarchical_neighbors")]
        let centers = Self::assign_neighbors(&coords_map, &mut commands, options.map_size);

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
                    WithRelated::new(coords_map.clone().into_values()),
                    #[cfg(feature = "hierarchical_neighbors")]
                    WithRelated::new(centers),
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
            #[cfg(not(any(feature = "simple_neighbors", feature = "hierarchical_neighbors")))]
            coords_map,
        });
    }

    /// Places bombs and bomb neighbor tiles
    fn set_bombs(
        #[cfg(not(feature = "simple_neighbors"))] query: Query<(Entity, &Coordinates, &Children)>,
        #[cfg(feature = "simple_neighbors")] query: Query<
            (Entity, &Neighbors, &Children),
            With<Coordinates>,
        >,
        cover_query: Query<(), With<TileCover>>,
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        board_assets: Res<BoardAssets>,
        board: Res<Board>,
        #[cfg(feature = "hierarchical_neighbors")] query_neighbors_2: Query<(
            &Coordinates,
            &Center,
            &GridMap,
        )>,
        #[cfg(feature = "hierarchical_neighbors")] query_neighbor_of: Query<&GridChildOf>,
    ) {
        let mut rng = rng();
        let options = match board_options {
            None => BoardOptions::default(), // If no options is set we use the default one
            Some(o) => o.clone(),
        };
        let bomb_count = options.bomb_count as usize;
        let padding = options.tile_padding;
        let size = board.tile_size;
        #[cfg(feature = "simple_neighbors")]
        let mut entities: Vec<(Entity, &Neighbors)> =
            query.iter().map(|(e, n, _)| (e, n)).collect();
        #[cfg(not(feature = "simple_neighbors"))]
        let mut entities: Vec<(Entity, Coordinates)> =
            query.iter().map(|(e, c, _)| (e, *c)).collect();
        entities.shuffle(&mut rng);
        let mut bomb_entities = HashSet::new();

        for i in 0..bomb_count {
            #[cfg(feature = "simple_neighbors")]
            if let Some((entity, _)) = entities.get(i) {
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
            #[cfg(not(feature = "simple_neighbors"))]
            if let Some((entity, _)) = entities.get(i) {
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

        let start = Instant::now();

        #[cfg(not(any(feature = "simple_neighbors", feature = "hierarchical_neighbors")))]
        {
            for (entity, coords) in entities.iter().skip(bomb_count).copied() {
                let count = SQUARE_COORDINATES
                    .map(|tuple| coords + tuple)
                    .into_iter()
                    .filter_map(|c| board.coords_map.get(&c).copied())
                    .filter(|e| bomb_entities.contains(e))
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

            let elapsed = start.elapsed();
            println!("hash_neighbors took {:?}", elapsed);
        }

        #[cfg(feature = "simple_neighbors")]
        {
            for (entity, neighbors) in entities.iter().skip(bomb_count).copied() {
                let count = neighbors
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

            let elapsed = start.elapsed();
            println!("simple_neighbors took {:?}", elapsed);
        }

        #[cfg(feature = "simple_neighbors")]
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
        #[cfg(all(feature = "hierarchical_neighbors", not(feature = "simple_neighbors")))]
        {
            for (entity, coords) in entities.iter().skip(bomb_count).copied() {
                let count = find_neighbors(entity, coords, &query_neighbors_2, &query_neighbor_of)
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

            let elapsed = start.elapsed();
            println!("hierarchical_neighbors took {:?}", elapsed);
        }
        #[cfg(not(feature = "simple_neighbors"))]
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
    #[cfg(any(feature = "simple_neighbors", feature = "hierarchical_neighbors"))]
    fn assign_neighbors(
        coords_map: &HashMap<Coordinates, Entity>,
        commands: &mut Commands,
        #[cfg(feature = "hierarchical_neighbors")] (width, height): (u16, u16),
    ) -> Vec<Entity> {
        #[cfg(feature = "simple_neighbors")]
        {
            for (&coords, &entity) in coords_map {
                let neighbors = SQUARE_COORDINATES
                    .map(|tuple| coords + tuple)
                    .map(|c| coords_map.get(&c).copied());
                commands.entity(entity).insert(Neighbors(neighbors));
            }
        }

        #[cfg(feature = "hierarchical_neighbors")]
        {
            let mut centers = Vec::new();
            let mut temp = coords_map.clone();
            let mut divisor: u16 = 3;

            while temp.len() > 1 {
                let mut new_map = HashMap::new();
                let level = divisor.ilog(3) as u8;

                for y in 0..height.div_ceil(divisor) {
                    for x in 0..width.div_ceil(divisor) {
                        let center_coords = Coordinates {
                            x: x * divisor + divisor / 2,
                            y: y * divisor + divisor / 2,
                        };

                        let center_entity = commands
                            .spawn((
                                Name::new(format!("Level{} Center {}", level, center_coords)),
                                Center(level),
                                center_coords,
                            ))
                            .id();

                        centers.push(center_entity);
                        new_map.insert(center_coords, center_entity);

                        let mut grid_map = [None; 9];

                        if let Some(&entity) = temp.get(&center_coords) {
                            commands.entity(entity).insert(GridChildOf(center_entity));
                            grid_map[0] = Some((entity, center_coords));
                        }

                        let neighbors = SQUARE_COORDINATES
                            .map(|tuple| center_coords + tuple * divisor as i32 / 3);

                        for (i, coords) in neighbors.into_iter().enumerate() {
                            if let Some(&entity) = temp.get(&coords) {
                                commands.entity(entity).insert(GridChildOf(center_entity));
                                grid_map[i + 1] = Some((entity, coords));
                            }
                        }

                        commands.entity(center_entity).insert(GridMap(grid_map));
                    }
                }

                temp = new_map;
                divisor *= 3;
            }

            return centers;
        }

        Vec::new()
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

    #[cfg(all(feature = "simple_neighbors", feature = "hierarchical_neighbors"))]
    fn check_neighbors(
        query_neighbors: Query<(Entity, &Coordinates, &Neighbors)>,
        query_neighbors_2: Query<(&Coordinates, &Center, &GridMap)>,
        query_neighbor_of: Query<&GridChildOf>,
        query_coordinates: Query<&Coordinates>,
    ) {
        for (entity, &coords, neighbors) in query_neighbors {
            let neighbors: HashSet<Entity> = neighbors.iter().flatten().copied().collect();
            let neighbors_2: HashSet<Entity> =
                find_neighbors(entity, coords, &query_neighbors_2, &query_neighbor_of)
                    .into_iter()
                    .collect();

            if neighbors != neighbors_2 {
                println!("--{}--", coords);

                let all_neighbors: HashSet<Entity> =
                    neighbors.union(&neighbors_2).copied().collect();

                for n in all_neighbors {
                    let coords = query_coordinates.get(n).ok();
                    let in_first = neighbors.contains(&n);
                    let in_second = neighbors_2.contains(&n);
                    match (coords, in_first, in_second) {
                        (Some(coords), true, true) => println!("{} / {}", coords, coords),
                        (Some(coords), true, false) => println!("{} / None", coords),
                        (Some(coords), false, true) => println!("None / {}", coords),
                        _ => println!("None / None"),
                    }
                }
            }
        }
    }
}
#[cfg(feature = "hierarchical_neighbors")]
pub fn find_neighbors(
    entity: Entity,
    coords: Coordinates,
    query_neighbors: &Query<(&Coordinates, &Center, &GridMap)>,
    query_neighbor_of: &Query<&GridChildOf>,
) -> SmallVec<[Entity; 8]> {
    let Ok(child_of) = query_neighbor_of.get(entity) else {
        return smallvec![];
    };
    let center_entity = child_of.0;
    let Ok((&coords_l1, _, grid_map)) = query_neighbors.get(center_entity) else {
        return smallvec![];
    };

    let offset = coords - coords_l1;

    let mut found = SmallVec::new();

    match (offset.x, offset.y) {
        (0, 0) => {
            add_neighbors(&mut found, grid_map, &[1, 2, 3, 4, 5, 6, 7, 8]);
            return found;
        }
        (-1, 0) => add_neighbors(&mut found, grid_map, &[0, 1, 2, 6, 7]),
        (1, 0) => add_neighbors(&mut found, grid_map, &[0, 2, 3, 7, 8]),
        (0, -1) => add_neighbors(&mut found, grid_map, &[0, 1, 3, 4, 5]),
        (0, 1) => add_neighbors(&mut found, grid_map, &[0, 4, 5, 6, 8]),
        (-1, -1) => add_neighbors(&mut found, grid_map, &[0, 2, 4]),
        (1, -1) => add_neighbors(&mut found, grid_map, &[0, 2, 5]),
        (-1, 1) => add_neighbors(&mut found, grid_map, &[0, 4, 7]),
        (1, 1) => add_neighbors(&mut found, grid_map, &[0, 5, 7]),
        _ => {}
    };

    // 2 level
    let Ok(child_of) = query_neighbor_of.get(center_entity) else {
        return found;
    };
    let center_entity = child_of.0;
    let Ok((&coords_l2, _, grid_map)) = query_neighbors.get(center_entity) else {
        return found;
    };

    let index_map: SmallVec<[((i8, i8), SmallVec<[usize; 3]>); 3]> = match (offset.x, offset.y) {
        (-1, 0) => smallvec![((-3, 0), smallvec![3, 5, 8])],
        (1, 0) => smallvec![((3, 0), smallvec![1, 4, 6])],
        (0, -1) => smallvec![((0, -3), smallvec![6, 7, 8])],
        (0, 1) => smallvec![((0, 3), smallvec![1, 2, 3])],
        (-1, -1) => smallvec![
            ((-3, 0), smallvec![3, 5]),
            ((0, -3), smallvec![6, 7]),
            ((-3, -3), smallvec![8]),
        ],
        (1, -1) => smallvec![
            ((3, 0), smallvec![1, 4]),
            ((0, -3), smallvec![7, 8]),
            ((3, -3), smallvec![6]),
        ],
        (-1, 1) => smallvec![
            ((-3, 0), smallvec![5, 8]),
            ((0, 3), smallvec![1, 2]),
            ((-3, 3), smallvec![3]),
        ],
        (1, 1) => smallvec![
            ((3, 0), smallvec![4, 6]),
            ((0, 3), smallvec![2, 3]),
            ((3, 3), smallvec![1]),
        ],
        _ => smallvec![],
    };

    let bounds = IRect::from_center_size(coords_l2.into(), IVec2::splat(9));

    for (tuple, indexes) in index_map {
        let coords_l1 = coords_l1 + tuple;

        if bounds.contains((coords_l1).into()) {
            if let Some(i) = offset_to_index(coords_l1 - coords_l2) {
                if let Some((child_entity, _)) = grid_map[i] {
                    if let Ok((_, _, grid_map)) = query_neighbors.get(child_entity) {
                        add_neighbors(&mut found, grid_map, &indexes);
                    }
                }
            }
        } else if let Ok(child_of) = query_neighbor_of.get(center_entity) {
            found.extend(find_coordinates(
                coords_l1,
                &indexes,
                child_of.0,
                center_entity,
                &query_neighbors,
                &query_neighbor_of,
            ));
        }
    }

    found
}

#[cfg(feature = "hierarchical_neighbors")]
fn find_intersecting(
    area: IRect,
    center_entity: Entity,
    source_entity: Entity,
    visited: &mut Vec<Entity>,
    found: &mut SmallVec<[Entity; 8]>,
    query_neighbors: &Query<(&Center, &GridMap)>,
    query_neighbor_of: &Query<&GridChildOf>,
) {
    if visited.contains(&center_entity) {
        return;
    }
    visited.push(center_entity);

    let Ok((level, grid_map)) = query_neighbors.get(center_entity) else {
        return;
    };

    if **level == 1 {
        for (child_entity, coords) in grid_map.iter().flatten().copied() {
            if child_entity == source_entity {
                continue;
            }

            if area.contains(coords.into()) {
                found.push(child_entity);
            }
        }
    }

    if **level == 2 {
        for (child_entity, coords) in grid_map.iter().flatten().copied() {
            if visited.contains(&child_entity) {
                continue;
            }
            if let Ok((level, grid_map)) = query_neighbors.get(child_entity) {
                let bounds = IRect::from_center_size(coords.into(), get_size(**level));

                if intersects(bounds, area) {
                    for (child_entity, coords) in grid_map.iter().flatten().copied() {
                        if area.contains(coords.into()) {
                            found.push(child_entity);
                        }
                    }
                }
            }
        }
    }

    if **level > 2 {
        for (child_entity, coords) in grid_map.iter().flatten().copied() {
            if let Ok((level, _)) = query_neighbors.get(child_entity) {
                let bounds = IRect::from_center_size(coords.into(), get_size(**level));

                if intersects(bounds, area) {
                    find_intersecting(
                        area,
                        child_entity,
                        source_entity,
                        visited,
                        found,
                        query_neighbors,
                        query_neighbor_of,
                    );
                }
            }
        }
    }

    if found.len() < 8 {
        if let Ok(child_of) = query_neighbor_of.get(center_entity) {
            find_intersecting(
                area,
                child_of.0,
                source_entity,
                visited,
                found,
                query_neighbors,
                query_neighbor_of,
            );
        }
    }
}

#[cfg(feature = "hierarchical_neighbors")]
fn find_coordinates(
    coords_l1: Coordinates,
    indexes: &[usize],
    center_entity: Entity,
    source_entity: Entity,
    query_neighbors: &Query<(&Coordinates, &Center, &GridMap)>,
    query_neighbor_of: &Query<&GridChildOf>,
) -> SmallVec<[Entity; 8]> {
    let Ok((&coords_ln, level, grid_map)) = query_neighbors.get(center_entity) else {
        return smallvec![];
    };

    if **level == 2 {
        if let Some(i) = offset_to_index(coords_l1 - coords_ln) {
            if let Some((child_entity, _)) = grid_map[i] {
                if let Ok((_, _, grid_map)) = query_neighbors.get(child_entity) {
                    let mut result = SmallVec::new();
                    add_neighbors(&mut result, grid_map, indexes);
                    return result;
                }
            }
        }
    }

    for (child_entity, coords_ln) in grid_map.iter().flatten().copied() {
        if child_entity == source_entity {
            continue;
        }

        let bounds = IRect::from_center_size(coords_ln.into(), get_size(**level - 1));
        if bounds.contains(coords_l1.into()) {
            return find_coordinates(
                coords_l1,
                indexes,
                child_entity,
                source_entity,
                query_neighbors,
                query_neighbor_of,
            );
        }
    }

    if let Ok(child_of) = query_neighbor_of.get(center_entity) {
        return find_coordinates(
            coords_l1,
            indexes,
            child_of.0,
            center_entity,
            query_neighbors,
            query_neighbor_of,
        );
    }

    smallvec![]
}

/// Delta coordinates for all 8 square neighbors
pub const SQUARE_COORDINATES: [IVec2; 8] = [
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

fn intersects(a: IRect, b: IRect) -> bool {
    !(a.max.x < b.min.x || b.max.x < a.min.x || a.max.y < b.min.y || b.max.y < a.min.y)
}

#[inline]
fn add_neighbors(found: &mut SmallVec<[Entity; 8]>, grid_map: &GridMap, indexes: &[usize]) {
    for &i in indexes {
        if let Some((e, _)) = grid_map[i] {
            found.push(e);
        }
    }
}

#[inline]
fn offset_to_index(offset: IVec2) -> Option<usize> {
    match (offset.x, offset.y) {
        (-3, -3) => Some(1),
        (0, -3) => Some(2),
        (3, -3) => Some(3),
        (-3, 0) => Some(4),
        (0, 0) => Some(0),
        (3, 0) => Some(5),
        (-3, 3) => Some(6),
        (0, 3) => Some(7),
        (3, 3) => Some(8),
        _ => None,
    }
}
#[inline]
fn get_size(level: u8) -> IVec2 {
    IVec2::splat(3_i32.pow(level as u32))
}
