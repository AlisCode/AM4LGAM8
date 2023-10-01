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
    systems::{explosion::ExplosionAnimation, marked_for_deletion::MarkedForDeletion},
};

#[derive(Bundle)]
pub struct ExplosionBundle {
    sprite_sheet: SpriteSheetBundle,
    deletion_marker: MarkedForDeletion,
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
            deletion_marker: MarkedForDeletion(Timer::new(
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
