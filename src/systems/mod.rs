use bevy::prelude::Component;

pub mod animations;
pub mod camera;
pub mod debug;
pub mod explosion;
pub mod game_over;
pub mod grid;
pub mod marked_for_deletion;
pub mod movables;
pub mod tiles;
pub mod title_screen;
pub mod ui;

#[derive(Debug, Component)]
pub struct OnPlayingScreen;
