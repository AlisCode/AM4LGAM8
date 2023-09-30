use bevy::prelude::{Commands, Component, Event};

use crate::bundles::tile::spawn_tile_type_bundle;

use super::tiles::TileType;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
pub struct GridCoordinates {
    pub x: i32,
    pub y: i32,
}

pub const GRID_SIZE: i32 = 7;

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

#[derive(Debug, Event)]
pub struct MoveTileEvent {
    pub source: GridCoordinates,
    pub target: GridCoordinates,
}
