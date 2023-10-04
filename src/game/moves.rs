use bevy::prelude::Event;

use super::{
    grid::{GridCoordinates, MoveTileEvent, TileGrid},
    tile::{CombinationResult, TileType},
};

// Events

#[derive(Debug, PartialEq, Eq, Event)]
pub struct ValidMoveEvent {
    pub coords: GridCoordinates,
    pub move_direction: MoveDirection,
}

#[derive(Debug, PartialEq, Eq, Clone, Event)]
pub struct MergeTilesEvent {
    pub source: GridCoordinates,
    pub target: GridCoordinates,
    pub resulting_type: Option<TileType>,
}

#[derive(Debug, PartialEq, Eq, Clone, Event)]
pub struct ExplosionEvent {
    pub target: GridCoordinates,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidEvent {
    Move(MoveTileEvent),
    Merge(MergeTilesEvent),
    Explosions(ExplosionEvent),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidatedEventQueue {
    InvalidMove,
    ValidMove(Vec<ValidEvent>),
}

// Domain

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
    Yes(CombinationResult),
    No,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ValidatedEventQueue {
    pub fn valid_move_set(mut events: Vec<ValidEvent>) -> Self {
        events.reverse();
        ValidatedEventQueue::ValidMove(events)
    }

    pub fn validate_move(
        tile_grid: &TileGrid,
        // TODO:
        // coords: GridCoordinates,
        candidate_coords: Vec<GridCoordinates>,
        move_direction: MoveDirection,
    ) -> Self {
        let mut valid_events = Vec::default();
        for coords in candidate_coords {
            match tile_grid.get(&coords) {
                Some(tile_type) => match tile_type {
                    TileType::Bomb | TileType::Coin(_) => (),
                    TileType::Wall => break,
                },
                None => break,
            }
            let can_combine_result = tile_grid.can_combine_tile(&coords, move_direction);
            match can_combine_result {
                CanCombineResult::Yes(result) => {
                    let target = coords.coords_after_move(move_direction);
                    match result {
                        CombinationResult::MergeTilesInto(resulting_type) => {
                            valid_events.push(ValidEvent::Merge(MergeTilesEvent {
                                source: coords,
                                target,
                                resulting_type: Some(resulting_type),
                            }));
                        }
                        CombinationResult::Explosion => {
                            valid_events
                                .push(ValidEvent::Explosions(ExplosionEvent { target: coords }));
                            valid_events.push(ValidEvent::Explosions(ExplosionEvent { target }));
                        }
                    }
                    return ValidatedEventQueue::valid_move_set(valid_events);
                }
                CanCombineResult::No => {
                    let can_move_result = tile_grid.can_move_tile(&coords, move_direction);
                    match can_move_result {
                        CanMoveResult::Yes => {
                            let target = coords.coords_after_move(move_direction);
                            valid_events.push(ValidEvent::Move(MoveTileEvent {
                                source: coords,
                                target,
                            }));
                            return ValidatedEventQueue::valid_move_set(valid_events);
                        }
                        CanMoveResult::No => return ValidatedEventQueue::InvalidMove,
                        CanMoveResult::YesIfNextCanMove => {
                            // Otherwise we add the valid Move event and keep checking the
                            // move with the next candidate
                            let target = coords.coords_after_move(move_direction);
                            let valid_move_event = MoveTileEvent {
                                source: coords,
                                target,
                            };
                            valid_events.push(ValidEvent::Move(valid_move_event));
                        }
                    }
                }
            }
        }

        ValidatedEventQueue::InvalidMove
    }
}

#[cfg(test)]
pub mod tests {
    use crate::game::{
        grid::{GridCoordinates, MoveTileEvent, TileGrid},
        moves::{ExplosionEvent, MergeTilesEvent, MoveDirection, ValidEvent, ValidatedEventQueue},
        tile::{CoinValue, TileType},
    };

    #[test]
    fn should_move_tile() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        let move_events = vec![MoveTileEvent {
            source: GridCoordinates { x: 0, y: 0 },
            target: GridCoordinates { x: 1, y: 0 },
        }];
        tile_grid.handle_move_tile_events(move_events.iter());

        assert!(!tile_grid.get(&GridCoordinates { x: 0, y: 0 }).is_some());
        assert!(tile_grid.get(&GridCoordinates { x: 1, y: 0 }).is_some());
    }

    #[test]
    fn should_move_many_tiles() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
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

        assert!(!tile_grid.get(&GridCoordinates { x: 0, y: 0 }).is_some());
        assert_eq!(
            tile_grid.get(&GridCoordinates { x: 1, y: 0 }),
            Some(&TileType::Coin(CoinValue::One)),
        );
        assert_eq!(
            tile_grid.get(&GridCoordinates { x: 2, y: 0 }),
            Some(&TileType::Coin(CoinValue::Two)),
        );
    }

    #[test]
    fn should_validate_moves() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
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
            &ValidatedEventQueue::ValidMove(vec![
                ValidEvent::Move(MoveTileEvent {
                    source: GridCoordinates { x: 1, y: 0 },
                    target: GridCoordinates { x: 2, y: 0 },
                }),
                ValidEvent::Move(MoveTileEvent {
                    source: GridCoordinates { x: 0, y: 0 },
                    target: GridCoordinates { x: 1, y: 0 },
                }),
            ],)
        );
    }

    #[test]
    fn should_return_that_move_is_invalid() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(GridCoordinates { x: -1, y: 0 }, TileType::Wall);
        tile_grid.insert(GridCoordinates { x: 0, y: 0 }, TileType::Bomb);
        tile_grid.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        let coords = GridCoordinates { x: 1, y: 0 }.candidate_coords_for_dir(MoveDirection::Left);
        let invalid_move =
            ValidatedEventQueue::validate_move(&tile_grid, coords, MoveDirection::Left);
        assert_eq!(invalid_move, ValidatedEventQueue::InvalidMove);
    }

    #[test]
    fn should_push_to_explode_tiles() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(GridCoordinates { x: 0, y: 0 }, TileType::Bomb);
        tile_grid.insert(GridCoordinates { x: 1, y: 0 }, TileType::Bomb);
        tile_grid.insert(
            GridCoordinates { x: 2, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        let coords = GridCoordinates { x: 2, y: 0 }.candidate_coords_for_dir(MoveDirection::Left);
        let validated_event_queue =
            ValidatedEventQueue::validate_move(&tile_grid, coords, MoveDirection::Left);
        assert_eq!(
            validated_event_queue,
            ValidatedEventQueue::ValidMove(vec![
                ValidEvent::Explosions(ExplosionEvent {
                    target: GridCoordinates { x: 0, y: 0 }
                }),
                ValidEvent::Explosions(ExplosionEvent {
                    target: GridCoordinates { x: 1, y: 0 }
                }),
                ValidEvent::Move(MoveTileEvent {
                    source: GridCoordinates { x: 2, y: 0 },
                    target: GridCoordinates { x: 1, y: 0 },
                }),
            ])
        );
    }

    #[test]
    fn should_push_to_merge_tiles_right() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(GridCoordinates { x: 0, y: 0 }, TileType::Bomb);
        tile_grid.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 2, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        let coords = GridCoordinates { x: 0, y: 0 }.candidate_coords_for_dir(MoveDirection::Right);
        let validated_event_queue =
            ValidatedEventQueue::validate_move(&tile_grid, coords, MoveDirection::Right);
        assert_eq!(
            validated_event_queue,
            ValidatedEventQueue::ValidMove(vec![
                ValidEvent::Merge(MergeTilesEvent {
                    source: GridCoordinates { x: 1, y: 0 },
                    target: GridCoordinates { x: 2, y: 0 },
                    resulting_type: Some(TileType::Coin(CoinValue::Two)),
                }),
                ValidEvent::Move(MoveTileEvent {
                    source: GridCoordinates { x: 0, y: 0 },
                    target: GridCoordinates { x: 1, y: 0 },
                }),
            ])
        );
    }

    #[test]
    fn should_push_to_merge_tiles_left() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(GridCoordinates { x: 2, y: 0 }, TileType::Bomb);
        tile_grid.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        let coords = GridCoordinates { x: 2, y: 0 }.candidate_coords_for_dir(MoveDirection::Left);
        let validated_event_queue =
            ValidatedEventQueue::validate_move(&tile_grid, coords, MoveDirection::Left);
        assert_eq!(
            validated_event_queue,
            ValidatedEventQueue::ValidMove(vec![
                ValidEvent::Merge(MergeTilesEvent {
                    source: GridCoordinates { x: 1, y: 0 },
                    target: GridCoordinates { x: 0, y: 0 },
                    resulting_type: Some(TileType::Coin(CoinValue::Two)),
                }),
                ValidEvent::Move(MoveTileEvent {
                    source: GridCoordinates { x: 2, y: 0 },
                    target: GridCoordinates { x: 1, y: 0 },
                }),
            ])
        );
    }
}
