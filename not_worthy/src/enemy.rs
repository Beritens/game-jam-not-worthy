use crate::asset_load::EnemySprite;
use crate::combat::{CombatSet, Dead, Stunned};
use crate::game_state::GameState;
use crate::summoning::spawn_deceased;
use avian2d::prelude::LinearVelocity;
use bevy::app::{App, Plugin, PreUpdate, Update};
use bevy::prelude::{
    in_state, Commands, Component, Entity, IntoSystemConfigs, Query, Res, Time, TimerMode,
    Transform, Vec2, With,
};
use bevy::time::Timer;
use bevy_sprite3d::Sprite3dParams;
use std::time::Duration;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (walk_to_target.before(CombatSet)));

        app.add_systems(
            PreUpdate,
            (
                basic_enem_active_state_system,
                basic_enem_stunned_state_system,
                basic_enem_dead_state_system.run_if(in_state(GameState::Main)),
            ),
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
    mut query: Query<(&mut LinearVelocity, &Target, &Transform, &Walker), With<Walking>>,
) {
    for (mut linear_vel, target, transform, walker) in query.iter_mut() {
        linear_vel.x = (target.pos.x - transform.translation.x).signum() * walker.speed;
    }
}

#[derive(Component)]
pub struct BasicEnemStateMachine {
    pub stunne_time: f32,
}

#[derive(Component)]
pub struct BacicEnemActiveState {
    pub new: bool,
}

fn basic_enem_active_state_system(
    mut commands: Commands,
    mut active_state_query: Query<(&mut BacicEnemActiveState, Entity)>,
    stunned_query: Query<(&Stunned, Entity), With<BacicEnemActiveState>>,
) {
    for (mut state, entity) in active_state_query.iter_mut() {
        if (state.new) {
            basic_enem_active_on_enter(&mut commands, entity);
            state.new = false;
        }
    }
    for (stunned, entity) in stunned_query.iter() {
        basic_enem_active_on_exit(&mut commands, entity);
        commands
            .entity(entity)
            .insert(BacisEnemStunnedState { new: true });
    }
}

fn basic_enem_active_on_enter(mut commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(Walking {});
}
fn basic_enem_active_on_exit(mut commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<BacicEnemActiveState>();
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
    stunned_query: Query<(&Stunned)>,
    state_machine_query: Query<(&BasicEnemStateMachine)>,
) {
    for (mut state, entity) in stunned_state_query.iter_mut() {
        if (state.new) {
            if let Ok(state_machine) = state_machine_query.get(entity) {
                basic_enem_stunned_on_enter(&mut commands, entity, state_machine.stunne_time);
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
            }
        }
    }
}

fn basic_enem_stunned_on_enter(mut commands: &mut Commands, entity: Entity, stunned_time: f32) {
    commands.entity(entity).insert(StunnedTimer {
        timer: Timer::new(Duration::from_secs_f32(stunned_time), TimerMode::Once),
    });
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
        commands.entity(entity).despawn();
        spawn_deceased(
            &mut commands,
            transform.translation.x,
            &hero_asset.idle,
            &mut sprite_params,
        );
    }
}
