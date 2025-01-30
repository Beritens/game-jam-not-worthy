use crate::animation::AnimationTimer;
use crate::asset_load::{
    CutSceneArt, CutSceneSounds, EnemySounds, EnemySprite, EnvironmentArt, MusicAssets,
    SkeletonSprite, SwordAnimation,
};
use crate::combat::CombatPlugin;
use crate::game_state::{GameState, PauseState};
use crate::spawning::{EnemySpawner, EnemyType};
use crate::summoning::{spawn_deceased, spawn_player, DeceasedSpawnPoint};
use crate::ui_stuff::CompText;
use avian2d::collision::Collider;
use avian2d::prelude::RigidBody;
use bevy::app::{App, Main, Plugin, Startup, Update};
use bevy::asset::ErasedAssetLoader;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackMode, PlaybackSettings, Volume};
use bevy::color::Color;
use bevy::core_pipeline::bloom::Bloom;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::math::{Vec2, Vec3};
use bevy::pbr::{AmbientLight, PointLight};
use bevy::prelude::{
    default, in_state, resource_changed, Alpha, AlphaMode, AssetServer, Assets, AudioBundle,
    Camera, Camera3d, Commands, Component, DespawnRecursiveExt, Entity, EventReader,
    GlobalTransform, Handle, IntoSystemConfigs, LinearRgba, Msaa, Name, NextState, OnEnter, OnExit,
    Query, Res, ResMut, SystemSet, Text, TextureAtlas, Time, Timer, TimerMode, Transform, Vec4,
    Window, With, Without,
};
use bevy::window::PrimaryWindow;
use bevy_firework::bevy_utilitarian::prelude::{RandF32, RandVec3};
use bevy_firework::core::{BlendMode, ParticleSpawner};
use bevy_firework::curve::{FireworkCurve, FireworkGradient};
use bevy_firework::emission_shape::EmissionShape;
use bevy_pipelines_ready::{PipelinesReady, PipelinesReadyPlugin};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use std::f32::consts::PI;
use std::time::Duration;

pub struct LevelLoadingPlugin;

impl Plugin for LevelLoadingPlugin {
    fn build(&self, app: &mut App) {
        println!("test");
        //debug
        // app.add_systems(Update, (asset_placer_sytem));
        app.add_systems(
            OnEnter(GameState::InGame),
            (
                setup.run_if(in_state(GameState::InGame)),
                summon_world.run_if(in_state(GameState::InGame)),
            ),
        );
        app.add_systems(
            OnEnter(GameState::Menu),
            (
                setup_menu.run_if(in_state(GameState::Menu)),
                summon_world.run_if(in_state(GameState::Menu)),
            ),
        );
        app.add_systems(
            OnEnter(GameState::CutScene),
            (setup_cut_scene.run_if(in_state(GameState::CutScene)),),
        );

        app.add_systems(
            Update,
            (cut_scene_wait_system.run_if(in_state(GameState::CutScene)),),
        );

        app.add_systems(
            OnEnter(GameState::Loading),
            (setup_loading.run_if(in_state(GameState::Loading)),),
        );

        app.add_systems(
            OnEnter(GameState::Shop),
            (
                setup_shop.run_if(in_state(GameState::Shop)),
                summon_world.run_if(in_state(GameState::Shop)),
            ),
        );

        app.add_systems(
            OnEnter(GameState::CompilingShaders),
            (
                setup_mock_world.run_if(in_state(GameState::CompilingShaders)),
                summon_world.run_if(in_state(GameState::CompilingShaders)),
            ),
        );
        app.add_plugins(PipelinesReadyPlugin);
        app.add_systems(
            Update,
            check_ready.run_if(in_state(GameState::CompilingShaders)),
        );

        app.add_systems(Startup, setup_necessary);
        app.add_systems(OnExit(GameState::Menu), (delete_everything));
        app.add_systems(OnExit(GameState::CompilingShaders), (delete_everything));
        app.add_systems(OnExit(GameState::Loading), (delete_everything));
        app.add_systems(OnExit(GameState::InGame), (delete_everything));
        app.add_systems(OnExit(GameState::CutScene), (delete_everything));
        app.add_systems(OnExit(GameState::Shop), (delete_everything));
    }
}
#[derive(Component)]
pub struct SceneObject;

#[derive(Component)]
struct MainCamera;

fn setup_mock_world(mut commands: Commands) {
    commands.spawn((
        SceneObject,
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 2.00, 10.0),
        Msaa::Off,
    ));
}
fn check_ready(
    ready: Res<PipelinesReady>,
    mut next_state: ResMut<NextState<GameState>>,
    mut info_query: Query<&mut Text, With<CompText>>,
) {
    println!("{}", ready.get());
    for (mut text) in info_query.iter_mut() {
        text.0 = format!("Compiling Shaders: {}", ready.get().to_string());
    }
    if ready.get() >= 7 {
        next_state.set(GameState::Menu);
    }
}

fn setup(
    mut commands: Commands,
    enemy_sounds: Res<EnemySounds>,
    music_assets: Res<MusicAssets>,
    mut paused_state: ResMut<NextState<PauseState>>,
) {
    paused_state.set(PauseState::Paused);
    commands.spawn((
        AudioPlayer::new(music_assets.in_game.clone()),
        PlaybackSettings {
            volume: Volume::new(0.4),
            mode: PlaybackMode::Loop,
            ..Default::default()
        },
        SceneObject,
    ));
    commands.spawn((
        AudioPlayer::new(enemy_sounds.steps.clone()),
        PlaybackSettings {
            volume: Volume::new(1.0),
            mode: PlaybackMode::Loop,
            ..Default::default()
        },
        SceneObject,
    ));
    // commands.spawn((
    //     Camera3d::default(),
    //     MainCamera,
    //     Transform::from_xyz(0.0, 2.00, 10.0),
    // ));
    commands.spawn((
        SceneObject,
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 2.00, 10.0),
        Msaa::Off,
    ));

    commands.spawn((
        Transform::from_xyz(0.0, -1.0, -2.0).with_scale(Vec3::new(1000.0, 1.0, 1.0)),
        RigidBody::Static,
        Collider::rectangle(1.0, 1.0),
        SceneObject,
    ));

    commands.spawn((
        SceneObject {},
        DeceasedSpawnPoint {
            enemy_type: EnemyType::BASIC,
        },
        Transform::from_xyz(1.5, 0.0, 0.0),
    ));

    if (true) {
        for i in -50..50 {
            commands.spawn((
                SceneObject {},
                DeceasedSpawnPoint {
                    enemy_type: EnemyType::BASIC,
                },
                Transform::from_xyz(6.0 * i as f32 / 50.0, 0.0, 0.0),
            ));
        }
    }
    // commands.spawn((
    //     Transform::from_xyz(30.0, 2.00, 0.0),
    //     EnemySpawner {
    //         inactive: Timer::default(),
    //         preheat: 0.0,
    //         min: 0.2,
    //         max: 10.0,
    //         factor: 0.9,
    //         timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Repeating),
    //         enemy_type: EnemyType::BASIC,
    //     },
    //     SceneObject,
    // ));
    //
    // commands.spawn((
    //     Transform::from_xyz(-30.0, 2.00, 0.0),
    //     EnemySpawner {
    //         inactive: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Once),
    //         preheat: 0.0,
    //         min: 0.2,
    //         max: 10.0,
    //         factor: 0.9,
    //         timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Repeating),
    //         enemy_type: EnemyType::BASIC,
    //     },
    //     SceneObject,
    // ));
    // commands.spawn((
    //     Transform::from_xyz(-30.0, 2.00, 0.0),
    //     EnemySpawner {
    //         inactive: Timer::default(),
    //         preheat: 2.0,
    //         min: 0.1,
    //         max: 10.0,
    //         factor: 1.1,
    //         timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Repeating),
    //         enemy_type: EnemyType::FAST,
    //     },
    //     SceneObject,
    // ));
    //
    // commands.spawn((
    //     Transform::from_xyz(30.0, 2.00, 0.0),
    //     EnemySpawner {
    //         inactive: Timer::default(),
    //         preheat: 3.0,
    //         min: 0.1,
    //         max: 10.0,
    //         factor: 1.1,
    //         timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Repeating),
    //         enemy_type: EnemyType::FAST,
    //     },
    //     SceneObject,
    // ));
    // commands.spawn((
    //     Transform::from_xyz(-30.0, 2.00, 0.0),
    //     EnemySpawner {
    //         inactive: Timer::new(Duration::from_secs_f32(50.0), TimerMode::Once),
    //         preheat: 2.0,
    //         min: 1.0,
    //         max: 10.0,
    //         factor: 0.9,
    //         timer: Timer::new(Duration::from_secs_f32(45.0), TimerMode::Repeating),
    //         enemy_type: EnemyType::BIG,
    //     },
    //     SceneObject,
    // ));
    //
    // commands.spawn((
    //     Transform::from_xyz(30.0, 2.00, 0.0),
    //     EnemySpawner {
    //         inactive: Timer::new(Duration::from_secs_f32(20.0), TimerMode::Once),
    //         preheat: 0.0,
    //         min: 1.0,
    //         max: 10.0,
    //         factor: 0.9,
    //         timer: Timer::new(Duration::from_secs_f32(50.0), TimerMode::Repeating),
    //         enemy_type: EnemyType::BIG,
    //     },
    //     SceneObject,
    // ));
    commands.spawn((
        Transform::from_xyz(30.0, 2.00, 0.0),
        EnemySpawner {
            once: true,
            inactive: Timer::default(),
            preheat: 17.0,
            min: 1.0,
            max: 10.0,
            factor: 0.9,
            timer: Timer::new(Duration::from_secs_f32(0.10), TimerMode::Repeating),
            enemy_type: EnemyType::BIG,
        },
        SceneObject,
    ));
}

fn delete_everything(query: Query<Entity, With<SceneObject>>, mut commands: Commands) {
    for (entity) in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
fn setup_necessary(mut commands: Commands) {
    commands.spawn((
        ParticleSpawner {
            one_shot: false,
            rate: 80.0,
            emission_shape: EmissionShape::Circle {
                normal: Vec3::Z,
                radius: 30.0,
            },
            lifetime: RandF32 { min: 2.0, max: 8.0 },
            inherit_parent_velocity: true,
            initial_velocity: RandVec3 {
                magnitude: RandF32 { min: 3., max: 5.6 },
                direction: Vec3::X,
                spread: PI / 2.0,
            },
            initial_scale: RandF32 {
                min: 0.02,
                max: 0.08,
            },
            scale_curve: FireworkCurve::uneven_samples(vec![(0., 4.0), (1., 0.0)]),
            color: FireworkGradient::uneven_samples(vec![
                (0., crate::effects::THE_GREEN.clone().with_alpha(0.0)),
                (0.1, crate::effects::THE_GREEN.clone().with_alpha(0.2)),
                (1., crate::effects::THE_GREEN.clone().with_alpha(0.0)),
            ]),
            blend_mode: BlendMode::Add,
            linear_drag: 0.0,
            acceleration: Vec3::new(1.0, 2.0, 0.0),
            pbr: false,
            starts_enabled: true,
            ..default()
        },
        Transform::from_xyz(0., -0.25, -20.0),
    ));
}
fn setup_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneObject,
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 2.00, 10.0),
        Msaa::Off,
    ));
}

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    music_assets: Res<MusicAssets>,
) {
    commands.spawn((
        SceneObject,
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 2.00, 10.0),
        Msaa::Off,
    ));
    commands.spawn((
        AudioPlayer::new(music_assets.menu.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..Default::default()
        },
        SceneObject,
    ));
}

fn setup_shop(mut commands: Commands, music_assets: Res<MusicAssets>) {
    commands.spawn((
        SceneObject,
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 2.00, 10.0),
        Msaa::Off,
    ));
    commands.spawn((
        AudioPlayer::new(music_assets.shop.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..Default::default()
        },
        SceneObject,
    ));
}
#[derive(Component)]
struct CutSceneTime {
    timer: Timer,
}

fn setup_cut_scene(
    mut commands: Commands,
    cut_scene_asset: Res<CutSceneArt>,
    asset_server: Res<AssetServer>,
    mut sprite_params: Sprite3dParams,
    cut_scene_sounds: Res<CutSceneSounds>,
) {
    commands.spawn((
        SceneObject,
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 0.00, 10.0),
        Msaa::Off,
    ));
    let unsheathed = Sprite3dBuilder {
        image: cut_scene_asset.cut_scene.clone(),
        pixels_per_metre: 220.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };

    commands.spawn((
        SceneObject,
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.0, 1.0, 1.0)),
        unsheathed.bundle(&mut sprite_params),
        CutSceneTime {
            timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Once),
        },
    ));

    commands.spawn((
        AudioPlayer::new(cut_scene_sounds.draw_sword.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..Default::default()
        },
        SceneObject,
    ));
}

fn cut_scene_wait_system(
    time: Res<Time>,
    mut cut_scne_query: Query<&mut CutSceneTime>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(mut cut_scene) = cut_scne_query.get_single_mut() {
        cut_scene.timer.tick(time.delta());
        if cut_scene.timer.just_finished() {
            game_state.set(GameState::Shop);
        }
    }
}
fn summon_world(
    mut commands: Commands,
    sword_asset: Res<SwordAnimation>,
    world_asset: Res<EnvironmentArt>,
    mut sprite_params: Sprite3dParams,
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
        SceneObject,
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
        SceneObject,
        Sprite3dBuilder {
            image: sword_asset.idle.clone(),
            pixels_per_metre: 500.0,
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            pivot: Option::from(Vec2::new(0.22, 0.5)),
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
        SceneObject,
        Transform::from_xyz(-0.01, -0.399, -0.99).with_scale(Vec3::new(1.0, 1.0, 1.0)),
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
        SceneObject,
        Transform::from_xyz(-0.01, -0.399, -1.01).with_scale(Vec3::new(1.0, 1.0, 1.0)),
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
        SceneObject,
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
        SceneObject,
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
        SceneObject,
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
        SceneObject,
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
        SceneObject,
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
