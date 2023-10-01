use bevy::math::Vec2;
use bevy::prelude::{AssetServer, Assets, Handle, Image, Resource};
use bevy::sprite::TextureAtlas;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        columns = 4,
        rows = 4,
        padding_x = 0.,
        padding_y = 0.,
    ))]
    #[asset(path = "tileset.png")]
    pub tileset: Handle<TextureAtlas>,
    #[asset(path = "ui.png")]
    pub ui: Handle<Image>,
    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        columns = 6,
        rows = 1,
        padding_x = 0.,
        padding_y = 0.,
    ))]
    #[asset(path = "explosion.png")]
    pub explosion: Handle<TextureAtlas>,
}
