use bevy::{
    prelude::{Component, Event, Resource},
    utils::HashMap,
};

use super::{
    moves::{CanCombineResult, CanMoveResult, MoveDirection},
    tile::TileType,
};

// Components

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
pub struct GridCoordinates {
    pub x: i32,
    pub y: i32,
}

// Events

#[derive(Debug, Event)]
pub struct MoveTileEvent {
    pub source: GridCoordinates,
    pub target: GridCoordinates,
}

// Resource

#[derive(Debug, Default, Resource)]
pub struct TileGrid(HashMap<GridCoordinates, TileType>);

impl TileGrid {
    pub fn can_move_tile(&self, at_coords: &GridCoordinates, dir: MoveDirection) -> CanMoveResult {
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

    pub fn can_combine_tile(
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

    pub fn handle_move_tile_events<'a, I: Iterator<Item = &'a MoveTileEvent>>(
        &mut self,
        events: I,
    ) {
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

    pub fn insert(&mut self, coords: GridCoordinates, tile_type: TileType) -> Option<TileType> {
        self.0.insert(coords, tile_type)
    }

    pub fn get(&self, coords: &GridCoordinates) -> Option<&TileType> {
        self.0.get(coords)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::game::{
        grid::{GridCoordinates, TileGrid},
        moves::{CanMoveResult, MoveDirection},
        tile::{CoinValue, TileType},
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
        tile_grid.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(GridCoordinates { x: 1, y: 0 }, TileType::Wall);

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
