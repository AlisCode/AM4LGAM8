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

#[derive(Debug, PartialEq, Eq)]
pub enum CombinationResult {
    MergeTilesInto(TileType),
}

impl TileType {
    pub fn is_movable(&self) -> bool {
        match self {
            TileType::Wall => false,
            TileType::Coin(_) => true,
        }
    }

    pub fn try_combine_with(&self, other: &TileType) -> Option<CombinationResult> {
        match (self, other) {
            (TileType::Coin(CoinValue::One), TileType::Coin(CoinValue::One)) => Some(
                CombinationResult::MergeTilesInto(TileType::Coin(CoinValue::Two)),
            ),
            (TileType::Coin(CoinValue::Two), TileType::Coin(CoinValue::Two)) => Some(
                CombinationResult::MergeTilesInto(TileType::Coin(CoinValue::Four)),
            ),
            (TileType::Coin(CoinValue::Four), TileType::Coin(CoinValue::Four)) => Some(
                CombinationResult::MergeTilesInto(TileType::Coin(CoinValue::Eight)),
            ),
            (TileType::Coin(CoinValue::Eight), TileType::Coin(CoinValue::Eight)) => None,
            (TileType::Coin(_), TileType::Coin(_) | TileType::Wall) => None,
            (TileType::Wall, TileType::Coin(_) | TileType::Wall) => None,
        }
    }
}
