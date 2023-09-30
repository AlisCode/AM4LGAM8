use bevy::prelude::Commands;

use crate::bundles::tile::TileBundle;

pub fn setup_debug(mut commands: Commands) {
    commands.spawn(TileBundle::coin());
}
