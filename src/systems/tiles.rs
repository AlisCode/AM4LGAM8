use bevy::{
    ecs::system::Despawn,
    prelude::{Commands, Entity, EventReader, EventWriter, Query, Res, Transform},
    utils::{HashMap, HashSet},
};

use crate::{
    bundles::tile::spawn_tile_type_bundle,
    constants::{GRID_SIZE, TILE_SIZE},
    game::{
        grid::{GridCoordinates, MoveTileEvent, TileGrid},
        moves::{CombineEvent, MoveDirection, ValidMoveEvent, ValidatedEventQueue},
        tile::TileType,
    },
};

use super::movables::RequestMoveEvent;

/// Validates incoming RequestMoveEvent into ValidMoveEvent
pub fn handle_requested_move_events(
    mut requested_event_rx: EventReader<RequestMoveEvent>,
    mut valid_move_event_tx: EventWriter<ValidMoveEvent>,
    mut combine_event_tx: EventWriter<CombineEvent>,
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
                combine_event_tx.send_batch(events.combines);
            }
        }
    }
}

pub fn handle_valid_move_events(
    mut valid_event_rx: EventReader<ValidMoveEvent>,
    mut move_tile_event_tx: EventWriter<MoveTileEvent>,
    mut query: Query<(&mut GridCoordinates, &mut Transform, &TileType)>,
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
    for (mut coords, mut transform, _tile_type) in query.iter_mut() {
        if let Some(new_coords) = old_coords_to_new_coords.remove(&*coords) {
            let old_coords = coords.clone();
            *coords = new_coords;

            // TODO: Animate
            transform.translation.x = coords.x as f32 * TILE_SIZE;
            transform.translation.y = coords.y as f32 * TILE_SIZE;

            move_tile_event_tx.send(MoveTileEvent {
                source: old_coords,
                target: coords.clone(),
            });
        }
    }
}

pub fn handle_combine_events(
    mut commands: Commands,
    mut combine_event_rx: EventReader<CombineEvent>,
    query: Query<(Entity, &GridCoordinates)>,
) {
    let mut grid_coords_to_delete: HashSet<&GridCoordinates> = HashSet::default();
    let mut tiles_to_spawn: Vec<(GridCoordinates, TileType)> = Vec::default();
    for event in combine_event_rx.iter() {
        let CombineEvent {
            source,
            target,
            resulting_type,
        } = event;
        grid_coords_to_delete.extend(vec![source, target]);
        if let Some(tile_type) = resulting_type {
            tiles_to_spawn.push((target.clone(), tile_type.clone()));
        }
    }

    // Despawns tiles that have been combined
    // TODO: Animate
    for despawn_command in query.iter().filter_map(|(entity, grid_coords)| {
        grid_coords_to_delete
            .contains(grid_coords)
            .then(|| Despawn { entity })
    }) {
        commands.add(despawn_command);
    }

    // Spawn tiles issued from combinations
    for (coords, tile_type) in tiles_to_spawn {
        spawn_tile_type_bundle(&mut commands, tile_type, coords.x, coords.y);
    }
}
