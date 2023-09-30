use bevy::prelude::Component;

#[derive(Default, Component)]
pub struct GridCoordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
#[allow(dead_code)]
pub enum TileType {
    Coin,
    Bomb,
    Wall,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Coin
    }
}
