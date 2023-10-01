use bevy::{
    ecs::system::Despawn,
    prelude::{Commands, Entity, EventReader, EventWriter, Query, Res, ResMut, Transform},
    utils::{HashMap, HashSet},
};

use crate::{
    assets::GameAssets,
    bundles::{explosion::ExplosionBundle, tile::spawn_tile_type_bundle},
    constants::GRID_SIZE,
    game::{
        grid::{GridCoordinates, MoveTileEvent, TileGrid},
        moves::{
            ExplosionEvent, MergeTilesEvent, MoveDirection, ValidMoveEvent, ValidatedEventQueue,
        },
        tile::{ExplosionResult, TileType},
    },
};

use super::{
    animations::{add_movement_animation, AndDeleteAfter},
    grid::ValidTurnEvent,
    movables::RequestMoveEvent,
    ui::GameScore,
    OnPlayingScreen,
};

/// Validates incoming RequestMoveEvent into ValidMoveEvent
pub fn handle_requested_move_events(
    mut requested_event_rx: EventReader<RequestMoveEvent>,
    mut valid_move_event_tx: EventWriter<ValidMoveEvent>,
    mut combine_event_tx: EventWriter<MergeTilesEvent>,
    mut explosion_event_tx: EventWriter<ExplosionEvent>,
    mut valid_turn_tx: EventWriter<ValidTurnEvent>,
    tile_grid: Res<TileGrid>,
) {
    for move_event in requested_event_rx.iter() {
        let RequestMoveEvent {
            move_direction,
            source_coords,
        } = move_event;

        // TODO: Extract to game logic
        let candidate_coords: Vec<GridCoordinates> = match move_direction {
            MoveDirection::Left => (-1..=source_coords.x)
                .rev()
                .map(|x| GridCoordinates {
                    x,
                    y: source_coords.y,
                })
                .collect(),
            MoveDirection::Right => (source_coords.x..=GRID_SIZE)
                .map(|x| GridCoordinates {
                    x,
                    y: source_coords.y,
                })
                .collect(),
            MoveDirection::Up => (source_coords.y..=GRID_SIZE)
                .map(|y| GridCoordinates {
                    x: source_coords.x,
                    y,
                })
                .collect(),
            MoveDirection::Down => (-1..=source_coords.y)
                .rev()
                .map(|y| GridCoordinates {
                    x: source_coords.x,
                    y,
                })
                .collect(),
        };

        let validated_event_queue =
            ValidatedEventQueue::validate_move(&tile_grid, candidate_coords, *move_direction);

        match validated_event_queue {
            ValidatedEventQueue::InvalidMove => (),
            ValidatedEventQueue::ValidMove(events) => {
                valid_move_event_tx.send_batch(events.moves);
                combine_event_tx.send_batch(events.merges);
                explosion_event_tx.send_batch(events.explosions);
                valid_turn_tx.send(ValidTurnEvent);
            }
        }
    }
}

pub fn handle_valid_move_events(
    mut commands: Commands,
    mut valid_event_rx: EventReader<ValidMoveEvent>,
    mut move_tile_event_tx: EventWriter<MoveTileEvent>,
    mut query: Query<(Entity, &mut GridCoordinates, &Transform, &TileType)>,
    mut tile_grid: ResMut<TileGrid>,
) {
    // Necessary because mutating the coords as we go may have unwanted side-effects
    let mut old_coords_to_new_coords: HashMap<GridCoordinates, GridCoordinates> = valid_event_rx
        .iter()
        .map(
            |ValidMoveEvent {
                 coords,
                 move_direction,
             }| {
                let mut new_coords = coords.clone();
                match move_direction {
                    MoveDirection::Up => new_coords.y += 1,
                    MoveDirection::Down => new_coords.y -= 1,
                    MoveDirection::Left => new_coords.x -= 1,
                    MoveDirection::Right => new_coords.x += 1,
                };
                (coords.clone(), new_coords)
            },
        )
        .collect();
    let mut events = Vec::default();
    for (entity, mut coords, transform, _tile_type) in query.iter_mut() {
        if let Some(new_coords) = old_coords_to_new_coords.remove(&*coords) {
            let old_coords = coords.clone();
            *coords = new_coords;

            add_movement_animation(
                &mut commands,
                entity,
                transform,
                coords.clone(),
                AndDeleteAfter::No,
            );

            events.push(MoveTileEvent {
                source: old_coords,
                target: coords.clone(),
            });
        }
    }
    tile_grid.handle_move_tile_events(events.iter());
    move_tile_event_tx.send_batch(events);
}

pub fn handle_combine_events(
    mut commands: Commands,
    mut combine_event_rx: EventReader<MergeTilesEvent>,
    query: Query<(Entity, &GridCoordinates, &Transform)>,
    assets: Res<GameAssets>,
    mut tile_grid: ResMut<TileGrid>,
) {
    let mut grid_coords_to_new_coords: HashMap<GridCoordinates, GridCoordinates> =
        HashMap::default();
    let mut tiles_to_spawn: Vec<(GridCoordinates, TileType)> = Vec::default();
    let mut events: Vec<_> = Vec::default();
    for event in combine_event_rx.iter() {
        events.push(event.clone());
        let MergeTilesEvent {
            source,
            target,
            resulting_type,
        } = event;
        grid_coords_to_new_coords.insert(source.clone(), target.clone());
        grid_coords_to_new_coords.insert(target.clone(), target.clone());
        if let Some(tile_type) = resulting_type {
            tiles_to_spawn.push((target.clone(), tile_type.clone()));
        }
    }
    tile_grid.handle_combine_events(events.iter());

    // Play the animation to combine the tiles
    for (entity, grid_coords, transform) in query.iter() {
        if let Some(new_coords) = grid_coords_to_new_coords.get(grid_coords) {
            if grid_coords == new_coords {
                // TODO: Animate fusion ?
                commands.add(Despawn { entity })
            } else {
                add_movement_animation(
                    &mut commands,
                    entity,
                    transform,
                    new_coords.clone(),
                    AndDeleteAfter::Yes,
                );
            }
        }
    }

    // Spawn tiles issued from combinations
    for (coords, tile_type) in tiles_to_spawn {
        spawn_tile_type_bundle(
            &mut commands,
            assets.tileset.clone(),
            tile_type,
            coords.x,
            coords.y,
        );
    }
}

pub fn handle_explosion_events(
    mut commands: Commands,
    mut explosion_event_rx: EventReader<ExplosionEvent>,
    query: Query<(Entity, &GridCoordinates, &TileType)>,
    mut tile_grid: ResMut<TileGrid>,
    mut game_score: ResMut<GameScore>,
    assets: Res<GameAssets>,
) {
    let mut grid_coords_to_delete: HashSet<GridCoordinates> = HashSet::default();
    let mut events = Vec::default();
    for event in explosion_event_rx.iter() {
        events.push(event.clone());
        let ExplosionEvent { target } = event;
        grid_coords_to_delete.extend(target.explosion_radius());
    }
    tile_grid.handle_explosion_events(events.iter());

    let assets = &*assets;
    for (entity, coords, tile_type) in query.iter() {
        if !grid_coords_to_delete.contains(coords) {
            continue;
        }

        match tile_type.explosion_result() {
            ExplosionResult::NoExplosion => continue,
            ExplosionResult::ScorePoints(points) => {
                game_score.add(points);
                commands.spawn((
                    ExplosionBundle::new(&assets, coords.clone()),
                    OnPlayingScreen,
                ));
                commands.add(Despawn { entity });
            }
        }
    }
}
