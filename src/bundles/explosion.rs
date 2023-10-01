use std::time::Duration;

use bevy::{
    prelude::{Bundle, Transform, Vec3},
    sprite::{SpriteSheetBundle, TextureAtlasSprite},
    time::{Timer, TimerMode},
};

use crate::{
    assets::GameAssets,
    constants::TILE_SIZE,
    game::grid::GridCoordinates,
    systems::explosion::{Explosion, ExplosionAnimation},
};

#[derive(Bundle)]
pub struct ExplosionBundle {
    sprite_sheet: SpriteSheetBundle,
    explosion: Explosion,
    explosion_anim: ExplosionAnimation,
}

const ANIMATION_DURATION: f32 = 0.5;
const ANIMATION_FRAMES: f32 = 6.;

impl ExplosionBundle {
    pub fn new(assets: &GameAssets, grid_coordinates: GridCoordinates) -> Self {
        ExplosionBundle {
            sprite_sheet: SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: assets.explosion.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        grid_coordinates.x as f32 * TILE_SIZE,
                        grid_coordinates.y as f32 * TILE_SIZE,
                        2.,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            },
            explosion: Explosion(Timer::new(
                Duration::from_secs_f32(ANIMATION_DURATION),
                TimerMode::Once,
            )),
            explosion_anim: ExplosionAnimation(Timer::new(
                Duration::from_secs_f32(ANIMATION_DURATION / ANIMATION_FRAMES),
                TimerMode::Once,
            )),
        }
    }
}
