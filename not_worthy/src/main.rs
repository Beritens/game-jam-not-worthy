mod animation;
mod asset_load;
mod combat;
mod enemy;
mod game_manager;
mod game_state;
mod input_manager;
mod level_loading;
mod movement;
mod player_states;
mod spawning;
mod summoning;
mod ui_stuff;

use crate::animation::{AnimationTimer, SpriteAnimationPlugin};
use crate::asset_load::{
    CutSceneArt, EnemySounds, EnemySprite, EnvironmentArt, PlayerSounds, SkeletonSprite,
    SwordAnimation,
};
use crate::combat::{CombatPlugin, Hitter, Opfer};
use crate::enemy::{BacicEnemActiveState, BasicEnemStateMachine, EnemyPlugin, Target, Walker};
use crate::game_manager::GameManagerPlugin;
use crate::game_state::GameState;
use crate::input_manager::InputManagingPlugin;
use crate::level_loading::LevelLoadingPlugin;
use crate::movement::{
    get_enemy_collision_layers, get_player_collision_layers, Controllable, MovementPlugin,
};
use crate::player_states::PlayerPlugin;
use crate::spawning::{spawn_enemy, Spawner, SpawningPlugin};
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
use bevy::window::PrimaryWindow;
use bevy_asset_loader::prelude::{
    AssetCollection, ConfigureLoadingState, LoadingState, LoadingStateAppExt, LoadingStateConfig,
};
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
    app.add_plugins(MovementPlugin);
    app.add_plugins(GameManagerPlugin);
    app.add_plugins(CombatPlugin);
    app.add_plugins(PlayerPlugin);
    app.add_plugins(InputManagingPlugin);
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
            .continue_to_state(GameState::Menu)
            .load_collection::<SwordAnimation>()
            .load_collection::<EnvironmentArt>()
            .load_collection::<CutSceneArt>()
            .load_collection::<EnemySounds>()
            .load_collection::<PlayerSounds>()
            // .load_collection::<DebugSprite>()
            .load_collection::<EnemySprite>()
            .load_collection::<SkeletonSprite>(),
    );

    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    #[cfg(debug_assertions)] // debug/dev builds only
    {
        use bevy::diagnostic::LogDiagnosticsPlugin;
        app.add_plugins(LogDiagnosticsPlugin::default());
    }
    app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)));
    app.run();
}
