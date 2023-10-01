use std::time::Duration;

use bevy::{
    prelude::{Commands, Entity, Transform, Vec3},
    time::{Timer, TimerMode},
};
use bevy_easings::{Ease, EaseMethod};

use crate::{constants::TILE_SIZE, game::grid::GridCoordinates};

use super::marked_for_deletion::MarkedForDeletion;

pub enum AndDeleteAfter {
    Yes,
    No,
}

const MOVEMENT_ANIMATION: f32 = 0.15;

pub fn add_movement_animation(
    commands: &mut Commands,
    entity: Entity,
    current_transform: &Transform,
    new_coords: GridCoordinates,
    and_delete_after: AndDeleteAfter,
) {
    let easing = current_transform.ease_to(
        Transform {
            translation: Vec3::new(
                new_coords.x as f32 * TILE_SIZE,
                new_coords.y as f32 * TILE_SIZE,
                1.,
            ),
            rotation: current_transform.rotation,
            scale: current_transform.scale,
        },
        EaseMethod::EaseFunction(bevy_easings::EaseFunction::CubicIn),
        bevy_easings::EasingType::Once {
            duration: Duration::from_secs_f32(MOVEMENT_ANIMATION),
        },
    );
    match and_delete_after {
        AndDeleteAfter::Yes => {
            commands.entity(entity).insert((
                easing,
                MarkedForDeletion(Timer::new(
                    Duration::from_secs_f32(MOVEMENT_ANIMATION),
                    TimerMode::Once,
                )),
            ));
        }
        AndDeleteAfter::No => {
            commands.entity(entity).insert(easing);
        }
    }
}
