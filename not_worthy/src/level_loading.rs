use crate::animation::AnimationTimer;
use crate::asset_load::{
    CutSceneArt, CutSceneSounds, EnemySounds, EnemySprite, EnvironmentArt, MusicAssets,
    SkeletonSprite, SwordAnimation,
};
use crate::combat::CombatPlugin;
use crate::game_state::GameState;
use crate::spawning::{EnemySpawner, EnemyType};
use crate::summoning::{spawn_deceased, spawn_player, DeceasedSpawnPoint};
use avian2d::collision::Collider;
use avian2d::prelude::RigidBody;
use bevy::app::{App, Main, Plugin, Startup, Update};
use bevy::asset::ErasedAssetLoader;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackMode, PlaybackSettings, Volume};
use bevy::color::Color;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::math::{Vec2, Vec3};
use bevy::pbr::{AmbientLight, PointLight};
use bevy::prelude::{
    default, in_state, AlphaMode, AssetServer, Assets, AudioBundle, Camera, Camera3d, Commands,
    Component, DespawnRecursiveExt, Entity, EventReader, GlobalTransform, Handle,
    IntoSystemConfigs, LinearRgba, Name, NextState, OnEnter, OnExit, Query, Res, ResMut, SystemSet,
    TextureAtlas, Time, Timer, TimerMode, Transform, Vec4, Window, With, Without,
};
use bevy::window::PrimaryWindow;
use bevy_hanabi::{
    AccelModifier, Attribute, ColorOverLifetimeModifier, EffectAsset, ExprWriter, Gradient,
    LinearDragModifier, Module, ParticleEffect, ParticleEffectBundle, ParticleGroupSet, ScalarType,
    SetAttributeModifier, SetPositionCircleModifier, SetPositionSphereModifier,
    SetVelocitySphereModifier, ShapeDimension, SizeOverLifetimeModifier, Spawner,
};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
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
                setup_particles.run_if(in_state(GameState::InGame)),
                setup_arise_particles.run_if(in_state(GameState::InGame)),
                setup_attack_particles.run_if(in_state(GameState::InGame)),
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

        app.add_systems(Startup, setup_necessary);
        app.add_systems(OnExit(GameState::Menu), (delete_everything));
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
fn setup(
    mut commands: Commands,
    hero_asset: Res<EnemySprite>,
    mut sprite_params: Sprite3dParams,
    asset_server: Res<AssetServer>,
    enemy_sounds: Res<EnemySounds>,
    music_assets: Res<MusicAssets>,
) {
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
    ));

    commands.spawn((
        Transform::from_xyz(0.0, -1.0, -2.0).with_scale(Vec3::new(100.0, 1.0, 1.0)),
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
    commands.spawn((
        Transform::from_xyz(30.0, 2.00, 0.0),
        EnemySpawner {
            inactive: Timer::default(),
            preheat: 0.0,
            min: 0.1,
            max: 10.0,
            factor: 0.8,
            timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Repeating),
            enemy_type: EnemyType::BASIC,
        },
        SceneObject,
    ));

    commands.spawn((
        Transform::from_xyz(-30.0, 2.00, 0.0),
        EnemySpawner {
            inactive: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Once),
            preheat: 0.0,
            min: 0.1,
            max: 10.0,
            factor: 0.8,
            timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Repeating),
            enemy_type: EnemyType::BASIC,
        },
        SceneObject,
    ));
    commands.spawn((
        Transform::from_xyz(-30.0, 2.00, 0.0),
        EnemySpawner {
            inactive: Timer::default(),
            preheat: 2.0,
            min: 0.1,
            max: 10.0,
            factor: 1.1,
            timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Repeating),
            enemy_type: EnemyType::FAST,
        },
        SceneObject,
    ));

    commands.spawn((
        Transform::from_xyz(30.0, 2.00, 0.0),
        EnemySpawner {
            inactive: Timer::default(),
            preheat: 3.0,
            min: 0.1,
            max: 10.0,
            factor: 1.1,
            timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Repeating),
            enemy_type: EnemyType::FAST,
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
    // commands.spawn((
    //     Camera3d::default(),
    //     MainCamera,
    //     Transform::from_xyz(0.0, 2.00, 10.0),
    // ));
}
fn setup_loading(mut commands: Commands, asset_server: Res<AssetServer>) {}

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
            timer: Timer::new(Duration::from_secs_f32(2.0), TimerMode::Once),
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

#[derive(Component)]
pub struct SwordEffect;
fn setup_particles(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 1.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec3::splat(0.05));
    size_gradient1.add_key(0.3, Vec3::splat(0.05));
    size_gradient1.add_key(1.0, Vec3::splat(0.0));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    // let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    // let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.5).normal(writer.lit(0.1)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * 8.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down a bit after the initial explosion
    let drag = writer.lit(0.2).expr();
    let update_drag = LinearDragModifier::new(drag);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.1).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(-Vec3::Y).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(0.1) + writer.lit(1.5)).expr(),
    };

    // Clear the trail velocity so trail particles just stay in place as they fade
    // away
    let init_vel_trail =
        SetAttributeModifier::new(Attribute::VELOCITY, writer.lit(Vec3::ZERO).expr());

    let lead = ParticleGroupSet::single(0);

    let effect = EffectAsset::new(
        // 2k lead particles, with 32 trail particles each
        2048,
        Spawner::rate(100.0.into()),
        writer.finish(),
    )
    // Tie together trail particles to make arcs. This way we don't need a lot of them, yet there's
    // a continuity between them.
    .with_name("sword_effect")
    .init_groups(init_pos, lead)
    .init_groups(init_vel, lead)
    .init_groups(init_lifetime, lead)
    .update_groups(update_drag, lead)
    .update_groups(update_accel, lead)
    .render_groups(
        ColorOverLifetimeModifier {
            gradient: color_gradient1.clone(),
        },
        lead,
    )
    .render_groups(
        SizeOverLifetimeModifier {
            gradient: size_gradient1.clone(),
            screen_space_size: false,
        },
        lead,
    );

    let effect1 = effects.add(effect);

    commands.spawn((
        SceneObject,
        SwordEffect,
        Name::new("sword_effect"),
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect1),
            transform: Transform::from_xyz(0.0, -0.3, -1.2),
            ..Default::default()
        },
    ));
}

#[derive(Component)]
pub struct AriseEffect;
fn setup_arise_particles(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 1.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec3::splat(0.05));
    size_gradient1.add_key(0.3, Vec3::splat(0.05));
    size_gradient1.add_key(1.0, Vec3::splat(0.0));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    // let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    // let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.5).normal(writer.lit(0.1)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * 8.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down a bit after the initial explosion
    let drag = writer.lit(0.2).expr();
    let update_drag = LinearDragModifier::new(drag);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.1).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(-Vec3::Y * 0.07).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(1.2) + writer.lit(10.5)).expr(),
    };

    // Clear the trail velocity so trail particles just stay in place as they fade
    // away
    let init_vel_trail =
        SetAttributeModifier::new(Attribute::VELOCITY, writer.lit(Vec3::ZERO).expr());

    let lead = ParticleGroupSet::single(0);

    let effect = EffectAsset::new(
        // 2k lead particles, with 32 trail particles each
        400,
        Spawner::once(400.0.into(), false),
        writer.finish(),
    )
    // Tie together trail particles to make arcs. This way we don't need a lot of them, yet there's
    // a continuity between them.
    .with_name("sword_arise_effect")
    .init_groups(init_pos, lead)
    .init_groups(init_vel, lead)
    .init_groups(init_lifetime, lead)
    .update_groups(update_drag, lead)
    .update_groups(update_accel, lead)
    .render_groups(
        ColorOverLifetimeModifier {
            gradient: color_gradient1.clone(),
        },
        lead,
    )
    .render_groups(
        SizeOverLifetimeModifier {
            gradient: size_gradient1.clone(),
            screen_space_size: false,
        },
        lead,
    );

    let effect1 = effects.add(effect);

    commands.spawn((
        AriseEffect,
        SceneObject,
        Name::new("sword_arise_effect"),
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect1),
            transform: Transform::from_xyz(0.0, -0.3, -1.2),
            ..Default::default()
        },
    ));
}

#[derive(Component)]
pub struct AttackEffect;
fn setup_attack_particles(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 1.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec3::splat(0.05));
    size_gradient1.add_key(0.3, Vec3::splat(0.05));
    size_gradient1.add_key(1.0, Vec3::splat(0.0));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    // let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    // let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.1).normal(writer.lit(0.1)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * 2.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down a bit after the initial explosion
    let drag = writer.lit(0.2).expr();
    let update_drag = LinearDragModifier::new(drag);

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(0.1).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(-Vec3::Y * 0.00).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(3.2) + writer.lit(0.1)).expr(),
    };

    // Clear the trail velocity so trail particles just stay in place as they fade
    // away
    let init_vel_trail =
        SetAttributeModifier::new(Attribute::VELOCITY, writer.lit(Vec3::ZERO).expr());

    let lead = ParticleGroupSet::single(0);

    let effect = EffectAsset::new(
        // 2k lead particles, with 32 trail particles each
        100,
        Spawner::once(50.0.into(), false),
        writer.finish(),
    )
    // Tie together trail particles to make arcs. This way we don't need a lot of them, yet there's
    // a continuity between them.
    .with_name("sword_attack_effect")
    .init_groups(init_pos, lead)
    .init_groups(init_vel, lead)
    .init_groups(init_lifetime, lead)
    .update_groups(update_drag, lead)
    .update_groups(update_accel, lead)
    .render_groups(
        ColorOverLifetimeModifier {
            gradient: color_gradient1.clone(),
        },
        lead,
    )
    .render_groups(
        SizeOverLifetimeModifier {
            gradient: size_gradient1.clone(),
            screen_space_size: false,
        },
        lead,
    );

    let effect1 = effects.add(effect);

    commands.spawn((
        AttackEffect,
        SceneObject,
        Name::new("sword_attack_effect"),
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect1),
            transform: Transform::from_xyz(0.0, -0.3, -0.9),
            ..Default::default()
        },
    ));
}
