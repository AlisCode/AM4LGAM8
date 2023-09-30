use bevy::prelude::{App, DefaultPlugins, PluginGroup};
use bevy_mod_picking::{prelude::DebugPickingPlugin, DefaultPickingPlugins};

mod bundles;
pub mod constants;
mod core;
pub mod game;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_plugins(core::GamePlugin)
        .run()
}
