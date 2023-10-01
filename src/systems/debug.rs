use bevy::prelude::{Commands, Res};

use crate::{
    assets::GameAssets,
    bundles::tile::spawn_tile_type_bundle,
    game::tile::{CoinValue, TileType},
};

pub fn setup_debug(mut commands: Commands, assets: Res<GameAssets>) {
    spawn_tile_type_bundle(
        &mut commands,
        assets.tileset.clone(),
        TileType::Coin(CoinValue::One),
        0,
        0,
    );
    spawn_tile_type_bundle(
        &mut commands,
        assets.tileset.clone(),
        TileType::Coin(CoinValue::One),
        4,
        0,
    );
    spawn_tile_type_bundle(
        &mut commands,
        assets.tileset.clone(),
        TileType::Coin(CoinValue::Two),
        4,
        1,
    );
    spawn_tile_type_bundle(
        &mut commands,
        assets.tileset.clone(),
        TileType::Coin(CoinValue::Four),
        4,
        2,
    );
    spawn_tile_type_bundle(
        &mut commands,
        assets.tileset.clone(),
        TileType::Coin(CoinValue::Eight),
        4,
        3,
    );
    spawn_tile_type_bundle(&mut commands, assets.tileset.clone(), TileType::Bomb, 1, 3);
    spawn_tile_type_bundle(&mut commands, assets.tileset.clone(), TileType::Bomb, 1, 4);
}
