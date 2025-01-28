use crate::combat::Direction;
use crate::effects::AriseCooldownEffect;
use crate::enemy::Walking;
use crate::input_manager::{Action, BasicControl};
use crate::player_states::WalkAnim;
use avian2d::collision::CollisionLayers;
use avian2d::math::PI;
use avian2d::prelude::{LinearVelocity, PhysicsLayer};
use bevy::app::{App, Plugin};
use bevy::input::gamepad::{GamepadConnection, GamepadEvent};
use bevy::input::Axis;
use bevy::prelude::{
    debug, info, Children, Commands, Component, Entity, EventReader, Gamepad, GamepadAxis,
    GamepadButton, Quat, Query, Res, Resource, Transform, Update, Vec2, Vec3, With,
};
use bevy_firework::core::{ParticleSpawner, ParticleSpawnerData};
use leafwing_input_manager::clashing_inputs::BasicInputs;
use leafwing_input_manager::prelude::ActionState;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (movement, look_direction_system));
        // add things to your app here
    }
}
#[derive(Component)]
pub struct Controllable {
    pub speed: f32,
}

fn look_direction_system(
    child_query: Query<(&Children, &Direction)>,
    mut transform_query: Query<(&mut Transform)>,
) {
    for (children, direction) in child_query.iter() {
        for &child in children.iter() {
            if let Ok(mut transform) = transform_query.get_mut(child) {
                transform.rotation =
                    Quat::from_rotation_y((PI / 2.0) - (direction.direction * PI / 2.0))
            }
        }
    }
}

fn signum_with_zero_handling(n: f32) -> f32 {
    if n > 0.0 {
        1.0
    } else if n < 0.0 {
        -1.0
    } else {
        0.0
    }
}
fn movement(
    mut commands: Commands,
    input_query: Query<(&ActionState<Action>), With<BasicControl>>,
    mut rb_query: Query<(&mut LinearVelocity, &Controllable)>,
    mut walk_query: Query<&mut WalkAnim, With<Controllable>>,
    mut arise_cooldown_effect_query: Query<(&mut ParticleSpawner), With<AriseCooldownEffect>>,
) {
    let mut x_input = 0.0;
    for (action) in &input_query {
        // let x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
        let x = action.clamped_value(&Action::Move);
        if x.abs() > 0.01 {
            x_input = x;
        }
    }
    for (mut linear_velocity, controllable) in rb_query.iter_mut() {
        linear_velocity.x = x_input * controllable.speed;
    }
    for (mut walk) in walk_query.iter_mut() {
        walk.active = x_input.abs() > 0.01;
    }
    for (mut effect) in arise_cooldown_effect_query.iter_mut() {
        if (signum_with_zero_handling(effect.acceleration.x) != signum_with_zero_handling(x_input))
        {
            effect.acceleration =
                Vec3::Y * 6.0 + signum_with_zero_handling(x_input) * Vec3::X * 2.0;
        }
    }
}
#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Default, // Layer 0 - the default layer that objects are assigned to
    Player, // Layer 1
    Enemy,  // Layer 2
    Ground, // Layer 3
}

// Player collides with enemies and the ground, but not with other players
pub fn get_player_collision_layers() -> CollisionLayers {
    CollisionLayers::new(GameLayer::Player, [GameLayer::Default, GameLayer::Ground])
}
pub fn get_enemy_collision_layers() -> CollisionLayers {
    CollisionLayers::new(GameLayer::Enemy, [GameLayer::Default, GameLayer::Ground])
}
