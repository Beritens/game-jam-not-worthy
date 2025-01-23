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
use crate::asset_load::{DebugSprite, EnemySprite, EnvironmentArt, SkeletonSprite, SwordAnimation};
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
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PrimaryWindow;
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
            .load_collection::<EnvironmentArt>()
            .load_collection::<DebugSprite>()
            .load_collection::<EnemySprite>()
            .load_collection::<SkeletonSprite>(),
    );
    app.add_systems(
        OnEnter(GameState::Main),
        (
            setup.run_if(in_state(GameState::Main)),
            summon_world.run_if(in_state(GameState::Main)),
        ),
    );
    //debug
    app.add_systems(Update, asset_placer_sytem);
    app.run();
}

#[derive(Component)]
struct MainCamera;
fn setup(
    mut commands: Commands,
    asset: Res<SwordAnimation>,
    skel_asset: Res<SkeletonSprite>,
    mut sprite_params: Sprite3dParams,
) {
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 2.00, 10.0),
        // Projection::from(PerspectiveProjection {
        //     fov: 0.0,
        //     aspect_ratio: 0.0,
        //     near: 0.0,
        //     far: 0.0,
        // }), // Projection::from(OrthographicProjection {
        //     // 6 world units per pixel of window height.
        //     scaling_mode: ScalingMode::FixedHorizontal {
        //         viewport_width: 12.0,
        //     },
        //     ..OrthographicProjection::default_3d()
        // }),
    ));

    commands.spawn((
        Transform::from_xyz(0.0, -1.0, -2.0).with_scale(Vec3::new(100.0, 1.0, 1.0)),
        RigidBody::Static,
        Collider::rectangle(1.0, 1.0),
    ));

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
}

fn summon_world(
    mut commands: Commands,
    sword_asset: Res<SwordAnimation>,
    world_asset: Res<EnvironmentArt>,
    mut sprite_params: Sprite3dParams,
    debug_asset: Res<DebugSprite>,
) {
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

    let texture_atlas = TextureAtlas {
        layout: sword_asset.layout.clone(),
        index: 0,
    };

    commands.spawn((
        Sprite3dBuilder {
            image: sword_asset.idle.clone(),
            pixels_per_metre: 500.0,
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            pivot: Option::from(Vec2::new(0.2, 0.5)),
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

    let stone_bottom_sprite = Sprite3dBuilder {
        image: world_asset.stone_bottom.clone(),
        pixels_per_metre: 256.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(-0.009, -0.399, -0.99).with_scale(Vec3::new(1.0, 1.0, 1.0)),
        stone_bottom_sprite.bundle(&mut sprite_params),
    ));

    let stone_top_sprite = Sprite3dBuilder {
        image: world_asset.stone_top.clone(),
        pixels_per_metre: 256.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(-0.009, -0.399, -1.01).with_scale(Vec3::new(1.0, 1.0, 1.0)),
        stone_top_sprite.bundle(&mut sprite_params),
    ));

    let light_sprite = Sprite3dBuilder {
        image: world_asset.light.clone(),
        pixels_per_metre: 128.0,
        alpha_mode: AlphaMode::Add,
        unlit: false,
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(0.0, 3.67, 2.0).with_scale(Vec3::new(1.0, 1.0, 1.0)),
        light_sprite.bundle(&mut sprite_params),
    ));

    let background_sprite = Sprite3dBuilder {
        image: world_asset.background.clone(),
        pixels_per_metre: 256.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(0.0, 2.00, -30.0).with_scale(Vec3::new(120.0, 60.0, 1.0)),
        background_sprite.bundle(&mut sprite_params),
    ));

    let pile_sprite = Sprite3dBuilder {
        image: world_asset.bone_pile.clone(),
        pixels_per_metre: 100.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(0.09, 0.64, -2.3).with_scale(Vec3::new(1.0, 1.0, 1.0)),
        pile_sprite.bundle(&mut sprite_params),
    ));

    let ground_sprite = Sprite3dBuilder {
        image: world_asset.ground.clone(),
        pixels_per_metre: 100.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(0.0, -1.506, -2.2).with_scale(Vec3::new(1.0, 1.0, 1.0)),
        ground_sprite.bundle(&mut sprite_params),
    ));

    let ground_sprite = Sprite3dBuilder {
        image: world_asset.under_ground.clone(),
        pixels_per_metre: 1.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };

    commands.spawn((
        Transform::from_xyz(0.0, -5.0, -2.25).with_scale(Vec3::new(100.0, 5.0, 1.0)),
        ground_sprite.bundle(&mut sprite_params),
    ));
}
//dev stuff
#[derive(Component)]
pub struct AssetPlacer {}

fn asset_placer_sytem(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut asset_placer_query: Query<&mut Transform, With<AssetPlacer>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut evr_scroll: EventReader<MouseWheel>,
) {
    use bevy::input::ButtonState;
    let mut shoud_print = false;

    for ev in mousebtn_evr.read() {
        match ev.state {
            ButtonState::Pressed => {
                shoud_print = true;
            }
            ButtonState::Released => {}
        }
    }
    use bevy::input::mouse::MouseScrollUnit;
    let mut scroll = 0.0;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                scroll += ev.y;
            }
            MouseScrollUnit::Pixel => {
                println!(
                    "Scroll (pixel units): vertical: {}, horizontal: {}",
                    ev.y, ev.x
                );
            }
        }
    }

    let (camera, camera_transform) = match q_camera.get_single() {
        Ok(result) => result,
        Err(_) => return,
    };

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_windows.single();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| (ray.origin + ray.direction * 10.0).truncate())
    {
        for mut transform in asset_placer_query.iter_mut() {
            transform.translation.x = world_position.x;
            transform.translation.y = world_position.y;
            transform.translation.z += scroll * 0.1;
            if (shoud_print) {
                println!("asset placer pos: {}", transform.translation);
            }
        }
    }
}
