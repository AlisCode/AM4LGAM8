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
        tile::{CoinValue, TileType},
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
    mut valid_turn_event_tx: EventWriter<ValidTurnEvent>,
) {
    for (tile_type, coords) in newly_spawned_tiles.iter() {
        tile_grid.insert(coords.clone(), *tile_type);
    }

    // It is game over
    if !tile_grid.has_any_possible_moves() {
        // Unless we still do have unused coordinates, in which case we can trigger the spawn of a
        // new tile
        if tile_grid.has_unused_coordinates() {
            valid_turn_event_tx.send(ValidTurnEvent);
            return;
        }

        for (entity, _on_screen) in all_entities_on_screen.iter() {
            commands.add(DespawnRecursive { entity });
        }
        next_state.set(GameState::GameOver);
        *tile_grid = TileGrid::default();
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

pub fn spawn_first_tile(mut commands: Commands, game_assets: Res<GameAssets>) {
    spawn_tile_type_bundle(
        &mut commands,
        game_assets.tileset.clone(),
        TileType::Coin(CoinValue::One),
        1,
        1,
    );
}
