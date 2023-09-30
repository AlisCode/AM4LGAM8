use crate::{
    constants::TILE_SIZE,
    systems::{
        self,
        grid::GridCoordinates,
        tiles::{CoinValue, TileType},
    },
};
use bevy::{
    prelude::{Bundle, Color, Commands, Rect, Transform, Vec2, Vec3},
    sprite::{Sprite, SpriteBundle},
};
use bevy_mod_picking::PickableBundle;

#[derive(Bundle, Default)]
pub struct TileBundle {
    pub sprite: SpriteBundle,
    pub grid_coords: GridCoordinates,
}

pub fn spawn_tile_type_bundle(commands: &mut Commands, tile_type: TileType, x: i32, y: i32) {
    match tile_type {
        TileType::Coin(value) => spawn_coin(commands, x, y, value),
        TileType::Wall => spawn_wall(commands, x, y),
    }
}

fn spawn_coin(commands: &mut Commands, x: i32, y: i32, value: CoinValue) {
    let color = match value {
        CoinValue::One => Color::LIME_GREEN,
        CoinValue::Two => Color::GREEN,
        CoinValue::Four => Color::YELLOW_GREEN,
        CoinValue::Eight => Color::YELLOW,
    };

    commands.spawn((
        TileBundle {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color,
                    rect: Some(Rect::new(0., 0., TILE_SIZE as f32, TILE_SIZE as f32)),
                    custom_size: Some(Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 1.),
                    ..Default::default()
                },
                ..Default::default()
            },
            grid_coords: GridCoordinates { x, y },
        },
        TileType::Coin(value),
        PickableBundle::default(),
        systems::movables::on_pointer_drag_end_handler(),
    ));
}

fn spawn_wall(commands: &mut Commands, x: i32, y: i32) {
    commands.spawn((
        TileBundle {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    rect: Some(Rect::new(0., 0., TILE_SIZE as f32, TILE_SIZE as f32)),
                    custom_size: Some(Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 1.),
                    ..Default::default()
                },
                ..Default::default()
            },
            grid_coords: GridCoordinates { x, y },
        },
        TileType::Wall,
    ));
}
