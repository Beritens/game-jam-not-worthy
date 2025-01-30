use crate::combat::CombatSet;
use crate::enemy::{AttackingHit, FinishedAttack};
use crate::game_state::{GameState, PauseState};
use bevy::app::{App, FixedUpdate, Plugin, PreUpdate, Update};
use bevy::prelude::{
    in_state, Commands, Component, Entity, IntoSystemConfigs, Query, Res, SystemSet, Time, With,
};
use bevy::time::Timer;

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
        // add things to your app here
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
) {
    for (mut state, mut composer, entity) in state_query.iter_mut() {
        if (state.new) {
            composer.timer.reset();
            state.new = false;
        }
        composer.timer.tick(time.delta());

        if (composer.timer.just_finished()) {
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
) {
    for (mut state, entity) in state_query.iter_mut() {
        if (state.new) {
            state.new = false;
            commands.entity(entity).insert(AttackingHit { new: true });
        }

        if let Ok(finished_attack) = attack_finished_query.get(entity) {
            commands.entity(entity).remove::<AttackingHit>();
            commands
                .entity(entity)
                .insert(BossWalkingState { new: true });
            continue;
        }
    }
}
fn boss_dead_state_system() {}
