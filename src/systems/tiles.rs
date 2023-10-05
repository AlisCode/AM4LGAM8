use std::time::Duration;

use bevy::{
    prelude::{Commands, Entity, EventReader, EventWriter, Query, Res, ResMut, Transform},
    time::{Timer, TimerMode},
    utils::{HashMap, HashSet},
};

use crate::{
    assets::GameAssets,
    bundles::{explosion::ExplosionBundle, tile::spawn_tile_type_bundle},
    game::{
        grid::{GridCoordinates, MoveTileEvent, TileGrid},
        moves::{ExplosionEvent, MergeTilesEvent, ValidEvent, ValidatedEventQueue},
        tile::{ExplosionResult, TileType},
    },
};

use super::{
    animations::{add_movement_animation, AndDeleteAfter},
    grid::ValidTurnEvent,
    marked_for_deletion::MarkedForDeletion,
    movables::RequestMoveEvent,
    ui::GameScore,
    OnPlayingScreen,
};

/// Validates incoming RequestMoveEvent into ValidMoveEvent
pub fn handle_requested_move_events(
    mut requested_event_rx: EventReader<RequestMoveEvent>,
    mut move_tile_event_tx: EventWriter<MoveTileEvent>,
    mut combine_event_tx: EventWriter<MergeTilesEvent>,
    mut explosion_event_tx: EventWriter<ExplosionEvent>,
    mut valid_turn_tx: EventWriter<ValidTurnEvent>,
    mut tile_grid: ResMut<TileGrid>,
) {
    for move_event in requested_event_rx.iter() {
        let RequestMoveEvent {
            move_direction,
            source_coords,
        } = move_event;

        let candidate_coords = source_coords.candidate_coords_for_dir(*move_direction);

        let validated_event_queue =
            ValidatedEventQueue::validate_move(&tile_grid, candidate_coords, *move_direction);

        match validated_event_queue {
            ValidatedEventQueue::InvalidMove => (),
            ValidatedEventQueue::ValidMove(events) => {
                tile_grid.apply_events(&events);
                for event in events {
                    match event {
                        ValidEvent::Move(e) => move_tile_event_tx.send(e),
                        ValidEvent::Merge(e) => combine_event_tx.send(e),
                        ValidEvent::Explosions(e) => explosion_event_tx.send(e),
                    }
                }
                valid_turn_tx.send(ValidTurnEvent);
            }
        }
    }
}

pub fn handle_valid_move_events(
    mut commands: Commands,
    mut move_tile_event_rx: EventReader<MoveTileEvent>,
    mut query: Query<(Entity, &mut GridCoordinates, &Transform, &TileType)>,
) {
    let mut old_coords_to_new_coords: HashMap<GridCoordinates, GridCoordinates> =
        move_tile_event_rx
            .iter()
            .cloned()
            .map(|MoveTileEvent { source, target }| (source, target))
            .collect();
    for (entity, mut coords, transform, _tile_type) in query.iter_mut() {
        if let Some(new_coords) = old_coords_to_new_coords.remove(&*coords) {
            *coords = new_coords;

            add_movement_animation(
                &mut commands,
                entity,
                transform,
                coords.clone(),
                AndDeleteAfter::No,
            );
        }
    }
}

pub fn handle_combine_events(
    mut commands: Commands,
    mut combine_event_rx: EventReader<MergeTilesEvent>,
    query: Query<(Entity, &GridCoordinates, &Transform)>,
    assets: Res<GameAssets>,
) {
    for event in combine_event_rx.iter() {
        let MergeTilesEvent {
            source,
            target,
            resulting_type,
        } = event;

        for (entity, grid_coords, transform) in query.iter() {
            if grid_coords == source {
                add_movement_animation(
                    &mut commands,
                    entity,
                    transform,
                    target.clone(),
                    AndDeleteAfter::Yes,
                );
            }
            if grid_coords == target {
                commands.entity(entity).insert(MarkedForDeletion(Timer::new(
                    Duration::from_secs_f32(0.1),
                    TimerMode::Once,
                )));
            }
        }

        if let Some(tile_type) = resulting_type {
            spawn_tile_type_bundle(
                &mut commands,
                assets.tileset.clone(),
                *tile_type,
                target.x,
                target.y,
            );
        }
    }
}

pub fn handle_explosion_events(
    mut commands: Commands,
    mut explosion_event_rx: EventReader<ExplosionEvent>,
    query: Query<(Entity, &GridCoordinates, &TileType)>,
    mut game_score: ResMut<GameScore>,
    assets: Res<GameAssets>,
) {
    let mut grid_coords_to_delete: HashSet<GridCoordinates> = HashSet::default();
    let mut events = Vec::default();
    for event in explosion_event_rx.iter() {
        events.push(event.clone());
        let ExplosionEvent { target } = event;
        grid_coords_to_delete.extend(target.explosion_radius());
    }

    let assets = &*assets;
    for (entity, coords, tile_type) in query.iter() {
        if !grid_coords_to_delete.contains(coords) {
            continue;
        }

        match tile_type.explosion_result() {
            ExplosionResult::NoExplosion => continue,
            ExplosionResult::ScorePoints(points) => {
                game_score.add(points);
                commands.spawn((
                    ExplosionBundle::new(&assets, coords.clone()),
                    OnPlayingScreen,
                ));
                commands.entity(entity).insert(MarkedForDeletion(Timer::new(
                    Duration::from_secs_f32(0.1),
                    TimerMode::Once,
                )));
            }
        }
    }
}
