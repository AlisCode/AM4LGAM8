use bevy::prelude::Component;
use rand::Rng;

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
    Bomb,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CombinationResult {
    MergeTilesInto(TileType),
    Explosion,
}

pub enum ExplosionResult {
    ScorePoints(i32),
    NoExplosion,
}

impl TileType {
    pub fn is_movable(&self) -> bool {
        match self {
            TileType::Wall => false,
            TileType::Coin(_) | TileType::Bomb => true,
        }
    }

    pub fn explosion_result(&self) -> ExplosionResult {
        match self {
            TileType::Wall => ExplosionResult::NoExplosion,
            TileType::Coin(value) => match value {
                CoinValue::One => ExplosionResult::ScorePoints(1),
                CoinValue::Two => ExplosionResult::ScorePoints(2),
                CoinValue::Four => ExplosionResult::ScorePoints(4),
                CoinValue::Eight => ExplosionResult::ScorePoints(8),
            },
            TileType::Bomb => ExplosionResult::ScorePoints(1),
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
            (TileType::Coin(_), TileType::Coin(_) | TileType::Wall | TileType::Bomb) => None,
            (TileType::Bomb, TileType::Bomb) => Some(CombinationResult::Explosion),
            (TileType::Bomb, TileType::Coin(_) | TileType::Wall) => None,
            (TileType::Wall, TileType::Coin(_) | TileType::Wall | TileType::Bomb) => None,
        }
    }

    pub fn gen_random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..=10) {
            0..=4 => TileType::Coin(CoinValue::One),
            5..=6 => TileType::Coin(CoinValue::Two),
            7..=9 => TileType::Bomb,
            10..=10 => TileType::Wall,
            _ => panic!("range 0..=10"),
        }
    }
}
