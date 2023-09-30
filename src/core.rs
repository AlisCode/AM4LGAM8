use crate::systems::{
    self,
    grid::MoveTileEvent,
    movables::RequestMoveEvent,
    tiles::{TileGrid, ValidMoveEvent},
};
use bevy::prelude::{App, Plugin, Startup, Update};

pub struct GamePlugin;

impl GamePlugin {
    fn resources(app: &mut App) {
        app.add_event::<RequestMoveEvent>()
            .add_event::<ValidMoveEvent>()
            .add_event::<MoveTileEvent>()
            .insert_resource(TileGrid::default());
    }

    fn setup_systems(app: &mut App) {
        app.add_systems(Startup, systems::camera::setup)
            .add_systems(Startup, systems::grid::setup_grid)
            .add_systems(Startup, systems::debug::setup_debug);
    }

    fn update_systems(app: &mut App) {
        app.add_systems(Update, systems::tiles::sync_tile_grid)
            .add_systems(Update, systems::tiles::handle_requested_move_events)
            .add_systems(Update, systems::tiles::handle_valid_move_events);
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        GamePlugin::resources(app);
        GamePlugin::setup_systems(app);
        GamePlugin::update_systems(app);
    }
}
