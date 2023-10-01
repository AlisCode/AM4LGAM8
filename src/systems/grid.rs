use bevy::prelude::{Added, Commands, Event, EventReader, Query, Res, ResMut};

use crate::{
    assets::GameAssets,
    bundles::tile::spawn_tile_type_bundle,
    constants::GRID_SIZE,
    game::{
        grid::{GridCoordinates, TileGrid},
        tile::TileType,
    },
};

pub fn setup_grid(mut commands: Commands, assets: Res<GameAssets>) {
    for x in -1..=GRID_SIZE {
        spawn_tile_type_bundle(&mut commands, assets.tileset.clone(), TileType::Wall, x, -1);
        spawn_tile_type_bundle(
            &mut commands,
            assets.tileset.clone(),
            TileType::Wall,
            x,
            GRID_SIZE,
        );
    }
    for y in -1..=GRID_SIZE {
        spawn_tile_type_bundle(&mut commands, assets.tileset.clone(), TileType::Wall, -1, y);
        spawn_tile_type_bundle(
            &mut commands,
            assets.tileset.clone(),
            TileType::Wall,
            GRID_SIZE,
            y,
        );
    }
}

pub fn sync_tile_grid(
    mut tile_grid: ResMut<TileGrid>,
    newly_spawned_tiles: Query<(&TileType, &GridCoordinates), Added<GridCoordinates>>,
) {
    for (tile_type, coords) in newly_spawned_tiles.iter() {
        tile_grid.insert(coords.clone(), *tile_type);
    }
}

#[derive(Debug, PartialEq, Eq, Event)]
pub struct ValidTurnEvent;

pub fn spawn_new_tile_on_valid_move(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    tile_grid: Res<TileGrid>,
    mut valid_turn_event_rx: EventReader<ValidTurnEvent>,
) {
    // TODO: Player has played a valid turn
    // * Play sound
    // * Gen new tile
    // * Check for game over

    for _ in valid_turn_event_rx.iter() {
        let maybe_unused_coordinate = tile_grid.get_unused_coordinate();

        match maybe_unused_coordinate {
            Some(unused_coord) => {
                spawn_tile_type_bundle(
                    &mut commands,
                    game_assets.tileset.clone(),
                    TileType::gen_random(),
                    unused_coord.x,
                    unused_coord.y,
                );
            }
            // TODO: Game over! handle
            None => (),
        }
    }
}
