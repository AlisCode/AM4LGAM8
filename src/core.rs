use crate::{
    game::{
        grid::{MoveTileEvent, TileGrid},
        moves::{CombineEvent, ValidMoveEvent},
    },
    systems::{self, movables::RequestMoveEvent},
};
use bevy::prelude::{App, Plugin, PreUpdate, Startup, Update};

pub struct GamePlugin;

impl GamePlugin {
    fn resources(app: &mut App) {
        app.add_event::<RequestMoveEvent>()
            .add_event::<ValidMoveEvent>()
            .add_event::<MoveTileEvent>()
            .add_event::<CombineEvent>()
            .insert_resource(TileGrid::default());
    }

    fn setup_systems(app: &mut App) {
        app.add_systems(Startup, systems::camera::setup)
            .add_systems(Startup, systems::grid::setup_grid)
            .add_systems(Startup, systems::debug::setup_debug);
    }

    fn update_systems(app: &mut App) {
        app.add_systems(PreUpdate, systems::grid::sync_tile_grid);
        app.add_systems(Update, systems::tiles::handle_requested_move_events)
            .add_systems(Update, systems::tiles::handle_valid_move_events)
            .add_systems(Update, systems::tiles::handle_combine_events);
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        GamePlugin::resources(app);
        GamePlugin::setup_systems(app);
        GamePlugin::update_systems(app);
    }
}
