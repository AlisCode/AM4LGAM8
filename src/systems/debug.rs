use bevy::prelude::Commands;

use crate::{
    bundles::tile::spawn_tile_type_bundle,
    game::tile::{CoinValue, TileType},
};

pub fn setup_debug(mut commands: Commands) {
    spawn_tile_type_bundle(&mut commands, TileType::Coin(CoinValue::One), 0, 0);
    spawn_tile_type_bundle(&mut commands, TileType::Coin(CoinValue::Two), 4, 0);
}
