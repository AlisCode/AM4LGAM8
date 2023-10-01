use bevy::prelude::Color;

pub const GRID_SIZE: i32 = 7;
pub const TILE_SIZE_INTEGER: i32 = 16;
pub const TILE_SIZE: f32 = TILE_SIZE_INTEGER as f32;
pub const GAME_WINDOW_WIDTH: f32 = 512.;
pub const GAME_WINDOW_HEIGHT: f32 = 512.;

pub fn background_color() -> Color {
    Color::rgb_u8(47, 33, 59)
}
