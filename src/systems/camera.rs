use bevy::{
    prelude::{Camera2dBundle, Commands, Res, State, Vec3},
    render::camera::ScalingMode,
};

use crate::constants::{GAME_LOGIC_HEIGHT, GAME_LOGIC_WIDTH, GRID_SIZE, TILE_SIZE};
use crate::core::GameState;

use super::{game_over::OnGameOverScreen, title_screen::OnTitleScreen, OnPlayingScreen};

pub fn setup(mut commands: Commands, state: Res<State<GameState>>) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::Fixed {
        width: GAME_LOGIC_WIDTH,
        height: GAME_LOGIC_HEIGHT,
    };
    camera_bundle.transform.translation = Vec3::new(
        TILE_SIZE * (GRID_SIZE as f32 / 2. - 0.5),
        TILE_SIZE * (GRID_SIZE as f32 / 2. - 0.5) - 3.,
        0.,
    );

    match state.get() {
        &GameState::TitleScreen => {
            commands.spawn((camera_bundle, OnTitleScreen));
        }
        &GameState::Playing => {
            commands.spawn((camera_bundle, OnPlayingScreen));
        }
        &GameState::GameOver => {
            commands.spawn((camera_bundle, OnGameOverScreen));
        }
        _ => panic!("No camera is required on other states"),
    }
}
