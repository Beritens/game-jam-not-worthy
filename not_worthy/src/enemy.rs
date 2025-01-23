use crate::animation::AnimationManager;
use crate::asset_load::EnemySprite;
use crate::combat::{hit_test, CombatSet, Dead, Direction, Hitter, Hitting, Opfer, Stunned};
use crate::game_state::GameState;
use crate::movement::GameLayer;
use crate::summoning::spawn_deceased;
use avian2d::prelude::{LayerMask, LinearVelocity, SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, PreUpdate, Update};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{
    in_state, Bundle, Commands, Component, Entity, IntoSystemConfigs, Query, Res, SystemSet, Time,
    TimerMode, Transform, Vec2, Vec3Swizzles, With,
};
use bevy::time::Timer;
use bevy_sprite3d::Sprite3dParams;
use std::time::Duration;

pub struct EnemyPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnemBehaviorSet;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                walk_to_target.before(CombatSet),
                check_attack_system.before(CombatSet),
                attack_system.before(CombatSet),
            ),
        );
        //delete has to come before everything else to avoid panics (could also use try_insert)

        app.add_systems(
            PreUpdate,
            ((
                (
                    basic_enem_active_state_system,
                    basic_enem_stunned_state_system,
                    basic_enem_attack_state_system,
                )
                    .in_set(EnemBehaviorSet),
                basic_enem_dead_state_system.before(EnemBehaviorSet),
            )
                .run_if(in_state(GameState::Main)),),
        );
        // add things to your app here
    }
}

#[derive(Component)]
pub struct Target {
    pub pos: Vec2,
}

#[derive(Component)]
pub struct Walking {}
#[derive(Component)]
pub struct Walker {
    pub speed: f32,
}

fn walk_to_target(
    mut query: Query<
        (
            &mut LinearVelocity,
            &mut Direction,
            &Target,
            &Transform,
            &Walker,
        ),
        With<Walking>,
    >,
) {
    for (mut linear_vel, mut direction, target, transform, walker) in query.iter_mut() {
        direction.direction = (target.pos.x - transform.translation.x).signum();
        linear_vel.x = direction.direction * walker.speed;
    }
}

//for hitter
fn check_attack_system(
    mut commands: Commands,
    attack_check_query: Query<(&Hitter, &Direction, &Transform, Entity), With<AttackCheck>>,
    spatial_query: SpatialQuery,
    opfer_query: Query<(&Opfer)>,
) {
    for (hitter, direction, transform, entity) in attack_check_query.iter() {
        let hit: bool = hit_test(
            &spatial_query,
            hitter,
            direction,
            transform.translation.xy(),
            &opfer_query,
            &hitter.spatial_query_filter,
        );
        if (hit) {
            commands.entity(entity).insert(AttackReady {});
        }
    }
}

#[derive(Component)]
pub struct AttackingHit {
    new: bool,
}
#[derive(Component)]
pub struct HitComposer {
    pub timer: Timer,
    pub after_timer: Timer,
    pub state: i32,
}

//basic hit attack
fn attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&mut HitComposer, &mut AttackingHit, Entity)>,
    mut animation_query: Query<&mut AnimationManager>,
) {
    for (mut hit_composer, mut attacking_hit, entity) in query.iter_mut() {
        if (attacking_hit.new) {
            hit_composer.timer.reset();
            hit_composer.after_timer.reset();
            hit_composer.state = 0;
            attacking_hit.new = false;
            if let Ok(mut anim) = animation_query.get_mut(entity) {
                anim.running = 1;
                anim.new = true;
            }
        }
        match hit_composer.state {
            0 => {
                hit_composer.timer.tick(time.delta());
                if (hit_composer.timer.just_finished()) {
                    commands.entity(entity).insert(Hitting {});
                    if let Ok(mut anim) = animation_query.get_mut(entity) {
                        anim.running = 2;
                        anim.new = true;
                    }
                    hit_composer.state = 1;
                }
            }
            1 => {
                hit_composer.after_timer.tick(time.delta());
                if (hit_composer.after_timer.just_finished()) {
                    commands.entity(entity).insert(FinishedAttack {});
                    commands.entity(entity).remove::<AttackingHit>();
                }
            }
            _ => {}
        }
    }
    //for entities with Attacking
    //check if there are enems in attack state without
}

#[derive(Component)]
pub struct AttackCheck {}

#[derive(Component)]
pub struct AttackReady {}

#[derive(Component)]
pub struct FinishedAttack {}

pub enum AttackType {
    BasicAttack,
}
#[derive(Component)]
pub struct BasicEnemStateMachine {
    pub stunne_time: f32,
    pub basic_attack: AttackType,
}

#[derive(Component)]
pub struct BacicEnemActiveState {
    pub new: bool,
}

fn basic_enem_active_state_system(
    mut commands: Commands,
    mut active_state_query: Query<(&mut BacicEnemActiveState, Entity)>,
    stunned_query: Query<(&Stunned, Entity), With<BacicEnemActiveState>>,
    attack_ready_query: Query<(&AttackReady)>,
    dead_query: Query<&Dead>,
    mut animation_query: Query<&mut AnimationManager>,
) {
    for (mut state, entity) in active_state_query.iter_mut() {
        if (state.new) {
            if let Ok(mut anim) = animation_query.get_mut(entity) {
                basic_enem_active_on_enter(&mut commands, entity, &mut anim);
            }
            state.new = false;
        }

        if let Ok(dead) = dead_query.get(entity) {
            commands
                .entity(entity)
                .insert(BacicEnemDeadState { new: true });
            basic_enem_active_on_exit(&mut commands, entity);
            continue;
        }
        if let Ok(attack_ready) = attack_ready_query.get(entity) {
            basic_enem_active_on_exit(&mut commands, entity);
            commands
                .entity(entity)
                .insert(BacicEnemAttackState { new: true });
            continue;
        }
    }
    for (stunned, entity) in stunned_query.iter() {
        basic_enem_active_on_exit(&mut commands, entity);
        commands
            .entity(entity)
            .insert(BacisEnemStunnedState { new: true });
        continue;
    }
}

fn basic_enem_active_on_enter(
    mut commands: &mut Commands,
    entity: Entity,
    anim: &mut AnimationManager,
) {
    anim.running = 3;
    anim.new = true;
    commands.entity(entity).insert(Walking {});
    commands.entity(entity).insert(AttackCheck {});
}
fn basic_enem_active_on_exit(mut commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<BacicEnemActiveState>();
    commands.entity(entity).remove::<AttackCheck>();
    commands.entity(entity).remove::<AttackReady>();
    commands.entity(entity).remove::<Walking>();
}

#[derive(Component)]
pub struct BacisEnemStunnedState {
    new: bool,
}

#[derive(Component)]
pub struct StunnedTimer {
    timer: Timer,
}

fn basic_enem_stunned_state_system(
    time: Res<Time>,
    mut commands: Commands,
    mut stunned_state_query: Query<(&mut BacisEnemStunnedState, Entity)>,
    mut dead_query: Query<(&Dead)>,
    mut stunned_timer_query: Query<(&mut StunnedTimer)>,
    mut anim_query: Query<&mut AnimationManager>,
    stunned_query: Query<(&Stunned)>,
    state_machine_query: Query<(&BasicEnemStateMachine)>,
) {
    for (mut state, entity) in stunned_state_query.iter_mut() {
        if (state.new) {
            if let Ok(state_machine) = state_machine_query.get(entity) {
                if let Ok(mut anim) = anim_query.get_mut(entity) {
                    basic_enem_stunned_on_enter(
                        &mut commands,
                        entity,
                        state_machine.stunne_time,
                        &mut anim,
                    );
                }
                state.new = false;
            }
        }

        if let Ok(dead) = dead_query.get(entity) {
            commands
                .entity(entity)
                .insert(BacicEnemDeadState { new: true });
            basic_enem_stunned_on_exit(&mut commands, entity);
            continue;
        }
        if let Ok(mut stunned_timer) = stunned_timer_query.get_mut(entity) {
            if let Ok(stunned) = stunned_query.get(entity) {
                commands.entity(entity).remove::<Stunned>();
                stunned_timer.timer.reset();
            }
            stunned_timer.timer.tick(time.delta());
            if stunned_timer.timer.finished() {
                commands
                    .entity(entity)
                    .insert(BacicEnemActiveState { new: true });
                basic_enem_stunned_on_exit(&mut commands, entity);
                continue;
            }
        }
    }
}

fn basic_enem_stunned_on_enter(
    mut commands: &mut Commands,
    entity: Entity,
    stunned_time: f32,
    anim: &mut AnimationManager,
) {
    commands.entity(entity).insert(StunnedTimer {
        timer: Timer::new(Duration::from_secs_f32(stunned_time), TimerMode::Once),
    });
    anim.running = 0;
    anim.new = true;
    commands.entity(entity).remove::<Stunned>();
}
fn basic_enem_stunned_on_exit(mut commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<BacisEnemStunnedState>();
    commands.entity(entity).remove::<StunnedTimer>();
}

#[derive(Component)]
pub struct BacicEnemDeadState {
    new: bool,
}
fn basic_enem_dead_state_system(
    mut commands: Commands,
    mut dead_state_query: Query<(&BacicEnemDeadState, Entity, &Transform)>,
    hero_asset: Res<EnemySprite>,
    mut sprite_params: Sprite3dParams,
) {
    for (mut state, entity, transform) in dead_state_query.iter() {
        commands.entity(entity).despawn_recursive();
        spawn_deceased(
            &mut commands,
            transform.translation.x,
            &hero_asset.image,
            &hero_asset.layout,
            &mut sprite_params,
        );
    }
}

#[derive(Component)]
pub struct BacicEnemAttackState {
    new: bool,
}
fn basic_enem_attack_state_system(
    mut commands: Commands,
    // stunned_query: Query<(&Stunned)>,
    attack_finished_query: Query<(&FinishedAttack)>,
    dead_query: Query<&Dead>,
    mut attack_state_query: Query<(
        &mut BacicEnemAttackState,
        &BasicEnemStateMachine,
        Entity,
        &Transform,
    )>,
) {
    for (mut state, state_machine, entity, transform) in attack_state_query.iter_mut() {
        if (state.new) {
            state.new = false;
            basic_enem_attack_on_enter(&mut commands, entity, &state_machine.basic_attack);
        }
        // if let Ok(stunned) = stunned_query.get(entity) {
        //     basic_enem_attack_on_exit(&mut commands, entity, &state_machine.basic_attack);
        //     commands
        //         .entity(entity)
        //         .insert(BacisEnemStunnedState { new: true });
        // }
        if let Ok(dead) = dead_query.get(entity) {
            commands
                .entity(entity)
                .insert(BacicEnemDeadState { new: true });
            basic_enem_attack_on_exit(&mut commands, entity, &state_machine.basic_attack);
            continue;
        }

        if let Ok(finished_attack) = attack_finished_query.get(entity) {
            basic_enem_attack_on_exit(&mut commands, entity, &state_machine.basic_attack);
            commands
                .entity(entity)
                .insert(BacicEnemActiveState { new: true });
            continue;
        }
    }
}

fn basic_enem_attack_on_enter(
    mut commands: &mut Commands,
    entity: Entity,
    attack_type: &AttackType,
) {
    match attack_type {
        AttackType::BasicAttack => {
            commands.entity(entity).insert(AttackingHit { new: true });
        }
    }
}

fn basic_enem_attack_on_exit(
    mut commands: &mut Commands,
    entity: Entity,
    attack_type: &AttackType,
) {
    commands.entity(entity).remove::<BacicEnemAttackState>();
    commands.entity(entity).remove::<FinishedAttack>();

    match attack_type {
        AttackType::BasicAttack => {
            commands.entity(entity).remove::<AttackingHit>();
        }
    }
}
