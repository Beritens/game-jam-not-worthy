mod animation;
mod asset_load;
mod combat;
mod enemy;
mod game_state;
mod input_manager;
mod movement;
mod player_states;
mod spawning;
mod summoning;

use crate::animation::{AnimationTimer, SpriteAnimationPlugin};
use crate::asset_load::{DebugSprite, EnemySprite, SkeletonSprite, SwordAnimation};
use crate::combat::{CombatPlugin, Hitter, Opfer};
use crate::enemy::{BacicEnemActiveState, BasicEnemStateMachine, EnemyPlugin, Target, Walker};
use crate::game_state::GameState;
use crate::input_manager::InputManagingPlugin;
use crate::movement::{
    get_enemy_collision_layers, get_player_collision_layers, Controllable, MovementPlugin,
};
use crate::player_states::PlayerPlugin;
use crate::spawning::{spawn_enemy, Spawner, SpawningPlugin};
use crate::summoning::{spawn_player, SummoningPlugin};
use crate::GameState::Loading;
use avian2d::prelude::{
    Collider, CollisionLayers, Gravity, LockedAxes, MassPropertiesBundle, RigidBody,
};
use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_asset_loader::prelude::{
    AssetCollection, ConfigureLoadingState, LoadingState, LoadingStateAppExt, LoadingStateConfig,
};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams, Sprite3dPlugin};
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MovementPlugin);
    app.add_plugins(CombatPlugin);
    app.add_plugins(PlayerPlugin);
    app.add_plugins(InputManagingPlugin);
    app.add_plugins(SummoningPlugin);
    app.add_plugins(SpawningPlugin);
    app.add_plugins(EnemyPlugin);
    app.add_plugins(SpriteAnimationPlugin);
    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity(Vec2::new(0.0, -9.81)));
    app.add_plugins(Sprite3dPlugin);
    app.init_state::<GameState>();
    app.add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Main)
            .load_collection::<SwordAnimation>()
            .load_collection::<DebugSprite>()
            .load_collection::<EnemySprite>()
            .load_collection::<SkeletonSprite>(),
    );
    app.add_systems(
        OnEnter(GameState::Main),
        setup.run_if(in_state(GameState::Main)),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    asset: Res<SwordAnimation>,
    debug_asset: Res<DebugSprite>,
    skel_asset: Res<SkeletonSprite>,
    hero_asset: Res<EnemySprite>,
    mut sprite_params: Sprite3dParams,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.00, 10.0),
        Projection::from(OrthographicProjection {
            // 6 world units per pixel of window height.
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: 12.0,
            },
            ..OrthographicProjection::default_3d()
        }),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 1.0, 1.0),
            intensity: 200000.0,
            range: 20.0,
            radius: 0.0,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 2.0),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 1.0, 0.7),
        brightness: 100.0,
    });
    spawn_player(&mut commands, 0.0, &skel_asset, &mut sprite_params);
    spawn_player(&mut commands, 2.0, &skel_asset, &mut sprite_params);
    spawn_player(&mut commands, -1.5, &skel_asset, &mut sprite_params);

    // spawn_enemy(&mut commands, -10.0, &hero_asset.idle, &mut sprite_params);
    // spawn_enemy(&mut commands, -6.0, &hero_asset.idle, &mut sprite_params);
    // spawn_enemy(&mut commands, -3.0, &hero_asset.idle, &mut sprite_params);
    // spawn_enemy(&mut commands, 4.0, &hero_asset.idle, &mut sprite_params);
    // spawn_enemy(&mut commands, 7.0, &hero_asset.idle, &mut sprite_params);
    // spawn_enemy(&mut commands, 12.0, &hero_asset.idle, &mut sprite_params);
    commands.spawn((
        Transform::from_xyz(7.0, 2.00, 0.0),
        Spawner {
            timer: Timer::new(Duration::from_secs_f32(3.0), TimerMode::Repeating),
        },
    ));

    commands.spawn((
        Transform::from_xyz(-7.0, 2.00, 0.0),
        Spawner {
            timer: Timer::new(Duration::from_secs_f32(3.0), TimerMode::Repeating),
        },
    ));

    let texture_atlas = TextureAtlas {
        layout: asset.layout.clone(),
        index: 0,
    };

    commands.spawn((
        Sprite3dBuilder {
            image: asset.idle.clone(),
            pixels_per_metre: 500.0,
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            ..default()
        }
        .bundle_with_atlas(&mut sprite_params, texture_atlas),
        AnimationTimer {
            start: 0,
            end: 8,
            timer: Timer::new(Duration::from_secs_f32(0.15), TimerMode::Repeating),
        },
        Transform::from_xyz(0.0, -0.1, -1.0),
    ));
    let sprite = Sprite3dBuilder {
        image: debug_asset.idle.clone(),
        pixels_per_metre: 1.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(0.0, -1.0, 0.5).with_scale(Vec3::new(100.0, 1.0, 1.0)),
        RigidBody::Static,
        Collider::rectangle(1.0, 1.0),
        sprite.bundle(&mut sprite_params),
    ));
}
