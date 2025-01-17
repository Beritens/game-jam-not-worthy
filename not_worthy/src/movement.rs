use avian2d::prelude::LinearVelocity;
use bevy::app::{App, Plugin};
use bevy::input::gamepad::{GamepadConnection, GamepadEvent};
use bevy::input::Axis;
use bevy::prelude::{
    debug, info, Commands, Component, Entity, EventReader, Gamepad, GamepadAxis, GamepadButton,
    Query, Res, Resource, Transform, Update, Vec2, With,
};

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
    gamepads: Query<(Entity, &Gamepad)>,
    mut rb_query: Query<(&mut LinearVelocity, &Controllable)>,
) {
    let mut x_input = 0.0;
    for (entity, gamepad) in &gamepads {
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
        if left_stick_x.abs() > 0.01 {
            x_input = left_stick_x;
        }
    }
    for (mut linear_velocity, controllable) in rb_query.iter_mut() {
        linear_velocity.x = x_input * controllable.speed;
    }
}
