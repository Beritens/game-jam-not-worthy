use crate::animation::{Animation, AnimationManager};
use crate::asset_load::{BossSprite, EnemySprite, ShadowSprite, SkeletonSprite};
use crate::combat::{Direction, Health, Hitter, Opfer};
use crate::end_boss::{BossWalkStateComposer, BossWalkingState, StompComposer};
use crate::enemy::{
    AttackType, BacicEnemActiveState, BasicEnemStateMachine, HitComposer, Target, Walker,
};
use crate::game_state::{GameState, PauseState};
use crate::input_manager::{Action, BasicControl};
use crate::level_loading::SceneObject;
use crate::movement::{get_enemy_collision_layers, GameLayer};
use crate::player_states::WalkAnim;
use crate::shadows::Shadow;
use crate::summoning::{spawn_player, Deceased};
use crate::ui_stuff::HealthBarInitiator;
use avian2d::collision::{Collider, LayerMask};
use avian2d::prelude::{LockedAxes, MassPropertiesBundle, RigidBody, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::audio::AudioPlayer;
use bevy::image::Image;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{
    default, in_state, AlphaMode, BuildChildren, Bundle, ChildBuild, Circle, Commands, Component,
    Entity, IntoSystemConfigs, Query, Res, TextureAtlas, Time, Transform, Visibility, With,
};
use bevy::time::{Timer, TimerMode};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use leafwing_input_manager::action_state::ActionState;
use rand::Rng;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Duration;

pub struct SpawningPlugin;

impl Plugin for SpawningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                continuous_spawning_system,
                spawn_enemy_system,
                spawn_fast_enemy_system,
                spawn_big_enemy_system,
                spawn_end_boss_system,
            )
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(PauseState::Running))),
        );
    }
}

#[derive(Clone)]
pub enum EnemyType {
    BASIC,
    FAST,
    BIG,
    BOSS,
}

#[derive(Component)]
pub struct EnemySpawner {
    pub once: bool,
    pub inactive: Timer,
    pub preheat: f32,
    pub min: f32,
    pub max: f32,
    pub factor: f32,
    pub timer: Timer,
    pub enemy_type: EnemyType,
}

#[derive(Component)]
pub struct BasicEnemSpawnPoint;

#[derive(Component)]
pub struct FastEnemSpawnPoint;

#[derive(Component)]
pub struct BigEnemSpawnPoint;

#[derive(Component)]
pub struct BossSpawnPoint;

#[derive(Component)]
pub struct TimeTraveler {
    pub time_travel: f32,
}

fn continuous_spawning_system(
    time: Res<Time>,
    mut commands: Commands,
    mut spawner_query: Query<(&mut EnemySpawner, &Transform, Entity)>,
) {
    for (mut spawner, transform, entity) in spawner_query.iter_mut() {
        spawner.inactive.tick(time.delta());
        if (!spawner.inactive.finished()) {
            continue;
        }
        if (spawner.preheat > 0.0) {
            let preheat = spawner.preheat;
            let num = (spawner.preheat / spawner.timer.duration().as_secs_f32()).floor() as i32;
            let reminder = spawner.preheat - spawner.timer.duration().as_secs_f32() * num as f32;
            spawner.timer.set_elapsed(Duration::from_secs_f32(reminder));
            spawner.preheat = 0.0;
            for i in (0..=num) {
                let mut point = commands.spawn((
                    SceneObject {},
                    Transform::from_translation(
                        transform.translation + Vec3::Z * rand::thread_rng().gen_range(-0.3..0.3),
                    ),
                    TimeTraveler {
                        time_travel: i as f32 * spawner.timer.duration().as_secs_f32() + reminder,
                    },
                ));
                match &spawner.enemy_type {
                    EnemyType::BASIC => {
                        point.insert(BasicEnemSpawnPoint);
                    }
                    EnemyType::FAST => {
                        point.insert(FastEnemSpawnPoint);
                    }

                    EnemyType::BIG => {
                        point.insert(BigEnemSpawnPoint);
                    }
                    EnemyType::BOSS => {
                        point.insert(BossSpawnPoint);
                    }
                }
                if (spawner.once) {
                    break;
                }
            }
            if (spawner.once && num > 0) {
                commands.entity(entity).despawn();
                continue;
            }
        }
        spawner.timer.tick(time.delta());
        if (spawner.timer.just_finished()) {
            let mut point = commands.spawn((
                SceneObject,
                Transform::from_translation(
                    transform.translation + Vec3::Z * rand::thread_rng().gen_range(-0.3..0.3),
                ),
            ));
            match &spawner.enemy_type {
                EnemyType::BASIC => {
                    point.insert(BasicEnemSpawnPoint);
                }
                EnemyType::FAST => {
                    point.insert(FastEnemSpawnPoint);
                }
                EnemyType::BIG => {
                    point.insert(BigEnemSpawnPoint);
                }
                EnemyType::BOSS => {
                    point.insert(BossSpawnPoint);
                }
            }
            if (spawner.once) {
                commands.entity(entity).despawn();
                continue;
            }

            // spawn_player(
            //     &mut commands,
            //     transform.translation + Vec3::Z * rand::thread_rng().gen_range(-0.3..0.3),
            //     &enemy_asset,
            //     &mut sprite_params,
            // );
            let duration = spawner.timer.duration().as_secs_f32();
            let factor = spawner.factor;
            let min = spawner.min;
            let max = spawner.max;
            let clamped_duration = (duration * factor).clamp(min, max);
            spawner
                .timer
                .set_duration(Duration::from_secs_f32(clamped_duration));
        }
    }
}
#[derive(Component)]
pub struct Enemy {
    pub points: i32,
    pub enemy_type: EnemyType,
}

#[derive(Component)]
pub struct TimeTravel {
    pub time: f32,
}

pub fn spawn_enemy_system(
    mut commands: Commands,
    enemy_asset: Res<EnemySprite>,
    shadw_asset: Res<ShadowSprite>,
    mut sprite_params: Sprite3dParams,
    spawn_point_query: Query<
        (&Transform, Option<&TimeTraveler>, Entity),
        With<BasicEnemSpawnPoint>,
    >,
) {
    for (transform, time_traveler, entity) in spawn_point_query.iter() {
        commands.entity(entity).despawn();
        let sprite = Sprite3dBuilder {
            image: enemy_asset.image.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            pivot: Option::from(Vec2::new(0.35, 0.5)),
            ..default()
        };

        let texture_atlas = TextureAtlas {
            layout: enemy_asset.layout.clone(),
            index: 0,
        };
        let hit_composer = HitComposer {
            timer: Timer::new(Duration::from_secs_f32(0.4), TimerMode::Once),
            after_timer: Timer::new(Duration::from_secs_f32(0.2), TimerMode::Once),
            state: 0,
        };
        let mut enemy = commands.spawn((
            Transform::from_translation(transform.translation),
            RigidBody::Dynamic,
            get_enemy_collision_layers(),
            Target {
                pos: Vec2::new(0.0, 0.0),
            },
            BasicEnemStateMachine {
                cooldown_time: 1.0,
                stunne_time: 1.0,
                basic_attack: AttackType::BasicAttack,
            },
            Hitter {
                single: false,
                knockback: 3.0,
                damage: 1.0,
                hit_box: Vec2::new(0.5, 1.0),
                offset: Vec2::new(0.5, 0.0),
                hit_mask: 2,
                spatial_query_filter: SpatialQueryFilter::from_mask(LayerMask::from(
                    GameLayer::Player,
                )),
            },
            Direction { direction: 1.0 },
            BacicEnemActiveState { new: true },
            Walker { speed: 2.0 },
            Health::from_health(4.0),
            Opfer {
                hit_layer: 0,
                hits: VecDeque::new(),
                knockback_multiplier: 1.0,
            },
            Collider::circle(0.5),
            LockedAxes::ROTATION_LOCKED,
            MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        ));

        enemy
            .insert((
                Enemy {
                    points: 2,
                    enemy_type: EnemyType::BASIC,
                },
                SceneObject,
                hit_composer,
                Visibility::default(),
                AnimationManager {
                    running: 3,
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
                            end: 1,
                            repeating: false,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                        Animation {
                            start: 2,
                            end: 3,
                            repeating: false,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                        Animation {
                            start: 4,
                            end: 10,
                            repeating: true,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                    ],
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    sprite.bundle_with_atlas(&mut sprite_params, texture_atlas),
                    Transform::from_rotation(Quat::from_rotation_y(PI)),
                ));
            });
        if let Some(time_travel) = time_traveler {
            enemy.insert(TimeTravel {
                time: time_travel.time_travel,
            });
        }

        let shadow_sprite = Sprite3dBuilder {
            image: shadw_asset.image.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };
        let enemy_id = enemy.id();

        commands.spawn((
            SceneObject,
            Transform::from_xyz(transform.translation.x, -0.5, transform.translation.z - 0.1)
                .with_scale(Vec3::new(0.5, 0.1, 0.5)),
            Shadow { caster: enemy_id },
            shadow_sprite.bundle(&mut sprite_params),
        ));
    }
}

pub fn spawn_fast_enemy_system(
    mut commands: Commands,
    enemy_asset: Res<EnemySprite>,
    shadow_asset: Res<ShadowSprite>,
    mut sprite_params: Sprite3dParams,
    spawn_point_query: Query<(&Transform, Option<&TimeTraveler>, Entity), With<FastEnemSpawnPoint>>,
) {
    for (transform, time_traveler, entity) in spawn_point_query.iter() {
        commands.entity(entity).despawn();
        let sprite = Sprite3dBuilder {
            image: enemy_asset.image.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            pivot: Option::from(Vec2::new(0.35, 0.5)),
            ..default()
        };

        let texture_atlas = TextureAtlas {
            layout: enemy_asset.layout.clone(),
            index: 0,
        };
        let hit_composer = HitComposer {
            timer: Timer::new(Duration::from_secs_f32(0.4), TimerMode::Once),
            after_timer: Timer::new(Duration::from_secs_f32(0.2), TimerMode::Once),
            state: 0,
        };
        let mut enemy = commands.spawn((
            Transform::from_translation(transform.translation).with_scale(Vec3::splat(0.6)),
            RigidBody::Dynamic,
            get_enemy_collision_layers(),
            Target {
                pos: Vec2::new(0.0, 0.0),
            },
            BasicEnemStateMachine {
                cooldown_time: 1.0,
                stunne_time: 1.0,
                basic_attack: AttackType::BasicAttack,
            },
            Direction { direction: 1.0 },
            BacicEnemActiveState { new: true },
            Walker { speed: 4.0 },
            Health::from_health(1.0),
            Opfer {
                hit_layer: 0,
                hits: VecDeque::new(),
                knockback_multiplier: 1.0,
            },
            Collider::circle(0.5),
            LockedAxes::ROTATION_LOCKED,
            MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        ));

        enemy
            .insert((
                Enemy {
                    points: 1,
                    enemy_type: EnemyType::FAST,
                },
                SceneObject,
                hit_composer,
                Visibility::default(),
                AnimationManager {
                    running: 3,
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
                            end: 1,
                            repeating: false,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                        Animation {
                            start: 2,
                            end: 3,
                            repeating: false,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                        Animation {
                            start: 4,
                            end: 10,
                            repeating: true,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                    ],
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    sprite.bundle_with_atlas(&mut sprite_params, texture_atlas),
                    Transform::from_rotation(Quat::from_rotation_y(PI)),
                ));
            });
        if let Some(time_travel) = time_traveler {
            enemy.insert(TimeTravel {
                time: time_travel.time_travel,
            });
        }

        let shadow_sprite = Sprite3dBuilder {
            image: shadow_asset.image.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };
        let enemy_id = enemy.id();

        commands.spawn((
            SceneObject,
            Transform::from_xyz(transform.translation.x, -0.5, transform.translation.z - 0.1)
                .with_scale(Vec3::new(0.3, 0.06, 0.3)),
            Shadow { caster: enemy_id },
            shadow_sprite.bundle(&mut sprite_params),
        ));
    }
}

pub fn spawn_big_enemy_system(
    mut commands: Commands,
    enemy_asset: Res<EnemySprite>,
    shadw_asset: Res<ShadowSprite>,
    mut sprite_params: Sprite3dParams,
    spawn_point_query: Query<(&Transform, Option<&TimeTraveler>, Entity), With<BigEnemSpawnPoint>>,
) {
    for (transform, time_traveler, entity) in spawn_point_query.iter() {
        commands.entity(entity).despawn();
        let sprite = Sprite3dBuilder {
            image: enemy_asset.image.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            pivot: Option::from(Vec2::new(0.35, 0.5)),
            ..default()
        };

        let texture_atlas = TextureAtlas {
            layout: enemy_asset.layout.clone(),
            index: 0,
        };
        let hit_composer = HitComposer {
            timer: Timer::new(Duration::from_secs_f32(0.6), TimerMode::Once),
            after_timer: Timer::new(Duration::from_secs_f32(0.8), TimerMode::Once),
            state: 0,
        };
        let mut enemy = commands.spawn((
            Transform::from_translation(transform.translation).with_scale(Vec3::splat(2.0)),
            RigidBody::Dynamic,
            get_enemy_collision_layers(),
            Target {
                pos: Vec2::new(0.0, 0.0),
            },
            BasicEnemStateMachine {
                cooldown_time: 1.0,
                stunne_time: 0.0,
                basic_attack: AttackType::BasicAttack,
            },
            Hitter {
                single: false,
                knockback: 100.0,
                damage: 0.0,
                hit_box: Vec2::new(3.0, 2.0),
                offset: Vec2::new(1.0, 0.0),
                hit_mask: 2,
                spatial_query_filter: SpatialQueryFilter::from_mask(LayerMask::from(
                    GameLayer::Player,
                )),
            },
            Direction { direction: 1.0 },
            BacicEnemActiveState { new: true },
            Walker { speed: 1.2 },
            Health::from_health(300.0),
            Opfer {
                hit_layer: 0,
                hits: VecDeque::new(),
                knockback_multiplier: 0.01,
            },
            Collider::circle(0.5),
            LockedAxes::ROTATION_LOCKED,
            MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        ));

        enemy
            .insert((
                Enemy {
                    points: 10,
                    enemy_type: EnemyType::BIG,
                },
                SceneObject,
                hit_composer,
                Visibility::default(),
                AnimationManager {
                    running: 3,
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
                            end: 1,
                            repeating: false,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                        Animation {
                            start: 2,
                            end: 3,
                            repeating: false,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                        Animation {
                            start: 4,
                            end: 10,
                            repeating: true,
                            timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                        },
                    ],
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    sprite.bundle_with_atlas(&mut sprite_params, texture_atlas),
                    Transform::from_rotation(Quat::from_rotation_y(PI)),
                ));
            });
        if let Some(time_travel) = time_traveler {
            enemy.insert(TimeTravel {
                time: time_travel.time_travel,
            });
        }

        let shadow_sprite = Sprite3dBuilder {
            image: shadw_asset.image.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };
        let enemy_id = enemy.id();

        commands.spawn((
            SceneObject,
            Transform::from_xyz(transform.translation.x, -0.5, transform.translation.z - 0.1)
                .with_scale(Vec3::new(0.5, 0.1, 0.5)),
            Shadow { caster: enemy_id },
            shadow_sprite.bundle(&mut sprite_params),
        ));
    }
}

pub fn spawn_end_boss_system(
    mut commands: Commands,
    boss_asset: Res<BossSprite>,
    shadw_asset: Res<ShadowSprite>,
    mut sprite_params: Sprite3dParams,
    spawn_point_query: Query<(&Transform, Option<&TimeTraveler>, Entity), With<BossSpawnPoint>>,
) {
    for (transform, time_traveler, entity) in spawn_point_query.iter() {
        commands.entity(entity).despawn();
        let sprite = Sprite3dBuilder {
            image: boss_asset.image.clone(),
            pixels_per_metre: 512.0,
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            pivot: Option::from(Vec2::new(0.45, 0.5)),
            ..default()
        };

        let texture_atlas = TextureAtlas {
            layout: boss_asset.layout.clone(),
            index: 0,
        };
        let hit_composer = HitComposer {
            timer: Timer::new(Duration::from_secs_f32(0.4), TimerMode::Once),
            after_timer: Timer::new(Duration::from_secs_f32(0.8), TimerMode::Once),
            state: 0,
        };
        let stomp_composer = StompComposer {
            timer: Timer::new(Duration::from_secs_f32(1.0), TimerMode::Once),
            after_timer: Timer::new(Duration::from_secs_f32(0.8), TimerMode::Once),
            delay: 1.0,
            delay_delta: 0.1,
            state: 0,
        };
        let mut boss = commands.spawn((
            Transform::from_translation(transform.translation).with_scale(Vec3::splat(4.0)),
            RigidBody::Dynamic,
            get_enemy_collision_layers(),
            Target {
                pos: Vec2::new(0.0, 0.0),
            },
            BossWalkStateComposer {
                timer: Timer::new(Duration::from_secs_f32(1.0), TimerMode::Once),
            },
            BossWalkingState { new: true },
            Hitter {
                single: false,
                knockback: 100.0,
                damage: 100.0,
                hit_box: Vec2::new(3.0, 10.0),
                offset: Vec2::new(0.2, 0.0),
                hit_mask: 2,
                spatial_query_filter: SpatialQueryFilter::from_mask(LayerMask::from(
                    GameLayer::Player,
                )),
            },
            Direction { direction: 1.0 },
            Walker { speed: 0.5 },
            Health::from_health(50000.0),
            Opfer {
                hit_layer: 0,
                hits: VecDeque::new(),
                knockback_multiplier: 0.001,
            },
            Collider::circle(0.5),
            LockedAxes::ROTATION_LOCKED,
            MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        ));

        boss.insert((
            Enemy {
                points: 500,
                enemy_type: EnemyType::BOSS,
            },
            SceneObject,
            stomp_composer,
            hit_composer,
            Visibility::default(),
            AnimationManager {
                running: 3,
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
                        start: 8,
                        end: 8,
                        repeating: false,
                        timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                    },
                    Animation {
                        start: 9,
                        end: 11,
                        repeating: false,
                        timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                    },
                    Animation {
                        start: 0,
                        end: 7,
                        repeating: true,
                        timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                    },
                    Animation {
                        start: 12,
                        end: 12,
                        repeating: false,
                        timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                    },
                    Animation {
                        start: 13,
                        end: 14,
                        repeating: false,
                        timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                    },
                ],
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                sprite.bundle_with_atlas(&mut sprite_params, texture_atlas),
                Transform::from_rotation(Quat::from_rotation_y(PI)),
            ));
        });
        if let Some(time_travel) = time_traveler {
            boss.insert(TimeTravel {
                time: time_travel.time_travel,
            });
        }

        let shadow_sprite = Sprite3dBuilder {
            image: shadw_asset.image.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };
        let enemy_id = boss.id();

        commands.spawn((
            SceneObject,
            Transform::from_xyz(transform.translation.x, -0.5, transform.translation.z - 0.1)
                .with_scale(Vec3::new(1.0, 0.2, 1.0)),
            Shadow { caster: enemy_id },
            shadow_sprite.bundle(&mut sprite_params),
        ));
        commands.spawn((
            SceneObject,
            HealthBarInitiator {
                enity: enemy_id,
                name: "A TRUE HERO".to_string(),
            },
        ));
    }
}
