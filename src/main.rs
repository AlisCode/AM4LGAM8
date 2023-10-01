use bevy::{
    prelude::{App, ClearColor, DefaultPlugins, PluginGroup},
    window::{Window, WindowPlugin},
};
use bevy_mod_picking::{
    prelude::{DebugPickingPlugin, SpriteBackend},
    DefaultPickingPlugins,
};
use constants::{background_color, GAME_WINDOW_HEIGHT, GAME_WINDOW_WIDTH};
use texture_atlas_backend::TextureAtlasBackend;

mod assets;
mod bundles;
pub mod constants;
mod core;
pub mod game;
mod systems;
mod texture_atlas_backend;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (GAME_WINDOW_WIDTH, GAME_WINDOW_HEIGHT).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(ClearColor(background_color()))
        // bevy_mod_picking with hacked support for TextureAtlas
        // A big thank you to Github user focustense, saving my jam at 2AM with
        // this masterclass of a comment :
        // https://github.com/aevyrie/bevy_mod_picking/issues/210#issuecomment-1557737085
        // I'll see if I can contribute that after the jam
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>()
                .disable::<SpriteBackend>(),
        )
        .add_plugins(TextureAtlasBackend)
        .add_plugins(core::GamePlugin)
        .run()
}
