use bevy::prelude::{Camera2dBundle, Commands};

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        ..Default::default()
    });
}
