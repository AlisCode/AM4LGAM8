use bevy::{
    prelude::{Component, Event, Resource},
    utils::HashMap,
};

use super::{
    moves::{CanCombineResult, CanMoveResult, ExplosionEvent, MergeTilesEvent, MoveDirection},
    tile::TileType,
};

// Components

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
pub struct GridCoordinates {
    pub x: i32,
    pub y: i32,
}

impl GridCoordinates {
    pub fn coords_after_move(&self, dir: MoveDirection) -> GridCoordinates {
        let GridCoordinates { x, y } = self;
        match dir {
            MoveDirection::Left => GridCoordinates { x: x - 1, y: *y },
            MoveDirection::Right => GridCoordinates { x: x + 1, y: *y },
            MoveDirection::Up => GridCoordinates { x: *x, y: y + 1 },
            MoveDirection::Down => GridCoordinates { x: *x, y: y - 1 },
        }
    }

    pub fn explosion_radius(&self) -> Vec<GridCoordinates> {
        vec![
            self.clone(),
            self.coords_after_move(MoveDirection::Up),
            self.coords_after_move(MoveDirection::Down),
            self.coords_after_move(MoveDirection::Left),
            self.coords_after_move(MoveDirection::Right),
        ]
    }
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
        let target_coords = at_coords.coords_after_move(dir);
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
        let src_type = self.0.get(at_coords).expect("Failed to find source");
        let target_coords = at_coords.coords_after_move(dir);

        if let Some(tile_type) = self.0.get(&target_coords) {
            if let Some(result) = src_type.try_combine_with(tile_type) {
                return CanCombineResult::Yes(result);
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

    pub fn handle_combine_events<'a, I: Iterator<Item = &'a MergeTilesEvent>>(
        &mut self,
        events: I,
    ) {
        let mut deletions = Vec::default();
        let mut maybe_insertions = Vec::default();
        for event in events {
            let MergeTilesEvent {
                source,
                target,
                resulting_type,
            } = event;
            deletions.push(source);
            deletions.push(target);
            maybe_insertions.push((target.clone(), resulting_type.clone()));
        }

        for deletion in deletions {
            self.0.remove(deletion);
        }
        for (coords, maybe_tile_type) in maybe_insertions {
            self.maybe_insert(coords, maybe_tile_type);
        }
    }

    pub fn handle_explosion_events<'a, I: Iterator<Item = &'a ExplosionEvent>>(
        &mut self,
        events: I,
    ) {
        for event in events {
            let ExplosionEvent { target } = event;
            for coord in target.explosion_radius() {
                self.0.remove(&coord);
            }
        }
    }

    pub fn insert(&mut self, coords: GridCoordinates, tile_type: TileType) -> Option<TileType> {
        self.0.insert(coords, tile_type)
    }

    pub fn maybe_insert(
        &mut self,
        coords: GridCoordinates,
        maybe_tile_type: Option<TileType>,
    ) -> Option<TileType> {
        if let Some(tile_type) = maybe_tile_type {
            return self.insert(coords, tile_type);
        }
        self.0.remove(&coords)
    }

    pub fn get(&self, coords: &GridCoordinates) -> Option<&TileType> {
        self.0.get(coords)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::game::{
        grid::{GridCoordinates, TileGrid},
        moves::{CanMoveResult, ExplosionEvent, MergeTilesEvent, MoveDirection},
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

    #[test]
    fn should_maybe_insert_tile() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(GridCoordinates { x: 0, y: 0 }, TileType::Wall);

        tile_grid.maybe_insert(GridCoordinates { x: 1, y: 0 }, Some(TileType::Wall));
        assert_eq!(
            tile_grid.get(&GridCoordinates { x: 1, y: 0 }),
            Some(&TileType::Wall)
        );

        tile_grid.maybe_insert(GridCoordinates { x: 1, y: 0 }, None);
        assert_eq!(tile_grid.get(&GridCoordinates { x: 1, y: 0 }), None);
    }

    #[test]
    fn should_combine_coin_tiles() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        let events = vec![MergeTilesEvent {
            source: GridCoordinates { x: 0, y: 0 },
            target: GridCoordinates { x: 1, y: 0 },
            resulting_type: Some(TileType::Coin(CoinValue::Two)),
        }];
        tile_grid.handle_combine_events(events.iter());

        assert_eq!(tile_grid.get(&GridCoordinates { x: 0, y: 0 }), None);
        assert_eq!(
            tile_grid.get(&GridCoordinates { x: 1, y: 0 }),
            Some(&TileType::Coin(CoinValue::Two))
        );
    }

    #[test]
    fn should_explode_tiles() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 3, y: 0 },
            TileType::Coin(CoinValue::Two),
        );

        let events = vec![ExplosionEvent {
            target: GridCoordinates { x: 0, y: 0 },
        }];
        tile_grid.handle_explosion_events(events.iter());

        assert_eq!(tile_grid.get(&GridCoordinates { x: 0, y: 0 }), None);
        assert_eq!(tile_grid.get(&GridCoordinates { x: 1, y: 0 }), None,);
        assert_eq!(
            tile_grid.get(&GridCoordinates { x: 3, y: 0 }),
            Some(&TileType::Coin(CoinValue::Two))
        );
    }
}
