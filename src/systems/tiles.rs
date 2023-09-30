use bevy::{
    prelude::{
        Added, Component, Event, EventReader, EventWriter, Query, Res, ResMut, Resource, Transform,
    },
    utils::HashMap,
};

use crate::constants::TILE_SIZE;

use super::{
    grid::{GridCoordinates, MoveTileEvent, GRID_SIZE},
    movables::{MoveDirection, RequestMoveEvent},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum CoinValue {
    One,
    Two,
    Four,
    Eight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum TileType {
    Coin(CoinValue),
    Wall,
    // Bomb,
}

impl TileType {
    pub fn is_movable(&self) -> bool {
        match self {
            TileType::Wall => false,
            TileType::Coin(_) => true,
        }
    }

    pub fn can_combine_with(&self, other: &TileType) -> bool {
        match (self, other) {
            (TileType::Coin(CoinValue::One), TileType::Coin(CoinValue::One)) => true,
            (TileType::Coin(CoinValue::Two), TileType::Coin(CoinValue::Two)) => true,
            (TileType::Coin(CoinValue::Four), TileType::Coin(CoinValue::Four)) => true,
            (TileType::Coin(CoinValue::Eight), TileType::Coin(CoinValue::Eight)) => true,
            (TileType::Coin(_), TileType::Coin(_) | TileType::Wall) => false,
            (TileType::Wall, TileType::Coin(_) | TileType::Wall) => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Event)]
pub struct ValidMoveEvent {
    coords: GridCoordinates,
    move_direction: MoveDirection,
}

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

#[derive(Debug, PartialEq, Eq)]
struct ValidEvents {
    moves: Vec<ValidMoveEvent>,
}

#[derive(Debug, PartialEq, Eq)]
enum ValidatedEventQueue {
    InvalidMove,
    ValidMove(ValidEvents),
}

impl ValidatedEventQueue {
    pub fn validate_move(
        tile_grid: &TileGrid,
        candidate_coords: Vec<GridCoordinates>,
        move_direction: MoveDirection,
    ) -> Self {
        let mut valid_events = ValidEvents {
            moves: Vec::default(),
        };
        for coords in candidate_coords {
            let can_combine_result = tile_grid.can_combine_tile(&coords, move_direction);
            match can_combine_result {
                CanCombineResult::Yes => {
                    // TODO: Push combine event
                    return ValidatedEventQueue::ValidMove(valid_events);
                }
                CanCombineResult::No => {
                    let can_move_result = tile_grid.can_move_tile(&coords, move_direction);
                    match can_move_result {
                        CanMoveResult::Yes => {
                            let valid_move_event = ValidMoveEvent {
                                coords,
                                move_direction,
                            };
                            valid_events.moves.push(valid_move_event);
                            return ValidatedEventQueue::ValidMove(valid_events);
                        }
                        CanMoveResult::No => return ValidatedEventQueue::InvalidMove,
                        CanMoveResult::YesIfNextCanMove => {
                            // Otherwise we add the valid Move event and keep checking the
                            // move with the next candidate
                            let valid_move_event = ValidMoveEvent {
                                coords,
                                move_direction,
                            };
                            valid_events.moves.push(valid_move_event);
                        }
                    }
                }
            }
        }

        ValidatedEventQueue::InvalidMove
    }
}

pub fn handle_valid_move_events(
    mut valid_event_rx: EventReader<ValidMoveEvent>,
    mut move_tile_event_tx: EventWriter<MoveTileEvent>,
    mut query: Query<(&mut GridCoordinates, &mut Transform, &TileType)>,
) {
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

#[derive(Debug, Default, Resource)]
pub struct TileGrid(HashMap<GridCoordinates, TileType>);

#[derive(Debug, PartialEq, Eq)]
pub enum CanMoveResult {
    // Yes, there is an empty spot available after the move
    Yes,
    // After the move, the target spot is occupied
    // We need to check if this can move
    YesIfNextCanMove,
    // We're colliding with something that is unmovable
    No,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CanCombineResult {
    Yes,
    No,
}

impl TileGrid {
    fn can_move_tile(&self, at_coords: &GridCoordinates, dir: MoveDirection) -> CanMoveResult {
        let GridCoordinates { x, y } = at_coords;
        let target_coords = match dir {
            MoveDirection::Left => GridCoordinates { x: x - 1, y: *y },
            MoveDirection::Right => GridCoordinates { x: x + 1, y: *y },
            MoveDirection::Up => GridCoordinates { x: *x, y: y + 1 },
            MoveDirection::Down => GridCoordinates { x: *x, y: y - 1 },
        };

        if let Some(tile_type) = self.0.get(&target_coords) {
            if !tile_type.is_movable() {
                return CanMoveResult::No;
            }
            return CanMoveResult::YesIfNextCanMove;
        }

        CanMoveResult::Yes
    }

    fn can_combine_tile(
        &self,
        at_coords: &GridCoordinates,
        dir: MoveDirection,
    ) -> CanCombineResult {
        let GridCoordinates { x, y } = at_coords;
        let src_type = self.0.get(at_coords).expect("Failed to find source");
        let target_coords = match dir {
            MoveDirection::Left => GridCoordinates { x: x - 1, y: *y },
            MoveDirection::Right => GridCoordinates { x: x + 1, y: *y },
            MoveDirection::Up => GridCoordinates { x: *x, y: y + 1 },
            MoveDirection::Down => GridCoordinates { x: *x, y: y - 1 },
        };

        if let Some(tile_type) = self.0.get(&target_coords) {
            if src_type.can_combine_with(tile_type) {
                return CanCombineResult::Yes;
            }
            return CanCombineResult::No;
        }
        CanCombineResult::No
    }

    fn handle_move_tile_events<'a, I: Iterator<Item = &'a MoveTileEvent>>(&mut self, events: I) {
        let new_tiles: Vec<(GridCoordinates, TileType)> = events
            .map(|MoveTileEvent { source, target }| {
                let src_type = self
                    .0
                    .remove(source)
                    .expect("Failed to find source in grid");
                (target.clone(), src_type)
            })
            .collect();
        for (coord, new_type) in new_tiles {
            self.0.insert(coord, new_type);
        }
    }
}

pub fn sync_tile_grid(
    mut tile_grid: ResMut<TileGrid>,
    newly_spawned_tiles: Query<(&TileType, &GridCoordinates), Added<GridCoordinates>>,
    mut move_tile_event_rx: EventReader<MoveTileEvent>,
) {
    let events: Vec<_> = move_tile_event_rx.iter().collect();
    dbg!(&events);
    tile_grid.handle_move_tile_events(events.into_iter());
    for (tile_type, coords) in newly_spawned_tiles.iter() {
        tile_grid.0.insert(coords.clone(), *tile_type);
    }
}

#[cfg(test)]
pub mod tests {
    use super::{CoinValue, TileGrid, TileType, ValidatedEventQueue};
    use crate::systems::{
        grid::{GridCoordinates, MoveTileEvent},
        movables::MoveDirection,
        tiles::{CanMoveResult, ValidEvents, ValidMoveEvent},
    };

    #[test]
    fn can_move_tile_when_empty() {
        let mut tile_grid = TileGrid::default();
        tile_grid.0.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Right),
            CanMoveResult::Yes
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Left),
            CanMoveResult::Yes
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Up),
            CanMoveResult::Yes
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Down),
            CanMoveResult::Yes
        );
    }

    #[test]
    fn can_not_move_when_next_to_wall() {
        let mut tile_grid = TileGrid::default();
        tile_grid.0.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid
            .0
            .insert(GridCoordinates { x: 1, y: 0 }, TileType::Wall);

        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Right),
            CanMoveResult::No
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Left),
            CanMoveResult::Yes
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Up),
            CanMoveResult::Yes
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Down),
            CanMoveResult::Yes
        );
    }

    #[test]
    fn can_move_when_next_to_coin() {
        let mut tile_grid = TileGrid::default();
        tile_grid.0.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.0.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Right),
            CanMoveResult::YesIfNextCanMove
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Left),
            CanMoveResult::Yes
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Up),
            CanMoveResult::Yes
        );
        assert_eq!(
            tile_grid.can_move_tile(&GridCoordinates { x: 0, y: 0 }, MoveDirection::Down),
            CanMoveResult::Yes
        );
    }

    #[test]
    fn should_move_tile() {
        let mut tile_grid = TileGrid::default();
        tile_grid.0.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        let move_events = vec![MoveTileEvent {
            source: GridCoordinates { x: 0, y: 0 },
            target: GridCoordinates { x: 1, y: 0 },
        }];
        tile_grid.handle_move_tile_events(move_events.iter());

        assert!(!tile_grid.0.contains_key(&GridCoordinates { x: 0, y: 0 }));
        assert!(tile_grid.0.contains_key(&GridCoordinates { x: 1, y: 0 }));
    }

    #[test]
    fn should_move_many_tiles() {
        let mut tile_grid = TileGrid::default();
        tile_grid.0.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.0.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::Two),
        );

        let move_events = vec![
            MoveTileEvent {
                source: GridCoordinates { x: 0, y: 0 },
                target: GridCoordinates { x: 1, y: 0 },
            },
            MoveTileEvent {
                source: GridCoordinates { x: 1, y: 0 },
                target: GridCoordinates { x: 2, y: 0 },
            },
        ];
        tile_grid.handle_move_tile_events(move_events.iter());

        assert!(!tile_grid.0.contains_key(&GridCoordinates { x: 0, y: 0 }));
        assert_eq!(
            tile_grid.0[&GridCoordinates { x: 1, y: 0 }],
            TileType::Coin(CoinValue::One),
        );
        assert_eq!(
            tile_grid.0[&GridCoordinates { x: 2, y: 0 }],
            TileType::Coin(CoinValue::Two),
        );
    }

    #[test]
    fn should_validate_move() {
        let mut tile_grid = TileGrid::default();
        tile_grid.0.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.0.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::Two),
        );

        let validated_event_queue = ValidatedEventQueue::validate_move(
            &tile_grid,
            vec![
                GridCoordinates { x: 0, y: 0 },
                GridCoordinates { x: 1, y: 0 },
                GridCoordinates { x: 2, y: 0 },
                GridCoordinates { x: 3, y: 0 },
            ],
            MoveDirection::Right,
        );
        assert_eq!(
            &validated_event_queue,
            &ValidatedEventQueue::ValidMove(ValidEvents {
                moves: vec![
                    ValidMoveEvent {
                        move_direction: MoveDirection::Right,
                        coords: GridCoordinates { x: 0, y: 0 },
                    },
                    ValidMoveEvent {
                        move_direction: MoveDirection::Right,
                        coords: GridCoordinates { x: 1, y: 0 },
                    }
                ],
            }),
        );
    }
}
