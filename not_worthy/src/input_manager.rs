use bevy::app::{App, Plugin};
use bevy::prelude::{Commands, Component, GamepadButton, KeyCode, Reflect, Startup};
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::GamepadStick;

pub struct InputManagingPlugin;

impl Plugin for InputManagingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
        app.add_systems(Startup, spawn_input_manager);
        // add things to your app here
    }
}
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    #[actionlike(Axis)]
    Move,
    #[actionlike(Button)]
    Punch,
    #[actionlike(Button)]
    Special,
}

#[derive(Component)]
pub struct BasicControl {}
fn spawn_input_manager(mut commands: Commands) {
    let mut input_map = InputMap::default()
        .with_axis(Action::Move, GamepadControlAxis::LEFT_X)
        .with_axis(Action::Move, VirtualAxis::ad())
        .with(Action::Punch, GamepadButton::South)
        .with(Action::Punch, KeyCode::KeyJ)
        .with(Action::Special, GamepadButton::East)
        .with(Action::Special, KeyCode::Space);
    // input_map.insert_axis(Action::Move, VirtualAxis::ad());
    commands.spawn((InputManagerBundle::with_map(input_map), BasicControl {}));
}
