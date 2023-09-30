use bevy::prelude::{App, DefaultPlugins};

mod bundles;
mod core;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(core::GamePlugin)
        .run()
}
