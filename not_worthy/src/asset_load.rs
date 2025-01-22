use bevy::asset::Handle;
use bevy::image::Image;
use bevy::prelude::{Resource, TextureAtlasLayout};
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
pub struct SwordAnimation {
    #[asset(texture_atlas_layout(
        tile_size_x = 500,
        tile_size_y = 500,
        columns = 1,
        rows = 9,
        padding_x = 0,
        padding_y = 0,
        offset_x = 0,
        offset_y = 0
    ))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "sword.png")]
    pub idle: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct EnemySprite {
    #[asset(path = "hero.png")]
    pub idle: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct SkeletonSprite {
    #[asset(texture_atlas_layout(
        tile_size_x = 128,
        tile_size_y = 128,
        columns = 1,
        rows = 11,
        padding_x = 0,
        padding_y = 0,
        offset_x = 0,
        offset_y = 0
    ))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "skeleton_sprite_sheet.png")]
    pub idle: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct DebugSprite {
    #[asset(path = "white.png")]
    pub idle: Handle<Image>,
}
