use bevy::prelude::{
    Added, Commands, DespawnRecursive, Entity, Event, EventReader, EventWriter, NextState, Query,
    Res, ResMut,
};

use crate::{
    assets::GameAssets,
    bundles::tile::spawn_tile_type_bundle,
    constants::GRID_SIZE,
    core::GameState,
    game::{
        grid::{GridCoordinates, TileGrid},
        tile::TileType,
    },
};

use super::OnPlayingScreen;

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
    mut commands: Commands,
    all_entities_on_screen: Query<(Entity, &OnPlayingScreen)>,
    mut tile_grid: ResMut<TileGrid>,
    newly_spawned_tiles: Query<(&TileType, &GridCoordinates), Added<GridCoordinates>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (tile_type, coords) in newly_spawned_tiles.iter() {
        tile_grid.insert(coords.clone(), *tile_type);
    }
    if !tile_grid.has_any_possible_moves() {
        for (entity, _on_screen) in all_entities_on_screen.iter() {
            commands.add(DespawnRecursive { entity });
        }
        next_state.set(GameState::GameOver);
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
            None => panic!("No coordinates to spawn a tile on. This is bug."),
        }
    }
}

pub fn spawn_first_tile(mut valid_turn_event_tx: EventWriter<ValidTurnEvent>) {
    valid_turn_event_tx.send(ValidTurnEvent);
}
