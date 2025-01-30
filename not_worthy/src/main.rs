mod animation;
mod asset_load;
mod combat;
mod effects;
mod end_boss;
mod enemy;
mod game_manager;
mod game_state;
mod hit_detection;
mod input_manager;
mod level_loading;
mod movement;
mod player_states;
mod shadows;
mod spawning;
mod state_handling;
mod summoning;
mod ui_stuff;

use crate::animation::{AnimationTimer, SpriteAnimationPlugin};
use crate::asset_load::{
    CutSceneArt, CutSceneSounds, EnemySounds, EnemySprite, EnvironmentArt, GameData, GameInfos,
    Messages, MusicAssets, PlayerSounds, ShadowSprite, SkeletonSprite, SwordAnimation, UIAssets,
    UISounds,
};
use crate::combat::{CombatPlugin, Hitter, Opfer};
use crate::effects::EffectPlugin;
use crate::enemy::{BacicEnemActiveState, BasicEnemStateMachine, EnemyPlugin, Target, Walker};
use crate::game_manager::GameManagerPlugin;
use crate::game_state::{GameState, PauseState};
use crate::hit_detection::{HitDetection, HitDetectionPlugin};
use crate::input_manager::InputManagingPlugin;
use crate::level_loading::LevelLoadingPlugin;
use crate::movement::{
    get_enemy_collision_layers, get_player_collision_layers, Controllable, MovementPlugin,
};
use crate::player_states::PlayerPlugin;
use crate::shadows::ShadowPlugin;
use crate::spawning::{EnemySpawner, SpawningPlugin};
use crate::summoning::{spawn_player, SummoningPlugin};
use crate::ui_stuff::UIStuffPlugin;
use crate::GameState::Loading;
use avian2d::prelude::{
    Collider, CollisionLayers, Gravity, LockedAxes, MassPropertiesBundle, RigidBody,
};
use avian2d::PhysicsPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::text::cosmic_text::Motion::Up;
use bevy::window::PrimaryWindow;
use bevy_asset_loader::prelude::{
    AssetCollection, ConfigureLoadingState, LoadingState, LoadingStateAppExt, LoadingStateConfig,
};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_firework::plugin::ParticleSystemPlugin;
use bevy_pkv::PkvStore;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams, Sprite3dPlugin};
use bevy_wasm_window_resize::WindowResizePlugin;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }));
    app.add_plugins(WindowResizePlugin);
    app.add_plugins(JsonAssetPlugin::<GameInfos>::new(&["data.json"]));

    app.add_plugins(JsonAssetPlugin::<Messages>::new(&["messages.json"]));
    app.add_plugins(MovementPlugin);
    app.add_plugins(GameManagerPlugin);
    app.add_plugins(HitDetectionPlugin);
    app.add_plugins(ShadowPlugin);
    app.add_plugins(CombatPlugin);
    app.add_plugins(PlayerPlugin);
    app.add_plugins(InputManagingPlugin);
    app.add_plugins(EffectPlugin);
    app.add_plugins(ParticleSystemPlugin::default());
    app.add_plugins(LevelLoadingPlugin);
    app.add_plugins(SummoningPlugin);
    app.add_plugins(SpawningPlugin);
    app.add_plugins(UIStuffPlugin);
    app.add_plugins(EnemyPlugin);
    app.add_plugins(SpriteAnimationPlugin);
    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity(Vec2::new(0.0, -9.81)));
    app.add_plugins(Sprite3dPlugin);
    app.init_state::<GameState>();

    app.add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::CompilingShaders)
            .load_collection::<SwordAnimation>()
            .load_collection::<EnvironmentArt>()
            .load_collection::<CutSceneArt>()
            .load_collection::<UIAssets>()
            .load_collection::<GameData>()
            .load_collection::<EnemySounds>()
            .load_collection::<PlayerSounds>()
            .load_collection::<UISounds>()
            .load_collection::<CutSceneSounds>()
            .load_collection::<MusicAssets>()
            // .load_collection::<DebugSprite>()
            .load_collection::<EnemySprite>()
            .load_collection::<ShadowSprite>()
            .load_collection::<SkeletonSprite>(),
    );

    app.insert_state(PauseState::Paused);

    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    app.insert_resource(PkvStore::new("beritens", "grumpy_sword"));
    #[cfg(debug_assertions)] // debug/dev builds only
    {
        use bevy::diagnostic::LogDiagnosticsPlugin;
        app.add_plugins(LogDiagnosticsPlugin::default());
    }
    app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)));
    // app.add_systems(Update, spawn_level.run_if(in_state(GameState::Menu)));
    app.run();
}

// #[derive(serde::Deserialize)]
// struct ShopDisplay {
//     cost: i32,
//     text: String,
// }
// #[derive(serde::Deserialize)]
// struct ShopItem {
//     name: String,
//     shop_displays: Vec<ShopDisplay>,
// }
// #[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
// struct Level {
//     shop_items: Vec<ShopItem>,
// }
// #[derive(Resource)]
// struct LevelHandle(Handle<Level>);

// fn test_load_system(mut commands: Commands, asset_server: Res<AssetServer>) {
//     let level = LevelHandle(asset_server.load("test.json"));
//     commands.insert_resource(level);
// }
// fn spawn_level(level: Res<LevelHandle>, mut levels: ResMut<Assets<Level>>) {
//     if let Some(level) = levels.get(level.0.id()) {
//         for shop_item in &level.shop_items {
//             println!("name: {:?}", &shop_item.name);
//             for shop_display in &shop_item.shop_displays {
//                 println!("{:?} : {:?}", shop_display.text, shop_display.cost);
//             }
//         }
//     }
// }

fn spawn_level(level: Res<GameData>, mut levels: ResMut<Assets<GameInfos>>) {
    if let Some(level) = levels.get(level.data.id()) {
        for shop_item in &level.shop_items {
            println!("name: {:?}", &shop_item.name);
            for shop_display in &shop_item.shop_displays {
                println!("{:?} : {:?}", shop_display.text, shop_display.cost);
            }
        }
    }
}
