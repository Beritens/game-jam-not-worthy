use crate::animation::{Animation, AnimationManager};
use crate::asset_load::{
    EnemySounds, EnemySprite, GameData, GameInfos, ShadowSprite, SkeletonSprite,
};
use crate::combat::{Dead, Direction, Health, Hitter, Opfer};
use crate::effects::{AriseCooldownEffect, AriseEffect};
use crate::enemy::{BacicEnemActiveState, BasicEnemStateMachine, Target, Walker};
use crate::game_state::{GameState, PauseState};
use crate::input_manager::{Action, BasicControl};
use crate::level_loading::SceneObject;
use crate::movement::{
    get_enemy_collision_layers, get_player_collision_layers, Barrier, Controllable, FancyWalk,
    GameLayer,
};
use crate::player_states::{PlayerIdleState, PlayerStateMaschine, WalkAnim};
use crate::shadows::Shadow;
use crate::spawning::EnemyType;
use crate::state_handling::get_sotred_value;
use avian2d::collision::Collider;
use avian2d::parry::transformation::utils::transform;
use avian2d::prelude::{
    LayerMask, LockedAxes, MassPropertiesBundle, RigidBody, SpatialQueryFilter,
};
use bevy::app::{App, Plugin, Update};
use bevy::asset::{Assets, Handle};
use bevy::image::Image;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{
    default, in_state, AlphaMode, BuildChildren, ChildBuild, Circle, Commands, Component,
    DespawnRecursiveExt, Entity, IntoSystemConfigs, NextState, OnEnter, PreUpdate, Query, Res,
    ResMut, TextureAtlasLayout, Time, Timer, Transform, Visibility, With,
};
use bevy::sprite::TextureAtlas;
use bevy::time::TimerMode;
use bevy::utils::tracing::Instrument;
use bevy_firework::core::{ParticleData, ParticleSpawnerData};
use bevy_pkv::PkvStore;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use leafwing_input_manager::action_state::ActionState;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::f32::consts::PI;
use std::time::Duration;

pub struct SummoningPlugin;

const BARRIER_MIN: f32 = -8.0;
const BARRIER_MAX: f32 = 8.0;

impl Plugin for SummoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (arise_system, spawn_deceased, update_effect_system)
                .run_if(in_state(GameState::InGame)),
        );
        app.add_systems(
            OnEnter(GameState::InGame),
            setup_arise_system.run_if(in_state(GameState::InGame)),
        );
        // app.add_systems(PreUpdate, die_system.run_if(in_state(GameState::Main)));
        // add things to your app here
    }
}

#[derive(Component)]
pub struct AriseSettings {
    cooldown: Timer,
    num: i32,
    knockback: f32,
    damage: f32,
    speed: f32,
}

fn setup_arise_system(
    mut commands: Commands,
    game_data: Res<GameData>,
    mut game_datas: ResMut<Assets<GameInfos>>,
    mut pkv: ResMut<PkvStore>,
) {
    let knockback_level = get_sotred_value(&mut pkv, "knockback");
    let damage_level = get_sotred_value(&mut pkv, "damage");
    let speed_level = get_sotred_value(&mut pkv, "speed");
    let arise_cooldown_level = get_sotred_value(&mut pkv, "arise_cooldown");
    let arise_count_level = get_sotred_value(&mut pkv, "arise_count");
    let mut knockback = 0.0;
    let mut damage = 0.0;
    let mut speed = 0.0;
    let mut arise_cooldown = 0.0;
    let mut arise_count = 0;
    if let Some(game_data) = game_datas.get(game_data.data.id()) {
        knockback = game_data.knockback[knockback_level as usize];
        damage = game_data.damage[damage_level as usize];
        speed = game_data.speed[speed_level as usize];
        arise_cooldown = game_data.arise_cooldown[arise_cooldown_level as usize];
        arise_count = game_data.arise_count[arise_count_level as usize];
    }

    let mut cooldown = Timer::new(Duration::from_secs_f32(arise_cooldown), TimerMode::Once);
    cooldown.set_elapsed(Duration::from_secs_f32(arise_cooldown));
    commands.spawn((
        SceneObject,
        AriseSettings {
            num: arise_count,
            cooldown,
            knockback,
            damage,
            speed,
        },
    ));
}

struct EntDist {
    entity: Entity,
    dist: f32,
}

impl PartialEq<Self> for EntDist {
    fn eq(&self, other: &Self) -> bool {
        self.dist == other.dist
    }
}

impl Eq for EntDist {}

impl PartialOrd<Self> for EntDist {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.dist.partial_cmp(&other.dist)
    }
}

impl Ord for EntDist {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

struct PlayerSettings {
    knockback: f32,
    speed: f32,
    damage: f32,
}

fn update_effect_system(
    arise_settings_query: Query<(&AriseSettings)>,
    mut arise_cooldown_effect_query: Query<(&mut ParticleSpawnerData), With<AriseCooldownEffect>>,
) {
    let Ok(arise_settings) = arise_settings_query.get_single() else {
        return;
    };
    for (mut effect) in arise_cooldown_effect_query.iter_mut() {
        effect.enabled = arise_settings.cooldown.finished();
    }
}
fn arise_system(
    time: Res<Time>,
    mut commands: Commands,
    input_query: Query<(&ActionState<Action>), With<BasicControl>>,
    deceased_query: Query<(Entity, &Transform), With<Deceased>>,
    skelet_asset: Res<SkeletonSprite>,
    shadow_asset: Res<ShadowSprite>,
    mut sprite_params: Sprite3dParams,
    mut arise_settings_query: Query<(&mut AriseSettings)>,
    mut arise_effect_query: Query<(&mut ParticleSpawnerData), With<AriseEffect>>,
    mut paused_state: ResMut<NextState<PauseState>>,
) {
    let mut summon = false;
    for (action) in &input_query {
        if (action.just_pressed(&Action::Special)) {
            summon = true;
        }
    }

    let Ok(mut arise_settings) = arise_settings_query.get_single_mut() else {
        return;
    };

    arise_settings.cooldown.tick(time.delta());
    if (arise_settings.cooldown.finished()) {
        if (summon) {
            arise_settings.cooldown.reset();
        }
    } else {
        summon = false;
    }
    if (summon) {
        paused_state.set(PauseState::Running);
        for (mut effect) in arise_effect_query.iter_mut() {
            effect.enabled = true;
        }
        let player_settings = PlayerSettings {
            knockback: arise_settings.knockback,
            speed: arise_settings.speed,
            damage: arise_settings.damage,
        };
        let mut max_heap = BinaryHeap::new();
        for (entity, transform) in deceased_query.iter() {
            let entdist = EntDist {
                entity,
                dist: transform.translation.x.abs(),
            };
            max_heap.push(entdist);
            if (max_heap.len() > arise_settings.num as usize) {
                max_heap.pop();
            }
        }
        for x in &max_heap {
            if let Ok((entity, transform)) = deceased_query.get(x.entity) {
                if (transform.translation.x > BARRIER_MIN && transform.translation.x < BARRIER_MAX)
                {
                    spawn_player(
                        &mut commands,
                        Vec3::new(
                            transform.translation.x,
                            0.0,
                            rand::thread_rng().gen_range(-0.3..0.3),
                        ),
                        &skelet_asset,
                        &shadow_asset,
                        &player_settings,
                        &mut sprite_params,
                    );
                }
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component)]
pub struct Player;

pub fn spawn_player(
    mut commands: &mut Commands,
    pos: Vec3,
    asset: &SkeletonSprite,
    shadow: &ShadowSprite,
    player_settings: &PlayerSettings,
    mut sprite3d_params: &mut Sprite3dParams,
) {
    let sprite = Sprite3dBuilder {
        image: asset.image.clone(),
        pixels_per_metre: 128.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        pivot: Option::from(Vec2::new(0.35, 0.5)),
        ..default()
    };

    let texture_atlas = TextureAtlas {
        layout: asset.layout.clone(),
        index: 0,
    };
    let mut player = commands.spawn((
        PlayerStateMaschine { attack_time: 0.15 },
        PlayerIdleState { new: true },
        WalkAnim { active: false },
        AnimationManager {
            running: 0,
            new: true,
            done: false,
            animations: vec![
                Animation {
                    start: 0,
                    end: 0,
                    repeating: true,
                    timer: Default::default(),
                },
                Animation {
                    start: 1,
                    end: 3,
                    repeating: false,
                    timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                },
                Animation {
                    start: 4,
                    end: 10,
                    repeating: true,
                    timer: Timer::new(Duration::from_secs_f32(0.12), TimerMode::Repeating),
                },
            ],
        },
        Transform::from_translation(pos),
        RigidBody::Dynamic,
        Collider::circle(0.5),
        Controllable {
            speed: player_settings.speed,
        },
        get_player_collision_layers(),
        Direction { direction: -1.0 },
        Opfer {
            hit_layer: 1,
            hits: VecDeque::new(),
            knockback_multiplier: -1.0,
        },
        Health::from_health(1.0),
        Hitter {
            single: false,
            knockback: player_settings.knockback,
            damage: player_settings.damage,
            hit_box: Vec2::new(1.0, 1.0),
            offset: Vec2::new(0.5, 0.0),
            hit_mask: 1,
            spatial_query_filter: SpatialQueryFilter::from_mask(LayerMask::from(GameLayer::Enemy)),
        },
        LockedAxes::ROTATION_LOCKED,
        MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
    ));
    player
        .insert((
            FancyWalk::default(),
            Visibility::default(),
            SceneObject,
            Player,
            Barrier {
                min: BARRIER_MIN,
                max: BARRIER_MAX,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                sprite.bundle_with_atlas(&mut sprite3d_params, texture_atlas),
                Transform::from_rotation(Quat::from_rotation_y(PI)),
            ));
        });

    let shadow_sprite = Sprite3dBuilder {
        image: shadow.image.clone(),
        pixels_per_metre: 128.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };
    let player_id = player.id();

    commands.spawn((
        SceneObject,
        Transform::from_xyz(pos.x, -0.5, pos.z - 0.1).with_scale(Vec3::new(0.5, 0.1, 0.5)),
        Shadow { caster: player_id },
        shadow_sprite.bundle(&mut sprite3d_params),
    ));
}

#[derive(Component)]
pub struct DeceasedSpawnPoint {
    pub enemy_type: EnemyType,
}
#[derive(Component)]
pub struct Deceased {}

pub fn spawn_deceased(
    mut commands: Commands,
    enemy_asset: Res<EnemySprite>,
    mut sprite_params: Sprite3dParams,
    spawn_point_query: Query<(&Transform, &DeceasedSpawnPoint, Entity)>,
) {
    for (transform, deceased, entity) in spawn_point_query.iter() {
        let texture_atlas = TextureAtlas {
            layout: enemy_asset.layout.clone(),
            index: 0,
        };
        let sprite = Sprite3dBuilder {
            image: enemy_asset.image.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            pivot: Option::from(Vec2::new(0.4, 0.5)),
            ..default()
        };
        let random = rand::thread_rng().gen_range(-0.1..0.1);
        match deceased.enemy_type {
            EnemyType::BASIC => {
                commands.spawn((
                    SceneObject {},
                    Transform::from_translation(Vec3::new(
                        transform.translation.x,
                        -0.5,
                        0.5 + random,
                    ))
                    .with_rotation(Quat::from_rotation_z(PI / 2.0)),
                    Deceased {},
                    sprite.bundle_with_atlas(&mut sprite_params, texture_atlas.clone()),
                ));
            }
            EnemyType::BIG => {
                commands.spawn((
                    SceneObject {},
                    Transform::from_translation(Vec3::new(
                        transform.translation.x,
                        -0.5,
                        0.5 + random,
                    ))
                    .with_scale(Vec3::splat(1.4))
                    .with_rotation(Quat::from_rotation_z(PI / 2.0)),
                    Deceased {},
                    sprite.bundle_with_atlas(&mut sprite_params, texture_atlas.clone()),
                ));
            }
            EnemyType::FAST => {
                commands.spawn((
                    SceneObject {},
                    Transform::from_translation(Vec3::new(
                        transform.translation.x,
                        -0.5,
                        0.5 + random,
                    ))
                    .with_scale(Vec3::splat(0.6))
                    .with_rotation(Quat::from_rotation_z(PI / 2.0)),
                    Deceased {},
                    sprite.bundle_with_atlas(&mut sprite_params, texture_atlas.clone()),
                ));
            }
            EnemyType::BOSS => {
                commands.spawn((
                    SceneObject {},
                    Transform::from_translation(Vec3::new(
                        transform.translation.x,
                        -0.5,
                        0.5 + random,
                    ))
                    .with_scale(Vec3::splat(2.0))
                    .with_rotation(Quat::from_rotation_z(PI / 2.0)),
                    Deceased {},
                    sprite.bundle_with_atlas(&mut sprite_params, texture_atlas.clone()),
                ));
            }
        }
        commands.entity(entity).despawn();
    }
}
// pub fn die_system(mut commands: Commands, query: Query<Entity, (With<Dead>, With<Controllable>)>) {
//     for entity in query.iter() {
//         commands.entity(entity).despawn_recursive();
//     }
// }
