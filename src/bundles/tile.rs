use crate::{
    constants::TILE_SIZE,
    game::{
        grid::GridCoordinates,
        tile::{CoinValue, TileType},
    },
    systems::{self},
};
use bevy::{
    prelude::{Bundle, Commands, Handle, Transform, Vec2, Vec3},
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};
use bevy_mod_picking::PickableBundle;

#[derive(Bundle, Default)]
pub struct TileBundle {
    pub sprite: SpriteSheetBundle,
    pub grid_coords: GridCoordinates,
}

pub fn spawn_tile_type_bundle(
    commands: &mut Commands,
    tileset: Handle<TextureAtlas>,
    tile_type: TileType,
    x: i32,
    y: i32,
) {
    match tile_type {
        TileType::Coin(value) => spawn_coin(commands, tileset, x, y, value),
        TileType::Wall => spawn_wall(commands, tileset, x, y),
        TileType::Bomb => spawn_bomb(commands, tileset, x, y),
    }
}

fn spawn_coin(
    commands: &mut Commands,
    tileset: Handle<TextureAtlas>,
    x: i32,
    y: i32,
    value: CoinValue,
) {
    let index = match value {
        CoinValue::One => 1,
        CoinValue::Two => 5,
        CoinValue::Four => 9,
        CoinValue::Eight => 13,
    };

    commands.spawn((
        TileBundle {
            sprite: SpriteSheetBundle {
                texture_atlas: tileset,
                sprite: TextureAtlasSprite {
                    index,
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

fn spawn_bomb(commands: &mut Commands, tileset: Handle<TextureAtlas>, x: i32, y: i32) {
    commands.spawn((
        TileBundle {
            sprite: SpriteSheetBundle {
                texture_atlas: tileset,
                sprite: TextureAtlasSprite {
                    index: 2,
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
        PickableBundle::default(),
        systems::movables::on_pointer_drag_end_handler(),
        TileType::Bomb,
    ));
}

fn spawn_wall(commands: &mut Commands, tileset: Handle<TextureAtlas>, x: i32, y: i32) {
    commands.spawn((
        TileBundle {
            sprite: SpriteSheetBundle {
                texture_atlas: tileset,
                sprite: TextureAtlasSprite {
                    index: 4,
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
