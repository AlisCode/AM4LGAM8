use std::time::Duration;

use crate::{
    constants::TILE_SIZE,
    game::{
        grid::GridCoordinates,
        tile::{CoinValue, TileType},
    },
    systems::{self, OnPlayingScreen},
};
use bevy::{
    prelude::{Bundle, Commands, Handle, Transform, Vec2, Vec3},
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};
use bevy_easings::{Ease, EaseFunction, EaseMethod, EasingComponent};
use bevy_mod_picking::PickableBundle;

#[derive(Bundle)]
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

fn gen_transform_and_easing(x: i32, y: i32) -> (Transform, EasingComponent<Transform>) {
    let transform = Transform {
        translation: Vec3::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.),
        scale: Vec3::new(0., 0., 1.),
        ..Default::default()
    };
    let easing = transform.ease_to(
        Transform {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: Vec3::ONE,
        },
        EaseMethod::EaseFunction(EaseFunction::CubicOut),
        bevy_easings::EasingType::Once {
            duration: Duration::from_secs_f32(0.4),
        },
    );
    (transform, easing)
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

    let (transform, easing) = gen_transform_and_easing(x, y);
    commands.spawn((
        TileBundle {
            sprite: SpriteSheetBundle {
                texture_atlas: tileset,
                sprite: TextureAtlasSprite {
                    index,
                    custom_size: Some(Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32)),
                    ..Default::default()
                },
                transform,
                ..Default::default()
            },
            grid_coords: GridCoordinates { x, y },
        },
        easing,
        TileType::Coin(value),
        PickableBundle::default(),
        systems::movables::on_pointer_drag_end_handler(),
        OnPlayingScreen,
    ));
}

fn spawn_bomb(commands: &mut Commands, tileset: Handle<TextureAtlas>, x: i32, y: i32) {
    let (transform, easing) = gen_transform_and_easing(x, y);
    commands.spawn((
        TileBundle {
            sprite: SpriteSheetBundle {
                texture_atlas: tileset,
                sprite: TextureAtlasSprite {
                    index: 2,
                    custom_size: Some(Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32)),
                    ..Default::default()
                },
                transform,
                ..Default::default()
            },
            grid_coords: GridCoordinates { x, y },
        },
        easing,
        PickableBundle::default(),
        systems::movables::on_pointer_drag_end_handler(),
        TileType::Bomb,
        OnPlayingScreen,
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
                    translation: Vec3::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.),
                    ..Default::default()
                },
                ..Default::default()
            },
            grid_coords: GridCoordinates { x, y },
        },
        TileType::Wall,
        OnPlayingScreen,
    ));
}
