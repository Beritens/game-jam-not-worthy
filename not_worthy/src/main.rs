mod combat;
mod enemy;
mod movement;

use crate::combat::{CombatPlugin, Hitter};
use crate::enemy::{EnemyPlugin, Target, Walker};
use crate::movement::{Controllable, MovementPlugin};
use avian2d::prelude::{Collider, Gravity, LockedAxes, MassPropertiesBundle, RigidBody};
use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MovementPlugin);
    app.add_plugins(CombatPlugin);
    app.add_plugins(EnemyPlugin);
    app.add_plugins(PhysicsPlugins::default());
    app.add_systems(Startup, setup);
    app.insert_resource(Gravity(Vec2::new(0.0, -9.81)));
    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 2.00, 0.0),
        OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: 40.0,
            },
            ..OrthographicProjection::default_2d()
        },
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        RigidBody::Dynamic,
        Collider::circle(0.5),
        Controllable { speed: 10.0 },
        Hitter {
            hit_box: Vec2::new(1.0, 1.0),
            offset: Vec2::new(1.0, 0.0),
            hit_mask: 0,
            direction: 1.0,
        },
        LockedAxes::ROTATION_LOCKED,
        MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, 1.0)),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(-1.0, 1.0, 0.0)),
        RigidBody::Dynamic,
        Collider::circle(0.5),
        Controllable { speed: 10.0 },
        Hitter {
            hit_box: Vec2::new(1.0, 1.0),
            offset: Vec2::new(1.0, 0.0),
            hit_mask: 0,
            direction: 1.0,
        },
        LockedAxes::ROTATION_LOCKED,
        MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, 1.0)),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(2.0, 1.0, 0.0)),
        RigidBody::Dynamic,
        Collider::circle(0.5),
        Controllable { speed: 10.0 },
        Hitter {
            hit_box: Vec2::new(1.0, 1.0),
            offset: Vec2::new(1.0, 0.0),
            hit_mask: 0,
            direction: 1.0,
        },
        LockedAxes::ROTATION_LOCKED,
        MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, 1.0)),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(6.0, 1.0, 0.0)),
        RigidBody::Dynamic,
        Target {
            pos: Vec2::new(0.0, 0.0),
        },
        Walker { speed: 2.0 },
        Collider::circle(0.5),
        LockedAxes::ROTATION_LOCKED,
        MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        Sprite::from_color(Color::linear_rgb(0.5, 0.1, 0.1), Vec2::new(1.0, 1.0)),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-6.0, 1.0, 0.0)),
        RigidBody::Dynamic,
        Target {
            pos: Vec2::new(0.0, 0.0),
        },
        Walker { speed: 2.0 },
        Collider::circle(0.5),
        LockedAxes::ROTATION_LOCKED,
        MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        Sprite::from_color(Color::linear_rgb(0.5, 0.1, 0.1), Vec2::new(1.0, 1.0)),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, -1.0, 0.0)),
        RigidBody::Static,
        Collider::rectangle(100.0, 1.0),
        Sprite::from_color(Color::WHITE, Vec2::new(100.0, 1.0)),
    ));
}
