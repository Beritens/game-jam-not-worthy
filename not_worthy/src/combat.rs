use crate::asset_load::{EnemySounds, PlayerSounds};
use crate::game_state::GameState;
use crate::input_manager::{Action, BasicControl};
use crate::level_loading::SceneObject;
use crate::movement::{Controllable, GameLayer};
use crate::player_states::AttackNow;
use avian2d::prelude::{Collider, LayerMask, LinearVelocity, SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::audio::{AudioPlayer, PlaybackMode, PlaybackSettings};
use bevy::math::{Quat, Vec2};
use bevy::prelude::{
    in_state, info, Commands, Component, Entity, Gamepad, GamepadAxis, GamepadButton,
    IntoSystemConfigs, Query, Res, Startup, SystemSet, Time, Timer, Transform, Vec3Swizzles, With,
};
use bevy::time::TimerMode;
use leafwing_input_manager::prelude::ActionState;
use std::collections::VecDeque;
use std::time::Duration;

pub struct CombatPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CombatSet;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player_attacks);
        app.add_systems(
            Update,
            ((
                player_hit.run_if(in_state(GameState::InGame)),
                hit_system
                    .after(player_hit)
                    .run_if(in_state(GameState::InGame)),
                enemy_take_damage.after(hit_system),
            )
                .in_set(CombatSet)),
        );
        // app.add_systems(Update, (player_hit));
    }
}

fn setup_player_attacks(mut commands: Commands) {
    commands.spawn((
        Cooldown {
            timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once),
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

#[derive(Component)]
pub struct Dead {}

#[derive(Component)]
pub struct Opfer {
    pub hit_layer: u32,
    pub hits: VecDeque<Attack>,
}

#[derive(Component)]
pub struct Cooldown {
    pub timer: Timer,
}
#[derive(Component)]
pub struct PlayerHit {}

fn player_hit(
    time: Res<Time>,
    mut commands: Commands,
    input_query: Query<(&ActionState<Action>), With<BasicControl>>,
    mut cooldown_query: Query<(&mut Cooldown), With<PlayerHit>>,
    mut query: Query<(&Transform, &mut Direction, Entity), (With<Hitter>, With<Controllable>)>,
    sound_asset: Res<PlayerSounds>,
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

    for mut cooldown in cooldown_query.iter_mut() {
        cooldown.timer.tick(time.delta());
        if (cooldown.timer.finished()) {
            if (attack) {
                cooldown.timer.reset();
            }
        } else {
            attack = false;
        }
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
        // let opfer = opfer_query.get(*entity);
        if let Ok(opfer) = opfer_query.get(*entity) {
            if ((1 << opfer.hit_layer & hitter.hit_mask) != 0) {
                count += 1;
            }
        }
    }
    for entity in intersections.iter() {
        if let Ok(mut opfer) = opfer_query.get_mut(*entity) {
            if ((1 << opfer.hit_layer & hitter.hit_mask) != 0) {
                opfer.hits.push_back(Attack {
                    damage: hitter.damage / count as f32,
                    knockback: hitter.knockback / count as f32,
                });
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
                transform.translation.x.signum() * element.knockback,
                element.knockback,
            );
            if stunned_query.get(entity).is_err() {
                commands.entity(entity).insert(Stunned {});
            }
            health.health -= element.damage;
            if (health.health <= 0.0) {
                linear_velocity.0 = Vec2::ZERO;
                commands.entity(entity).insert(Dead {});
            }
        }
    }
}
