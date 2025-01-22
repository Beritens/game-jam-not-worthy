use crate::animation::{HitAnimationRunning, IdleAnimationRunning, WalkingAnimationRunning};
use crate::combat::{CombatSet, Dead, Hitting, Stunned};
use crate::enemy::{
    AttackCheck, AttackReady, BacicEnemActiveState, BacicEnemAttackState, BacisEnemStunnedState,
    Walking,
};
use crate::game_state::GameState;
use bevy::app::{App, Plugin, PreUpdate, Update};
use bevy::prelude::{
    in_state, Commands, Component, DespawnRecursiveExt, Entity, IntoSystemConfigs, Query, Res,
    SystemSet, Time, Timer, With,
};
use bevy::time::TimerMode;
use std::time::Duration;

pub struct PlayerPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerSet;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (
                    player_idle_state_system,
                    player_walk_state_system,
                    player_attack_state_system,
                )
                    .in_set(PlayerSet),
                player_dead_state_system.before(PlayerSet),
            ),
        );
        // app.add_systems(
        //     PreUpdate,
        //     ((().in_set(PlayerSet),).run_if(in_state(GameState::Main)),),
        // );
        // add things to your app here
    }
}

#[derive(Component)]
pub struct PlayerStateMaschine {
    pub attack_time: f32,
}

#[derive(Component)]
pub struct PlayerIdleState {
    pub new: bool,
}
#[derive(Component)]
pub struct PlayerWalkingState {
    pub new: bool,
}

#[derive(Component)]
pub struct PlayerAttackingState {
    pub new: bool,
}

#[derive(Component)]
pub struct PlayerDeadState {
    pub new: bool,
}

#[derive(Component)]
pub struct AttackNow {}

#[derive(Component)]
pub struct WalkAnim {
    pub active: bool,
}

fn player_idle_state_system(
    mut commands: Commands,
    mut active_state_query: Query<(&mut PlayerIdleState, Entity)>,
    hitting_query: Query<&AttackNow>,
    walking_query: Query<&WalkAnim>,
    dead_query: Query<&Dead>,
) {
    for (mut state, entity) in active_state_query.iter_mut() {
        if (state.new) {
            player_idle_on_enter(&mut commands, entity);
            state.new = false;
        }
        if let Ok(hitting) = hitting_query.get(entity) {
            commands
                .entity(entity)
                .insert(PlayerAttackingState { new: true });
            player_idle_on_exit(&mut commands, entity);
            continue;
        }

        if let Ok(walking) = walking_query.get(entity) {
            if (walking.active) {
                commands
                    .entity(entity)
                    .insert(PlayerWalkingState { new: true });
                player_idle_on_exit(&mut commands, entity);
                continue;
            }
        }

        if let Ok(dead) = dead_query.get(entity) {
            commands
                .entity(entity)
                .insert(PlayerDeadState { new: true });
            player_idle_on_exit(&mut commands, entity);
            continue;
        }
    }
}

fn player_idle_on_enter(mut commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert(IdleAnimationRunning { new: true });
}
fn player_idle_on_exit(mut commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<PlayerIdleState>();
    commands.entity(entity).remove::<IdleAnimationRunning>();
}

#[derive(Component)]
struct AttackTimer {
    timer: Timer,
}

fn player_attack_state_system(
    mut commands: Commands,
    mut active_state_query: Query<(&mut PlayerAttackingState, Entity)>,
    dead_query: Query<&Dead>,
    state_machine_query: Query<&PlayerStateMaschine>,
    mut attack_timer_query: Query<&mut AttackTimer>,
    time: Res<Time>,
) {
    for (mut state, entity) in active_state_query.iter_mut() {
        if (state.new) {
            if let Ok(state_machine) = state_machine_query.get(entity) {
                player_attacking_on_enter(&mut commands, entity, state_machine.attack_time);
                state.new = false;
            }
        }
        if let Ok(mut attack_timer) = attack_timer_query.get_mut(entity) {
            attack_timer.timer.tick(time.delta());
            if (attack_timer.timer.finished()) {
                commands
                    .entity(entity)
                    .insert(PlayerIdleState { new: true });
                player_attacking_on_exit(&mut commands, entity);
                continue;
            }
        }

        if let Ok(dead) = dead_query.get(entity) {
            commands
                .entity(entity)
                .insert(PlayerDeadState { new: true });
            player_attacking_on_exit(&mut commands, entity);
            continue;
        }
    }
}

fn player_attacking_on_enter(mut commands: &mut Commands, entity: Entity, attack_time: f32) {
    commands
        .entity(entity)
        .insert(HitAnimationRunning { new: true });

    commands.entity(entity).insert(AttackTimer {
        timer: Timer::new(Duration::from_secs_f32(attack_time), TimerMode::Once),
    });

    commands.entity(entity).remove::<AttackNow>();
}
fn player_attacking_on_exit(mut commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<PlayerAttackingState>();
    commands.entity(entity).remove::<HitAnimationRunning>();
    commands.entity(entity).remove::<AttackTimer>();
}

fn player_walk_state_system(
    mut commands: Commands,
    mut active_state_query: Query<(&mut PlayerWalkingState, Entity)>,
    hitting_query: Query<&AttackNow>,
    walking_query: Query<&WalkAnim>,
    dead_query: Query<&Dead>,
) {
    for (mut state, entity) in active_state_query.iter_mut() {
        if (state.new) {
            player_walking_on_enter(&mut commands, entity);
            state.new = false;
        }
        if let Ok(hitting) = hitting_query.get(entity) {
            commands
                .entity(entity)
                .insert(PlayerAttackingState { new: true });
            player_walking_on_exit(&mut commands, entity);
            continue;
        }

        if let Ok(walking) = walking_query.get(entity) {
            if (!walking.active) {
                commands
                    .entity(entity)
                    .insert(PlayerIdleState { new: true });
                player_walking_on_exit(&mut commands, entity);
                continue;
            }
        }

        if let Ok(dead) = dead_query.get(entity) {
            commands
                .entity(entity)
                .insert(PlayerDeadState { new: true });
            player_walking_on_exit(&mut commands, entity);
            continue;
        }
    }
}

fn player_walking_on_enter(mut commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert(WalkingAnimationRunning { new: true });
}
fn player_walking_on_exit(mut commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<PlayerWalkingState>();
    commands.entity(entity).remove::<WalkingAnimationRunning>();
}

fn player_dead_state_system(
    mut commands: Commands,
    mut active_state_query: Query<(&mut PlayerDeadState, Entity)>,
) {
    for (mut state, entity) in active_state_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}
