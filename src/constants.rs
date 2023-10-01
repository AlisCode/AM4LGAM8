use bevy::prelude::Color;

pub const GRID_SIZE: i32 = 4;
pub const TILE_SIZE_INTEGER: i32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_INTEGER as f32;
pub const GAME_WINDOW_WIDTH: f32 = GAME_LOGIC_WIDTH * 5.;
pub const GAME_WINDOW_HEIGHT: f32 = GAME_LOGIC_HEIGHT * 5.;

pub const GAME_LOGIC_WIDTH: f32 = 100.; // The padding is for the UI
pub const GAME_LOGIC_HEIGHT: f32 = 106.; // The padding is for the UI

// pub const GAME_LOGIC_WIDTH: f32 = TILE_SIZE * (GRID_SIZE + 2) as f32 + 12.; // The padding is for the UI
// pub const GAME_LOGIC_HEIGHT: f32 = TILE_SIZE * (GRID_SIZE + 2) as f32 + 6.; // The padding is for the UI

pub fn background_color() -> Color {
    Color::rgb_u8(47, 33, 59)
}

pub fn foreground_color() -> Color {
    Color::rgb_u8(192, 209, 204)
}
