use crate::asset_load::{EnemySounds, GameData, GameInfos, PlayerSounds};
use crate::effects::{AriseEffect, AttackEffect};
use crate::game_state::{GameState, PauseState};
use crate::input_manager::{Action, BasicControl};
use crate::level_loading::SceneObject;
use crate::movement::{Controllable, GameLayer};
use crate::player_states::AttackNow;
use crate::state_handling::get_sotred_value;
use avian2d::prelude::{Collider, LayerMask, LinearVelocity, SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::asset::Assets;
use bevy::audio::{AudioPlayer, PlaybackMode, PlaybackSettings};
use bevy::math::{Quat, Vec2};
use bevy::prelude::{
    in_state, info, Commands, Component, Entity, Gamepad, GamepadAxis, GamepadButton,
    IntoSystemConfigs, OnEnter, Query, Res, ResMut, Startup, SystemSet, Time, Timer, Transform,
    Vec3Swizzles, With,
};
use bevy::time::TimerMode;
use bevy_firework::core::ParticleSpawnerData;
use bevy_pkv::PkvStore;
use leafwing_input_manager::prelude::ActionState;
use std::collections::VecDeque;
use std::time::Duration;

pub struct CombatPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CombatSet;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            setup_player_attacks.run_if(in_state(GameState::InGame)),
        );
        app.add_systems(
            Update,
            ((
                player_hit.run_if(in_state(GameState::InGame)),
                hit_system
                    .after(player_hit)
                    .run_if(in_state(GameState::InGame)),
                enemy_take_damage.after(hit_system),
            )
                .in_set(CombatSet))
            .run_if(in_state(PauseState::Running)),
        );
        // app.add_systems(Update, (player_hit));
    }
}

fn setup_player_attacks(
    mut commands: Commands,
    game_data: Res<GameData>,
    mut game_datas: ResMut<Assets<GameInfos>>,
    mut pkv: ResMut<PkvStore>,
) {
    let attack_cooldown_level = get_sotred_value(&mut pkv, "attack_cooldown");

    let mut attack_cooldown = 0.0;
    if let Some(game_data) = game_datas.get(game_data.data.id()) {
        attack_cooldown = game_data.attack_cooldown[attack_cooldown_level as usize];
    }
    commands.spawn((
        SceneObject,
        PlayerCombatSettings {
            cooldown: Timer::new(Duration::from_secs_f32(attack_cooldown), TimerMode::Once),
        },
        PlayerHit {},
    ));
}

#[derive(Component)]
pub struct Hitter {
    pub knockback: f32,
    pub damage: f32,
    pub hit_box: Vec2,
    pub offset: Vec2,
    pub hit_mask: u32,
    pub spatial_query_filter: SpatialQueryFilter,
    pub single: bool,
}
#[derive(Component)]
pub struct Direction {
    pub direction: f32,
}

#[derive(Component)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

impl Health {
    pub const fn from_health(health: f32) -> Self {
        return Health {
            health: health,
            max_health: health,
        };
    }
}

pub struct Attack {
    damage: f32,
    knockback: f32,
}

#[derive(Component)]
pub struct Stunned {}

pub enum Cause {
    Out,
    Attack,
    Default,
}
#[derive(Component, Default)]
pub struct Dead {
    pub reason: Cause,
}
impl Default for Cause {
    fn default() -> Self {
        Cause::Default
    }
}

#[derive(Component)]
pub struct Opfer {
    pub hit_layer: u32,
    pub hits: VecDeque<Attack>,
    pub knockback_multiplier: f32,
}

#[derive(Component)]
pub struct PlayerCombatSettings {
    pub cooldown: Timer,
}

#[derive(Component)]
pub struct PlayerHit {}

fn player_hit(
    time: Res<Time>,
    mut commands: Commands,
    input_query: Query<(&ActionState<Action>), With<BasicControl>>,
    mut player_setup_query: Query<(&mut PlayerCombatSettings), With<PlayerHit>>,
    mut query: Query<(&Transform, &mut Direction, Entity), (With<Hitter>, With<Controllable>)>,
    sound_asset: Res<PlayerSounds>,
    mut arise_effect_query: Query<(&mut ParticleSpawnerData), With<AttackEffect>>,
) {
    let mut dirr = 0.0;
    let mut attack = false;
    for (action) in &input_query {
        if action.just_pressed(&Action::Punch) {
            attack = true;
        }

        let x = action.clamped_value(&Action::Move);
        if x.abs() > 0.5 {
            dirr = x.signum();
        }
    }

    let Ok((mut player_combat_settings)) = player_setup_query.get_single_mut() else {
        return;
    };
    player_combat_settings.cooldown.tick(time.delta());
    if (player_combat_settings.cooldown.finished()) {
        if (attack) {
            player_combat_settings.cooldown.reset();
        }
    } else {
        attack = false;
    }
    if (attack) {
        commands.spawn((
            AudioPlayer::new(sound_asset.swoosh.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..Default::default()
            },
            SceneObject,
        ));
    }
    if (dirr.abs() > 0.0 || attack) {
        for (transform, mut direction, entity) in query.iter_mut() {
            if (dirr.abs() > 0.0) {
                direction.direction = dirr;
            }
            if (attack) {
                for (mut effect) in arise_effect_query.iter_mut() {
                    effect.enabled = true;
                }
                commands.entity(entity).insert(Hitting {});
                commands.entity(entity).insert(AttackNow {});
            }
        }
    }
}

pub fn hit_test(
    spatial_query: &SpatialQuery,
    hitter: &Hitter,
    direction: &Direction,
    origin: Vec2,
    opfer_query: &Query<&Opfer>,
    spatial_query_filter: &SpatialQueryFilter,
) -> bool {
    let intersections = spatial_query.shape_intersections(
        &Collider::rectangle(hitter.hit_box.x, hitter.hit_box.y),
        origin + hitter.offset * direction.direction,
        0.0,
        &spatial_query_filter,
    );
    let mut count = 0;
    for entity in intersections.iter() {
        if let Ok(opfer) = opfer_query.get(*entity) {
            if ((1 << opfer.hit_layer & hitter.hit_mask) != 0) {
                count += 1;
            }
        }
    }
    if (count > 0) {
        return true;
    }
    return false;
}
#[derive(Component)]
pub struct Hitting {}

fn hit_system(
    mut commands: Commands,
    query: Query<(&Transform, &Direction, &Hitter, Entity), With<Hitting>>,
    mut opfer_query: Query<(&mut Opfer)>,
    spatial_query: SpatialQuery,
) {
    for (transform, direction, hitter, entity) in query.iter() {
        hit(
            &spatial_query,
            &mut opfer_query,
            &hitter,
            &direction,
            transform.translation.xy(),
            &hitter.spatial_query_filter,
        );

        commands.entity(entity).remove::<Hitting>();
    }
}

fn hit(
    spatial_query: &SpatialQuery,
    mut opfer_query: &mut Query<&mut Opfer>,
    hitter: &Hitter,
    direction: &Direction,
    origin: Vec2,
    spatial_query_filter: &SpatialQueryFilter,
) {
    let intersections = spatial_query.shape_intersections(
        &Collider::rectangle(hitter.hit_box.x, hitter.hit_box.y),
        origin + hitter.offset * direction.direction,
        0.0,
        &spatial_query_filter,
    );
    let mut count = 0;
    for entity in intersections.iter() {
        if (spatial_query_filter.mask.0 & 2 > 0) {
            println!("ouch");
        } // let opfer = opfer_query.get(*entity);
        if let Ok(opfer) = opfer_query.get(*entity) {
            if ((1 << opfer.hit_layer & hitter.hit_mask) != 0) {
                count += 1;
                if (hitter.single) {
                    break;
                }
            }
        }
    }

    if (spatial_query_filter.mask.0 & 2 > 0) {
        println!("getting hit {}", count);
    }
    for entity in intersections.iter() {
        if let Ok(mut opfer) = opfer_query.get_mut(*entity) {
            if ((1 << opfer.hit_layer & hitter.hit_mask) != 0) {
                opfer.hits.push_back(Attack {
                    damage: hitter.damage / count as f32,
                    knockback: hitter.knockback / count as f32,
                });
                if (hitter.single) {
                    break;
                }
            }
        }
    }
}

fn enemy_take_damage(
    mut commands: Commands,
    mut opfer_query: Query<(
        Entity,
        &mut LinearVelocity,
        &mut Opfer,
        &Transform,
        &mut Health,
    )>,
    stunned_query: Query<&Stunned>,
) {
    for (entity, mut linear_velocity, mut opfer, transform, mut health) in opfer_query.iter_mut() {
        while let Some(element) = opfer.hits.pop_front() {
            linear_velocity.0 += Vec2::new(
                transform.translation.x.signum() * element.knockback * opfer.knockback_multiplier,
                element.knockback * opfer.knockback_multiplier.abs(),
            );
            if stunned_query.get(entity).is_err() {
                commands.entity(entity).insert(Stunned {});
            }
            health.health -= element.damage;
            if (health.health <= 0.0) {
                linear_velocity.0 = Vec2::ZERO;
                commands.entity(entity).insert(Dead::default());
            }
        }
    }
}
