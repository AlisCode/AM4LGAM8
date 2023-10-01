use crate::{
    assets,
    game::{
        grid::{MoveTileEvent, TileGrid},
        moves::{ExplosionEvent, MergeTilesEvent, ValidMoveEvent},
    },
    systems::{
        self, grid::ValidTurnEvent, movables::RequestMoveEvent, title_screen::TitleScreenEntities,
        ui::GameScore,
    },
};
use bevy::prelude::{in_state, App, IntoSystemConfigs, OnEnter, Plugin, PreUpdate, States, Update};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    TitleScreen,
    Playing,
    GameOver,
}

pub struct GamePlugin;

impl GamePlugin {
    fn assets(app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::TitleScreen),
        )
        .add_collection_to_loading_state::<_, assets::GameAssets>(GameState::Loading);
    }

    fn resources(app: &mut App) {
        app.add_event::<RequestMoveEvent>()
            .add_event::<ValidMoveEvent>()
            .add_event::<MoveTileEvent>()
            .add_event::<MergeTilesEvent>()
            .add_event::<ExplosionEvent>()
            .add_event::<ValidTurnEvent>()
            .insert_resource(TileGrid::default())
            .insert_resource(GameScore::default())
            .insert_resource(TitleScreenEntities::default());
    }

    fn on_enter_title_screen(app: &mut App) {
        app.add_systems(
            OnEnter(GameState::TitleScreen),
            (systems::camera::setup, systems::title_screen::setup),
        );
    }

    fn on_update_title_screen(app: &mut App) {
        app.add_systems(
            Update,
            systems::title_screen::update_ui.run_if(in_state(GameState::TitleScreen)),
        );
    }

    fn on_enter_playing_state(app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (
                systems::camera::setup,
                systems::grid::setup_grid,
                systems::debug::setup_debug,
                systems::ui::spawn_ui,
                systems::ui::reset_score,
            ),
        );
    }

    fn on_update_playing_state(app: &mut App) {
        // Pre-Update
        app.add_systems(
            PreUpdate,
            systems::grid::sync_tile_grid.run_if(in_state(GameState::Playing)),
        );

        // Update
        let handle_explosion_events =
            systems::tiles::handle_explosion_events.before(systems::tiles::handle_combine_events);
        let handle_combine_events =
            systems::tiles::handle_combine_events.before(systems::tiles::handle_valid_move_events);
        let handle_valid_move_events = systems::tiles::handle_valid_move_events;
        let handle_valid_turn =
            systems::grid::spawn_new_tile_on_valid_move.after(handle_valid_move_events);

        let update_systems = (
            systems::tiles::handle_requested_move_events,
            handle_explosion_events,
            handle_combine_events,
            handle_valid_move_events,
            handle_valid_turn,
            systems::ui::update_ui,
            systems::explosion::animate_explosion,
            systems::marked_for_deletion::tick_marked_for_deletion,
        )
            .run_if(in_state(GameState::Playing));
        app.add_systems(Update, update_systems);
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();
        GamePlugin::assets(app);
        GamePlugin::resources(app);
        GamePlugin::on_enter_title_screen(app);
        GamePlugin::on_update_title_screen(app);
        GamePlugin::on_enter_playing_state(app);
        GamePlugin::on_update_playing_state(app);
    }
}
