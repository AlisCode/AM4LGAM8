use bevy::{
    prelude::{Component, Event, Resource},
    utils::{HashMap, HashSet},
};
use rand::Rng;

use crate::constants::GRID_SIZE;

use super::{
    moves::{CanCombineResult, CanMoveResult, ExplosionEvent, MergeTilesEvent, MoveDirection},
    tile::{ExplosionResult, TileType},
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

#[derive(Debug, Resource)]
pub struct TileGrid {
    grid: HashMap<GridCoordinates, TileType>,
    unused_coordinates: HashSet<GridCoordinates>,
}

impl Default for TileGrid {
    fn default() -> Self {
        let unused_coordinates = (0..GRID_SIZE)
            .flat_map(|x| (0..GRID_SIZE).map(move |y| GridCoordinates { x, y }))
            .collect();
        TileGrid {
            grid: HashMap::default(),
            unused_coordinates,
        }
    }
}

impl TileGrid {
    pub fn can_move_tile(&self, at_coords: &GridCoordinates, dir: MoveDirection) -> CanMoveResult {
        let target_coords = at_coords.coords_after_move(dir);
        if let Some(tile_type) = self.grid.get(&target_coords) {
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
        let src_type = self.grid.get(at_coords).expect("Failed to find source");
        let target_coords = at_coords.coords_after_move(dir);

        if let Some(tile_type) = self.grid.get(&target_coords) {
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
                    .grid
                    .remove(source)
                    .expect("Failed to find source in grid");
                self.unused_coordinates.insert(source.clone());
                (target.clone(), src_type)
            })
            .collect();
        for (coord, new_type) in new_tiles {
            self.insert(coord, new_type);
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

        for deletion_coords in deletions {
            self.unused_coordinates.insert(deletion_coords.clone());
            self.grid.remove(deletion_coords);
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
                if let Some(tile) = self.get(&coord) {
                    match tile.explosion_result() {
                        ExplosionResult::NoExplosion => (),
                        ExplosionResult::ScorePoints(_) => {
                            self.unused_coordinates.insert(coord.clone());
                            self.grid.remove(&coord);
                        }
                    }
                }
            }
        }
    }

    pub fn get_unused_coordinate(&self) -> Option<GridCoordinates> {
        // Quick and hacky way to get an element from a set
        // I don't have a better idea rn
        let idx = rand::thread_rng().gen_range(0..self.unused_coordinates.len());
        self.unused_coordinates.iter().skip(idx).next().cloned()
    }

    pub fn insert(&mut self, coords: GridCoordinates, tile_type: TileType) -> Option<TileType> {
        self.unused_coordinates.remove(&coords);
        self.grid.insert(coords, tile_type)
    }

    pub fn maybe_insert(
        &mut self,
        coords: GridCoordinates,
        maybe_tile_type: Option<TileType>,
    ) -> Option<TileType> {
        if let Some(tile_type) = maybe_tile_type {
            self.unused_coordinates.remove(&coords);
            return self.insert(coords, tile_type);
        }
        self.unused_coordinates.insert(coords.clone());
        self.grid.remove(&coords)
    }

    pub fn get(&self, coords: &GridCoordinates) -> Option<&TileType> {
        self.grid.get(coords)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::game::{
        grid::{GridCoordinates, MoveTileEvent, TileGrid},
        moves::{CanMoveResult, ExplosionEvent, MergeTilesEvent, MoveDirection},
        tile::{CoinValue, TileType},
    };

    #[test]
    fn can_move_tile_when_empty() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(
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
        tile_grid.insert(
            GridCoordinates { x: 0, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
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
        // x
        // x
        // x11 1
        // xxxxxx
        // Explosion at 0,0 should remove :
        // * 0,0
        // * 1,0
        // * Leave walls untouched
        tile_grid.insert(GridCoordinates { x: -1, y: 0 }, TileType::Wall);
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

        assert_eq!(
            tile_grid.get(&GridCoordinates { x: -1, y: 0 }),
            Some(&TileType::Wall)
        );
        assert_eq!(tile_grid.get(&GridCoordinates { x: 0, y: 0 }), None);
        assert_eq!(tile_grid.get(&GridCoordinates { x: 1, y: 0 }), None,);
        assert_eq!(
            tile_grid.get(&GridCoordinates { x: 3, y: 0 }),
            Some(&TileType::Coin(CoinValue::Two))
        );
    }

    #[test]
    fn should_update_unused_coordinates() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(GridCoordinates { x: 0, y: 0 }, TileType::Wall);

        assert!(!tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 0, y: 0 }));
        assert!(tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 1, y: 0 }));

        // Deletes the tile at 0,0
        tile_grid.maybe_insert(GridCoordinates { x: 0, y: 0 }, None);
        assert!(tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 0, y: 0 }));

        // Inserts a tile at 1,0
        tile_grid.maybe_insert(GridCoordinates { x: 1, y: 0 }, Some(TileType::Bomb));
        assert!(!tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 1, y: 0 }));
    }

    #[test]
    fn unused_coordinates_sync_after_move_event() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(GridCoordinates { x: 1, y: 0 }, TileType::Bomb);
        tile_grid.insert(
            GridCoordinates { x: 2, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        // Moves tile 1,0 -> 2,0
        let events = vec![
            MoveTileEvent {
                source: GridCoordinates { x: 1, y: 0 },
                target: GridCoordinates { x: 2, y: 0 },
            },
            MoveTileEvent {
                source: GridCoordinates { x: 2, y: 0 },
                target: GridCoordinates { x: 3, y: 0 },
            },
        ];
        tile_grid.handle_move_tile_events(events.iter());
        assert!(tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 1, y: 0 }));
        assert!(!tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 2, y: 0 }));
        assert!(!tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 3, y: 0 }));
    }

    #[test]
    fn unused_coordinates_sync_after_combine_event() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 2, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        // Combine tiles 1,0 and 2,0 in 2,0
        let events = vec![MergeTilesEvent {
            source: GridCoordinates { x: 1, y: 0 },
            target: GridCoordinates { x: 2, y: 0 },
            resulting_type: Some(TileType::Coin(CoinValue::Two)),
        }];
        tile_grid.handle_combine_events(events.iter());
        assert!(tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 1, y: 0 }));
        assert!(!tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 2, y: 0 }));
    }

    #[test]
    fn unused_coordinates_sync_after_explosion_event() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(
            GridCoordinates { x: 1, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 2, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 3, y: 0 },
            TileType::Coin(CoinValue::One),
        );
        tile_grid.insert(
            GridCoordinates { x: 4, y: 0 },
            TileType::Coin(CoinValue::One),
        );

        // Combine tiles 1,0 and 2,0 in 2,0
        let events = vec![ExplosionEvent {
            target: GridCoordinates { x: 2, y: 0 },
        }];
        tile_grid.handle_explosion_events(events.iter());
        assert!(tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 1, y: 0 }));
        assert!(tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 2, y: 0 }));
        assert!(tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 3, y: 0 }));
        assert!(!tile_grid
            .unused_coordinates
            .contains(&GridCoordinates { x: 4, y: 0 }));
    }
}
