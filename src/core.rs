use crate::systems;
use bevy::prelude::{Plugin, Startup};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, systems::camera::setup)
            .add_systems(Startup, systems::debug::setup_debug);
    }
}
