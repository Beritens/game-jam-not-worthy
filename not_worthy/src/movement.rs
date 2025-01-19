use crate::input_manager::{Action, BasicControl};
use avian2d::collision::CollisionLayers;
use avian2d::prelude::{LinearVelocity, PhysicsLayer};
use bevy::app::{App, Plugin};
use bevy::input::gamepad::{GamepadConnection, GamepadEvent};
use bevy::input::Axis;
use bevy::prelude::{
    debug, info, Commands, Component, Entity, EventReader, Gamepad, GamepadAxis, GamepadButton,
    Query, Res, Resource, Transform, Update, Vec2, With,
};
use leafwing_input_manager::clashing_inputs::BasicInputs;
use leafwing_input_manager::prelude::ActionState;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (movement));
        // add things to your app here
    }
}
#[derive(Component)]
pub struct Controllable {
    pub speed: f32,
}

fn movement(
    input_query: Query<(&ActionState<Action>), With<BasicControl>>,
    mut rb_query: Query<(&mut LinearVelocity, &Controllable)>,
) {
    let mut x_input = 0.0;
    for (action) in &input_query {
        // let x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
        let x = action.clamped_value(&Action::Move);
        println!("x input {}", x);
        if x.abs() > 0.01 {
            x_input = x;
        }
    }
    for (mut linear_velocity, controllable) in rb_query.iter_mut() {
        linear_velocity.x = x_input * controllable.speed;
    }
}
#[derive(PhysicsLayer, Default)]
enum GameLayer {
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
