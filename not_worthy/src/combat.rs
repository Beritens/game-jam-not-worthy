use crate::input_manager::{Action, BasicControl};
use avian2d::prelude::{Collider, LinearVelocity, SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::math::{Quat, Vec2};
use bevy::prelude::{
    info, Commands, Component, Entity, Gamepad, GamepadAxis, GamepadButton, IntoSystemConfigs,
    Query, SystemSet, Transform, Vec3Swizzles, With,
};
use leafwing_input_manager::prelude::ActionState;
use std::collections::VecDeque;

pub struct CombatPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CombatSet;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((player_hit, enemy_take_damage.after(player_hit)).in_set(CombatSet)),
        );
    }
}

#[derive(Component)]
pub struct Hitter {
    pub hit_box: Vec2,
    pub offset: Vec2,
    pub hit_mask: u32,
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

fn player_hit(
    input_query: Query<(&ActionState<Action>), With<BasicControl>>,
    mut query: Query<(&Transform, &mut Hitter, Entity)>,
    mut opfer_query: Query<(&mut Opfer)>,
    spatial_query: SpatialQuery,
) {
    let mut direction = 0.0;
    let mut attack = false;
    for (action) in &input_query {
        if action.just_pressed(&Action::Punch) {
            attack = true;
        }

        let x = action.clamped_value(&Action::Move);
        if x.abs() > 0.01 {
            direction = x.signum();
        }
    }
    if (direction.abs() > 0.01 || attack) {
        for (transform, mut hitter, entity) in query.iter_mut() {
            if (direction.abs() > 0.0) {
                hitter.direction = direction;
            }
            if (attack) {
                hit(
                    &spatial_query,
                    &mut opfer_query,
                    &hitter,
                    transform.translation.xy(),
                );
            }
        }
    }
}

fn hit(
    spatial_query: &SpatialQuery,
    mut opfer_query: &mut Query<&mut Opfer>,
    hitter: &Hitter,
    origin: Vec2,
) {
    let intersections = spatial_query.shape_intersections(
        &Collider::rectangle(hitter.hit_box.x, hitter.hit_box.y),
        origin + hitter.offset * hitter.direction,
        0.0,
        &SpatialQueryFilter::default(),
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
                    damage: 2.0 / count as f32,
                    knockback: 4.0 / count as f32,
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
                commands.entity(entity).insert(Dead {});
            }
        }
    }
}
