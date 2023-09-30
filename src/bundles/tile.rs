use bevy::{
    prelude::{Bundle, Color, Rect},
    sprite::{Sprite, SpriteBundle},
};

use crate::systems::grid::{GridCoordinates, TileType};

const TILE_SIZE: u32 = 16;

#[derive(Bundle, Default)]
pub struct TileBundle {
    pub sprite: SpriteBundle,
    pub grid_coords: GridCoordinates,
    pub tile_type: TileType,
}

impl TileBundle {
    pub fn coin() -> Self {
        TileBundle {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::YELLOW,
                    rect: Some(Rect::new(0., 0., TILE_SIZE as f32, TILE_SIZE as f32)),
                    ..Default::default()
                },
                ..Default::default()
            },
            grid_coords: GridCoordinates::default(),
            tile_type: TileType::Coin,
        }
    }
}
