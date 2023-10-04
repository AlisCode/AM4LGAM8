use bevy::prelude::{
    Commands, DespawnRecursive, Entity, Event, EventReader, EventWriter, NextState, Query, Res,
    ResMut,
};

use crate::{
    assets::GameAssets,
    bundles::tile::spawn_tile_type_bundle,
    core::GameState,
    game::grid::{SpawnEvent, TileGrid},
};

use super::OnPlayingScreen;

pub fn setup_grid(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut tile_grid: ResMut<TileGrid>,
) {
    let spawn_events = tile_grid.setup_default_grid();

    for SpawnEvent { coords, tile_type } in spawn_events {
        spawn_tile_type_bundle(
            &mut commands,
            assets.tileset.clone(),
            tile_type,
            coords.x,
            coords.y,
        );
    }
}

pub fn check_for_game_over(
    mut commands: Commands,
    all_entities_on_screen: Query<(Entity, &OnPlayingScreen)>,
    mut tile_grid: ResMut<TileGrid>,
    mut next_state: ResMut<NextState<GameState>>,
    mut valid_turn_event_tx: EventWriter<ValidTurnEvent>,
) {
    // Check for game over
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
    mut tile_grid: ResMut<TileGrid>,
    mut valid_turn_event_rx: EventReader<ValidTurnEvent>,
) {
    for _ in valid_turn_event_rx.iter() {
        let maybe_spawn_event = tile_grid.try_spawn_new_tile();

        match maybe_spawn_event {
            Some(SpawnEvent { coords, tile_type }) => {
                spawn_tile_type_bundle(
                    &mut commands,
                    game_assets.tileset.clone(),
                    tile_type,
                    coords.x,
                    coords.y,
                );
            }
            None => panic!("No coordinates to spawn a tile on. This is a bug."),
        }
    }
}

pub fn spawn_first_tile(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut tile_grid: ResMut<TileGrid>,
) {
    let SpawnEvent { coords, tile_type } = tile_grid
        .spawn_first_tile()
        .expect("Failed to spawn first tile. This is a bug.");
    spawn_tile_type_bundle(
        &mut commands,
        game_assets.tileset.clone(),
        tile_type,
        coords.x,
        coords.y,
    );
}
