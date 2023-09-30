use bevy::{
    prelude::{
        Added, Component, Entity, Event, EventReader, EventWriter, Query, Res, ResMut, Resource,
        Transform,
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

#[derive(Debug, Clone, Copy, Component)]
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
}

#[derive(Debug)]
pub enum PlayerMoveAction {
    MoveTile,
    //CombineTile(GridCoordinates),
}

#[derive(Debug, Event)]
pub struct ValidMoveEvent {
    source: Entity,
    move_direction: MoveDirection,
    move_action: PlayerMoveAction,
}

/// Validates incoming RequestMoveEvent into ValidMoveEvent
pub fn handle_requested_move_events(
    mut requested_event_rx: EventReader<RequestMoveEvent>,
    mut valid_event_tx: EventWriter<ValidMoveEvent>,
    tile_grid: Res<TileGrid>,
) {
    for move_event in requested_event_rx.iter() {
        dbg!(&move_event);
        let RequestMoveEvent {
            source,
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
                .map(|y| GridCoordinates {
                    x: source_coords.x,
                    y,
                })
                .collect(),
        };

        if check_can_move_tile_on_grid(&tile_grid, &candidate_coords, *move_direction) {
            valid_event_tx.send(ValidMoveEvent {
                source: *source,
                move_direction: *move_direction,
                move_action: PlayerMoveAction::MoveTile,
            });
        }
    }
}

fn check_can_move_tile_on_grid(
    tile_grid: &TileGrid,
    candidate_coords: &[GridCoordinates],
    move_direction: MoveDirection,
) -> bool {
    for coord in candidate_coords {
        let can_move_result = tile_grid.can_move_tile(&coord, move_direction);
        match can_move_result {
            CanMoveResult::Yes => return true,
            CanMoveResult::No => return false,
            CanMoveResult::YesIfNextCanMove => (),
        }
    }
    panic!("The map is supposed to be closed");
}

pub fn handle_valid_move_events(
    mut valid_event_rx: EventReader<ValidMoveEvent>,
    mut move_tile_event_tx: EventWriter<MoveTileEvent>,
    mut query: Query<(Entity, &mut GridCoordinates, &mut Transform, &TileType)>,
) {
    for move_event in valid_event_rx.iter() {
        for (entity, mut coords, mut transform, _tile_type) in query.iter_mut() {
            if entity != move_event.source {
                continue;
            }

            match &move_event.move_action {
                PlayerMoveAction::MoveTile => {
                    let old_coords = coords.clone();
                    match move_event.move_direction {
                        MoveDirection::Up => coords.y += 1,
                        MoveDirection::Down => coords.y -= 1,
                        MoveDirection::Left => coords.x -= 1,
                        MoveDirection::Right => coords.x += 1,
                    }
                    // TODO: Animate
                    transform.translation.x = coords.x as f32 * TILE_SIZE;
                    transform.translation.y = coords.y as f32 * TILE_SIZE;

                    dbg!("sending move tile event");
                    move_tile_event_tx.send(MoveTileEvent {
                        source: old_coords,
                        target: coords.clone(),
                    });
                }
            }
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
}

pub fn sync_tile_grid(
    mut tile_grid: ResMut<TileGrid>,
    newly_spawned_tiles: Query<(&TileType, &GridCoordinates), Added<GridCoordinates>>,
    mut move_tile_event_rx: EventReader<MoveTileEvent>,
) {
    for event in move_tile_event_rx.iter() {
        let src_type = tile_grid
            .0
            .remove(&event.source)
            .expect("Failed to find source in grid");
        tile_grid.0.insert(event.target.clone(), src_type);
    }
    for (tile_type, coords) in newly_spawned_tiles.iter() {
        tile_grid.0.insert(coords.clone(), *tile_type);
    }
}

#[cfg(test)]
pub mod tests {
    use super::{CoinValue, TileGrid, TileType};
    use crate::systems::{grid::GridCoordinates, movables::MoveDirection, tiles::CanMoveResult};

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
}
