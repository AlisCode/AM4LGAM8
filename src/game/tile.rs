use bevy::prelude::Component;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum CoinValue {
    One,
    Two,
    Four,
    Eight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum TileType {
    Coin(CoinValue),
    Wall,
    // Bomb,
}

impl TileType {
    pub fn is_movable(&self) -> bool {
        match self {
            TileType::Wall => false,
            TileType::Coin(_) => true,
        }
    }

    pub fn can_combine_with(&self, other: &TileType) -> bool {
        match (self, other) {
            (TileType::Coin(CoinValue::One), TileType::Coin(CoinValue::One)) => true,
            (TileType::Coin(CoinValue::Two), TileType::Coin(CoinValue::Two)) => true,
            (TileType::Coin(CoinValue::Four), TileType::Coin(CoinValue::Four)) => true,
            (TileType::Coin(CoinValue::Eight), TileType::Coin(CoinValue::Eight)) => true,
            (TileType::Coin(_), TileType::Coin(_) | TileType::Wall) => false,
            (TileType::Wall, TileType::Coin(_) | TileType::Wall) => false,
        }
    }
}
