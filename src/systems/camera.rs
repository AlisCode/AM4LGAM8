use bevy::{
    prelude::{
        Camera, Camera2dBundle, Commands, OrthographicProjection, Transform, UVec2, Vec2, Vec3,
    },
    render::camera::{ScalingMode, Viewport},
};

use crate::constants::{GRID_SIZE, TILE_SIZE};

pub fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::Fixed {
        width: TILE_SIZE * (GRID_SIZE + 3) as f32,
        height: TILE_SIZE * (GRID_SIZE + 3) as f32,
    };
    camera_bundle.transform.translation = Vec3::new(
        TILE_SIZE * (GRID_SIZE as f32 / 2.),
        TILE_SIZE * (GRID_SIZE as f32 / 2.),
        0.,
    );
    commands.spawn(camera_bundle);
}
