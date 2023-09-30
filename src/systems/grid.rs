use bevy::prelude::{Added, Commands, EventReader, Query, ResMut};

use crate::{
    bundles::tile::spawn_tile_type_bundle,
    constants::GRID_SIZE,
    game::{
        grid::{GridCoordinates, MoveTileEvent, TileGrid},
        moves::CombineEvent,
        tile::TileType,
    },
};

pub fn setup_grid(mut commands: Commands) {
    for x in -1..=GRID_SIZE + 1 {
        spawn_tile_type_bundle(&mut commands, TileType::Wall, x, -1);
        spawn_tile_type_bundle(&mut commands, TileType::Wall, x, GRID_SIZE + 1);
    }
    for y in -1..=GRID_SIZE + 1 {
        spawn_tile_type_bundle(&mut commands, TileType::Wall, -1, y);
        spawn_tile_type_bundle(&mut commands, TileType::Wall, GRID_SIZE + 1, y);
    }
}

pub fn sync_tile_grid(
    mut tile_grid: ResMut<TileGrid>,
    newly_spawned_tiles: Query<(&TileType, &GridCoordinates), Added<GridCoordinates>>,
    mut move_tile_event_rx: EventReader<MoveTileEvent>,
    mut combine_event_rx: EventReader<CombineEvent>,
) {
    tile_grid.handle_move_tile_events(move_tile_event_rx.iter());
    tile_grid.handle_combine_events(combine_event_rx.iter());
    for (tile_type, coords) in newly_spawned_tiles.iter() {
        tile_grid.insert(coords.clone(), *tile_type);
    }
}
