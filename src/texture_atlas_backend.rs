#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::cmp::Ordering;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_mod_picking::{
    backend::{HitData, PointerHits},
    picking_core::PickSet,
    prelude::{PointerId, PointerLocation},
};
use bevy_picking_core::Pickable;

/// Adds support for TextureAtlas to bevy_mod_picking
#[derive(Clone)]
pub struct TextureAtlasBackend;
impl Plugin for TextureAtlasBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sprite_picking.in_set(PickSet::Backend));
    }
}

/// Checks if any sprite entities are under each pointer
pub fn sprite_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    sprite_query: Query<(
        Entity,
        &Handle<TextureAtlas>,
        &TextureAtlasSprite,
        &GlobalTransform,
        &ComputedVisibility,
        Option<&Pickable>,
    )>,
    mut output: EventWriter<PointerHits>,
) {
    let mut sorted_sprites: Vec<_> = sprite_query.iter().collect();
    sorted_sprites.sort_by(|a, b| {
        (b.3.translation().z)
            .partial_cmp(&a.3.translation().z)
            .unwrap_or(Ordering::Equal)
    });

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform)) = cameras.iter().find(|(_, camera, _)| {
            camera
                .target
                .normalize(Some(primary_window.single()))
                .unwrap()
                == location.target
        }) else {
            continue;
        };

        let Some(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, location.position)
        else {
            continue;
        };

        let picks: Vec<(Entity, HitData)> = sorted_sprites
            .iter()
            .copied()
            .filter_map(
                |(entity, handle, sprite, sprite_transform, visibility, sprite_focus)| {
                    if blocked || !visibility.is_visible() {
                        return None;
                    }
                    let position = sprite_transform.translation();
                    let scale = sprite_transform.compute_transform().scale;
                    let half_extents = sprite
                        .custom_size
                        .or_else(|| {
                            let texture_atlas = texture_atlases.get(handle)?;
                            let texture_bounds = texture_atlas.textures.get(sprite.index)?;
                            Some(texture_bounds.size())
                        })
                        .map(|size| size / 2.0 * scale.truncate())?;
                    let center = position.truncate() + (sprite.anchor.as_vec() * half_extents);
                    let rect = Rect::from_center_half_size(center, half_extents);

                    let is_cursor_in_sprite = rect.contains(cursor_pos_world);
                    blocked = is_cursor_in_sprite
                        && sprite_focus.map(|p| p.should_block_lower) != Some(false);

                    is_cursor_in_sprite
                        .then_some((entity, HitData::new(cam_entity, position.z, None, None)))
                },
            )
            .collect();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks, order))
    }
}
