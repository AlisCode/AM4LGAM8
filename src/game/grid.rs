use bevy::{
    prelude::{Component, Event, Resource},
    utils::{HashMap, HashSet},
};
use rand::Rng;

use crate::{constants::GRID_SIZE, game::moves::ValidatedEventQueue};

use super::{
    moves::{
        CanCombineResult, CanMoveResult, ExplosionEvent, MergeTilesEvent, MoveDirection, ValidEvent,
    },
    tile::{CoinValue, ExplosionResult, TileType},
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

    pub fn candidate_coords_for_dir(&self, move_direction: MoveDirection) -> Vec<GridCoordinates> {
        match move_direction {
            MoveDirection::Left => (-1..=self.x)
                .rev()
                .map(|x| GridCoordinates { x, y: self.y })
                .collect(),
            MoveDirection::Right => (self.x..=GRID_SIZE)
                .map(|x| GridCoordinates { x, y: self.y })
                .collect(),
            MoveDirection::Up => (self.y..=GRID_SIZE)
                .map(|y| GridCoordinates { x: self.x, y })
                .collect(),
            MoveDirection::Down => (-1..=self.y)
                .rev()
                .map(|y| GridCoordinates { x: self.x, y })
                .collect(),
        }
    }
}

// Events

#[derive(Debug, PartialEq, Eq, Clone, Event)]
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

    pub fn apply_events(&mut self, events: &Vec<ValidEvent>) {
        for event in events {
            match event {
                ValidEvent::Move(e) => self.handle_move_tile_event(e),
                ValidEvent::Merge(e) => self.handle_combine_event(e),
                ValidEvent::Explosions(e) => self.handle_explosion_event(e),
            }
        }
    }

    pub fn handle_move_tile_event(&mut self, event: &MoveTileEvent) {
        if let Some(tile_type) = self.grid.remove(&event.source) {
            self.insert(event.target.clone(), tile_type);
        }
        self.unused_coordinates.insert(event.source.clone());
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

    pub fn handle_combine_event(&mut self, event: &MergeTilesEvent) {
        let MergeTilesEvent {
            source,
            target,
            resulting_type,
        } = event;
        self.grid.remove(source);
        self.grid.remove(target);
        self.unused_coordinates.insert(source.clone());
        self.unused_coordinates.insert(target.clone());
        self.maybe_insert(target.clone(), *resulting_type);
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

    pub fn handle_explosion_event(&mut self, event: &ExplosionEvent) {
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

    fn get_unused_coordinate(&self) -> Option<GridCoordinates> {
        // Quick and hacky way to get an element from a set
        // I don't have a better idea rn
        if self.unused_coordinates.is_empty() {
            return None;
        }
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

    pub fn has_any_possible_moves(&self) -> bool {
        (0..=GRID_SIZE)
            .flat_map(move |x| (0..=GRID_SIZE).map(move |y| GridCoordinates { x, y }))
            .any(|coords| {
                self.has_possible_moves_for_direction(&coords, MoveDirection::Left)
                    || self.has_possible_moves_for_direction(&coords, MoveDirection::Right)
                    || self.has_possible_moves_for_direction(&coords, MoveDirection::Up)
                    || self.has_possible_moves_for_direction(&coords, MoveDirection::Down)
            })
    }

    fn has_possible_moves_for_direction(
        &self,
        coords: &GridCoordinates,
        move_direction: MoveDirection,
    ) -> bool {
        let candidate_coords = coords.candidate_coords_for_dir(move_direction);
        match ValidatedEventQueue::validate_move(self, candidate_coords, move_direction) {
            ValidatedEventQueue::ValidMove(_) => true,
            ValidatedEventQueue::InvalidMove => false,
        }
    }

    pub fn has_unused_coordinates(&self) -> bool {
        !self.unused_coordinates.is_empty()
    }

    pub fn try_spawn_new_tile(&mut self) -> Option<SpawnEvent> {
        let coords = self.get_unused_coordinate()?;
        let tile_type = TileType::gen_random();
        self.insert(coords.clone(), tile_type);
        Some(SpawnEvent { coords, tile_type })
    }

    pub fn spawn_first_tile(&mut self) -> Option<SpawnEvent> {
        let coords = self.get_unused_coordinate()?;
        let tile_type = TileType::Coin(CoinValue::One);
        self.insert(coords.clone(), tile_type);
        Some(SpawnEvent { coords, tile_type })
    }

    pub fn setup_default_grid(&mut self) -> Vec<SpawnEvent> {
        let mut spawn_events = Vec::default();
        for x in -1..=GRID_SIZE {
            let tile_type = TileType::Wall;
            let coords_bottom = GridCoordinates { x, y: -1 };
            self.insert(coords_bottom.clone(), tile_type);
            spawn_events.push(SpawnEvent {
                coords: coords_bottom,
                tile_type,
            });

            let coords_top = GridCoordinates { x, y: GRID_SIZE };
            self.insert(coords_top.clone(), tile_type);
            spawn_events.push(SpawnEvent {
                coords: coords_top,
                tile_type,
            });
        }
        for y in -1..=GRID_SIZE {
            let tile_type = TileType::Wall;
            let coords_left = GridCoordinates { x: -1, y };
            self.insert(coords_left.clone(), tile_type);
            spawn_events.push(SpawnEvent {
                coords: coords_left,
                tile_type,
            });

            let coords_right = GridCoordinates { x: GRID_SIZE, y };
            self.insert(coords_right.clone(), tile_type);
            spawn_events.push(SpawnEvent {
                coords: coords_right,
                tile_type,
            });
        }

        spawn_events
    }
}

pub struct SpawnEvent {
    pub coords: GridCoordinates,
    pub tile_type: TileType,
}

#[cfg(test)]
pub mod tests {
    use crate::{
        constants::GRID_SIZE,
        game::{
            grid::{GridCoordinates, MoveTileEvent, TileGrid},
            moves::{CanMoveResult, ExplosionEvent, MergeTilesEvent, MoveDirection},
            tile::{CoinValue, TileType},
        },
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

        tile_grid.handle_explosion_event(&ExplosionEvent {
            target: GridCoordinates { x: 0, y: 0 },
        });

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
        tile_grid.handle_combine_event(&MergeTilesEvent {
            source: GridCoordinates { x: 1, y: 0 },
            target: GridCoordinates { x: 2, y: 0 },
            resulting_type: Some(TileType::Coin(CoinValue::Two)),
        });
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
        tile_grid.handle_explosion_event(&ExplosionEvent {
            target: GridCoordinates { x: 2, y: 0 },
        });
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

    #[test]
    fn has_any_possible_moves_should_work() {
        let mut tile_grid = TileGrid::default();
        tile_grid.insert(GridCoordinates { x: 0, y: 0 }, TileType::Bomb);

        assert!(tile_grid.has_any_possible_moves());

        for coord in (-1..=GRID_SIZE)
            .flat_map(move |x| (-1..GRID_SIZE).map(move |y| GridCoordinates { x, y }))
        {
            tile_grid.insert(coord, TileType::Wall);
        }
        tile_grid.insert(GridCoordinates { x: 0, y: 0 }, TileType::Bomb);

        assert!(!tile_grid.has_any_possible_moves());
    }
}
