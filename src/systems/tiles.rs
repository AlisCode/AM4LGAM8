use bevy::{
    prelude::{EventReader, EventWriter, Query, Res, Transform},
    utils::HashMap,
};

use crate::{
    constants::{GRID_SIZE, TILE_SIZE},
    game::{
        grid::{GridCoordinates, MoveTileEvent, TileGrid},
        moves::{MoveDirection, ValidMoveEvent, ValidatedEventQueue},
        tile::TileType,
    },
};

use super::movables::RequestMoveEvent;

/// Validates incoming RequestMoveEvent into ValidMoveEvent
pub fn handle_requested_move_events(
    mut requested_event_rx: EventReader<RequestMoveEvent>,
    mut valid_move_event_tx: EventWriter<ValidMoveEvent>,
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
