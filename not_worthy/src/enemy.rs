use avian2d::prelude::LinearVelocity;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Component, Query, Transform, Vec2, With};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, walk_to_target);
        // add things to your app here
    }
}

#[derive(Component)]
pub struct Target {
    pub pos: Vec2,
}

#[derive(Component)]
pub struct Walker {
    pub speed: f32,
}

fn walk_to_target(mut query: Query<(&mut LinearVelocity, &Target, &Transform, &Walker)>) {
    for (mut linear_vel, target, transform, walker) in query.iter_mut() {
        linear_vel.x = (target.pos.x - transform.translation.x).signum() * walker.speed;
    }
}
