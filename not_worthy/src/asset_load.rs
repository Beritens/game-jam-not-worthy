use bevy::asset::Handle;
use bevy::audio::AudioSource;
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
    #[asset(texture_atlas_layout(
        tile_size_x = 128,
        tile_size_y = 128,
        columns = 1,
        rows = 12,
        padding_x = 0,
        padding_y = 0,
        offset_x = 0,
        offset_y = 0
    ))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "hero_sprite_sheet.png")]
    pub image: Handle<Image>,
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
    pub image: Handle<Image>,
}

// #[derive(AssetCollection, Resource)]
// pub struct DebugSprite {
//     #[asset(path = "white.png")]
//     pub idle: Handle<Image>,
// }

#[derive(AssetCollection, Resource)]
pub struct EnvironmentArt {
    #[asset(path = "light.png")]
    pub light: Handle<Image>,
    #[asset(path = "stone/stone_top.png")]
    pub stone_top: Handle<Image>,
    #[asset(path = "stone/stone_bottom.png")]
    pub stone_bottom: Handle<Image>,
    #[asset(path = "background.png")]
    pub background: Handle<Image>,
    #[asset(path = "pile.png")]
    pub bone_pile: Handle<Image>,
    #[asset(path = "ground.png")]
    pub ground: Handle<Image>,
    #[asset(path = "under_ground.png")]
    pub under_ground: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct CutSceneArt {
    #[asset(path = "cut_scene.png")]
    pub cut_scene: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct EnemySounds {
    #[asset(path = "sounds/swoosh.ogg")]
    pub swoosh: Handle<AudioSource>,
    #[asset(path = "sounds/steps.ogg")]
    pub steps: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct PlayerSounds {
    #[asset(path = "sounds/swoosh.ogg")]
    pub swoosh: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct MusicAssets {
    #[asset(path = "music/GrumpySworrd_instance.wav")]
    pub shop: Handle<AudioSource>,
    #[asset(path = "music/GrumpySworrd_LoadScreen.wav")]
    pub menu: Handle<AudioSource>,
    #[asset(path = "music/GrumpySworrd_intense.wav")]
    pub in_game: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct CutSceneSounds {
    #[asset(path = "music/draw_sword.wav")]
    pub draw_sword: Handle<AudioSource>,
}

#[derive(serde::Deserialize)]
pub struct ShopDisplay {
    pub cost: i32,
    pub text: String,
}
#[derive(serde::Deserialize)]
pub struct ShopItem {
    pub name: String,
    pub shop_displays: Vec<ShopDisplay>,
}
#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
pub struct GameInfos {
    pub shop_items: Vec<ShopItem>,
    pub knockback: Vec<f32>,
    pub damage: Vec<f32>,
    pub speed: Vec<f32>,
    pub arise_cooldown: Vec<f32>,
    pub arise_count: Vec<i32>,
    pub attack_cooldown: Vec<f32>,
}
// #[derive(Resource)]
// struct LevelHandle(Handle<crate::Level>);
#[derive(AssetCollection, Resource)]
pub struct GameData {
    #[asset(path = "game.data.json")]
    pub data: Handle<GameInfos>,
}
