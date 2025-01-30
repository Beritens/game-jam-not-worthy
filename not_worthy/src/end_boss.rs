use crate::animation::AnimationManager;
use crate::asset_load::{BossSprite, EnemySounds, UIAssets};
use crate::combat::{CombatSet, Dead, Direction, Hitter, Hitting};
use crate::enemy::{AttackingHit, FinishedAttack, HitComposer, Walking};
use crate::game_manager::Scorer;
use crate::game_state::{GameState, PauseState};
use crate::level_loading::SceneObject;
use crate::movement::GameLayer;
use crate::spawning::Enemy;
use crate::summoning::DeceasedSpawnPoint;
use avian2d::prelude::{LayerMask, LinearVelocity, SpatialQueryFilter};
use bevy::app::{App, FixedUpdate, Plugin, PreUpdate, Update};
use bevy::audio::{AudioPlayer, PlaybackMode, PlaybackSettings};
use bevy::prelude::{
    default, in_state, AlphaMode, Changed, Commands, Component, DespawnRecursiveExt, Entity,
    IntoSystemConfigs, Query, Res, SystemSet, TextureAtlas, Time, Transform, Vec2, Vec3, With,
};
use bevy::time::{Timer, TimerMode};
use bevy_sprite3d::{Sprite3d, Sprite3dBuilder, Sprite3dParams};
use rand::Rng;
use std::time::Duration;

pub struct BossPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BossBehaviorSet;
impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        //delete has to come before everything else to avoid panics (could also use try_insert)

        app.add_systems(
            PreUpdate,
            ((
                (boss_walking_state_system, boss_attacking_state_system).in_set(BossBehaviorSet),
                boss_dead_state_system.before(BossBehaviorSet),
            )
                .run_if(in_state(GameState::InGame)),)
                .run_if(in_state(PauseState::Running)),
        );

        app.add_systems(
            Update,
            ((stomp_attack_system, do_the_stomp_system)
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(PauseState::Running))),
        );
    }
}

#[derive(Component)]
pub struct BossWalkingState {
    pub new: bool,
}

#[derive(Component)]
pub struct BossAttackingState {
    pub new: bool,
}

#[derive(Component)]
pub struct BossDeadState {
    pub new: bool,
}

#[derive(Component)]
pub struct BossWalkStateComposer {
    pub timer: Timer,
}

fn boss_walking_state_system(
    mut commands: Commands,
    time: Res<Time>,
    mut state_query: Query<(&mut BossWalkingState, &mut BossWalkStateComposer, Entity)>,
    dead_query: Query<(&Dead)>,
    mut animation_query: Query<&mut AnimationManager>,
) {
    for (mut state, mut composer, entity) in state_query.iter_mut() {
        if (state.new) {
            composer.timer.reset();
            commands.entity(entity).insert(Walking {});
            state.new = false;
            if let Ok(mut anim) = animation_query.get_mut(entity) {
                anim.running = 3;
                anim.new = true;
            }
        }
        composer.timer.tick(time.delta());
        if let Ok(dead) = dead_query.get(entity) {
            commands.entity(entity).remove::<BossWalkingState>();
            commands.entity(entity).insert(BossDeadState { new: true });
            continue;
        }

        if (composer.timer.just_finished()) {
            commands.entity(entity).remove::<Walking>();
            commands.entity(entity).remove::<BossWalkingState>();
            commands
                .entity(entity)
                .insert(BossAttackingState { new: true });
        }
    }
}
fn boss_attacking_state_system(
    mut commands: Commands,
    mut state_query: Query<(&mut BossAttackingState, Entity)>,
    attack_finished_query: Query<(&FinishedAttack)>,
    dead_query: Query<(&Dead)>,
) {
    for (mut state, entity) in state_query.iter_mut() {
        if (state.new) {
            state.new = false;
            let attack = rand::thread_rng().gen_range(0..=1);
            match attack {
                0 => {
                    commands.entity(entity).insert(AttackingStomp { new: true });
                }

                1 => {
                    commands.entity(entity).insert(AttackingHit { new: true });
                }
                _ => {}
            }
        }
        if let Ok(dead) = dead_query.get(entity) {
            commands.entity(entity).remove::<BossAttackingState>();
            commands.entity(entity).insert(BossDeadState { new: true });
            continue;
        }

        if let Ok(finished_attack) = attack_finished_query.get(entity) {
            commands.entity(entity).remove::<FinishedAttack>();
            commands.entity(entity).remove::<AttackingStomp>();
            commands
                .entity(entity)
                .insert(BossWalkingState { new: true });
            continue;
        }
    }
}
fn boss_dead_state_system(
    mut commands: Commands,
    state_query: Query<(&BossDeadState, &Enemy, &Transform, Entity)>,
    mut scorer_query: Query<&mut Scorer>,
) {
    let Ok(mut scorer) = scorer_query.get_single_mut() else {
        return;
    };
    for (state, enemy, transform, entity) in state_query.iter() {
        commands.entity(entity).despawn_recursive();
        scorer.incoming.push_back(enemy.points);
        let enem_type = enemy.enemy_type.clone();
        commands.spawn((
            SceneObject {},
            DeceasedSpawnPoint {
                enemy_type: enem_type,
            },
            Transform::from_translation(transform.translation),
        ));
    }
}

#[derive(Component)]
pub struct StompComposer {
    pub(crate) timer: Timer,
    pub(crate) after_timer: Timer,
    pub(crate) delay: f32,
    pub(crate) delay_delta: f32,
    pub(crate) state: usize,
}

#[derive(Component)]
pub struct AttackingStomp {
    new: bool,
}

#[derive(Component)]
pub struct StompThing {
    display_delay: Timer,
    delay: Timer,
    delete_timer: Timer,
}

fn do_the_stomp_system(
    time: Res<Time>,
    mut stomp_query: Query<(&mut StompThing, &Transform, Entity)>,
    mut sprite_query: Query<&mut Sprite3d>,
    mut commands: Commands,
    attack_asset: Res<BossSprite>,
    mut sprite_params: Sprite3dParams,
) {
    for (mut stomp_thing, transform, entity) in stomp_query.iter_mut() {
        let telegraphSprite = Sprite3dBuilder {
            image: attack_asset.stomp_attack.clone(),
            pixels_per_metre: 128.0,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };
        let texture_atlas = TextureAtlas {
            layout: attack_asset.stomp_layout.clone(),
            index: 0,
        };
        stomp_thing.delete_timer.tick(time.delta());
        stomp_thing.delay.tick(time.delta());
        stomp_thing.display_delay.tick(time.delta());
        if (stomp_thing.display_delay.just_finished()) {
            commands
                .entity(entity)
                .insert(telegraphSprite.bundle_with_atlas(&mut sprite_params, texture_atlas));
        }
        if (stomp_thing.delay.just_finished()) {
            if let Ok(mut sprite) = sprite_query.get_mut(entity) {
                sprite.texture_atlas.as_mut().unwrap().index = 1;
            }
            commands.entity(entity).insert((
                Hitter {
                    knockback: 0.0,
                    damage: 100000.0,
                    hit_box: Vec2::new(0.8, 10.0),
                    offset: Vec2::ZERO,
                    hit_mask: 2,
                    spatial_query_filter: SpatialQueryFilter::from_mask(LayerMask::from(
                        GameLayer::Player,
                    )),
                    single: false,
                },
                Direction { direction: 1.0 },
                Hitting {},
            ));
        }
        if (stomp_thing.delete_timer.finished()) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
fn stomp_attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        &mut StompComposer,
        &mut LinearVelocity,
        &mut AttackingStomp,
        &Transform,
        Entity,
    )>,
    mut animation_query: Query<&mut AnimationManager>,
) {
    for (mut stomp_composer, mut linear_velocity, mut attacking_stomp, transform, entity) in
        query.iter_mut()
    {
        if (attacking_stomp.new) {
            linear_velocity.x = 0.0;
            stomp_composer.timer.reset();
            stomp_composer.after_timer.reset();
            stomp_composer.state = 0;
            attacking_stomp.new = false;
            if let Ok(mut anim) = animation_query.get_mut(entity) {
                anim.running = 4;
                anim.new = true;
            }
            let offset = rand::thread_rng().gen_range(-1.5..1.5);
            let dist_between: f32 = 3.0;

            for i in (-4..=4) {
                commands.spawn((
                    SceneObject,
                    StompThing {
                        display_delay: Timer::new(
                            Duration::from_secs_f32((i as f32).abs() * stomp_composer.delay_delta),
                            TimerMode::Once,
                        ),
                        delay: Timer::new(
                            Duration::from_secs_f32(
                                stomp_composer.delay
                                    + (i as f32).abs() * stomp_composer.delay_delta,
                            ),
                            TimerMode::Once,
                        ),
                        delete_timer: Timer::new(Duration::from_secs_f32(1.5), TimerMode::Once),
                    },
                    Transform::from_translation(Vec3::new(
                        transform.translation.x + offset + i as f32 * dist_between,
                        0.0,
                        0.5,
                    ))
                    .with_scale(Vec3::splat(1.3)),
                ));
            }
        }
        match stomp_composer.state {
            0 => {
                stomp_composer.timer.tick(time.delta());
                if (stomp_composer.timer.just_finished()) {
                    if let Ok(mut anim) = animation_query.get_mut(entity) {
                        anim.running = 5;
                        anim.new = true;
                    }
                    stomp_composer.state = 1;
                }
            }
            1 => {
                stomp_composer.after_timer.tick(time.delta());
                if (stomp_composer.after_timer.just_finished()) {
                    commands.entity(entity).insert(FinishedAttack {});
                    commands.entity(entity).remove::<AttackingStomp>();
                    commands.entity(entity).remove::<AttackingHit>();
                }
            }
            _ => {}
        }
    }
    //for entities with Attacking
    //check if there are enems in attack state without
}
