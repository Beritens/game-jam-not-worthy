use avian2d::prelude::{Collider, SpatialQuery, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::math::{Quat, Vec2};
use bevy::prelude::{
    info, Component, Entity, Gamepad, GamepadAxis, GamepadButton, Query, Transform, Vec3Swizzles,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_hit);
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
pub struct Opfer {
    pub hit_layer: u32,
}

fn player_hit(
    gamepads: Query<(Entity, &Gamepad)>,
    mut query: Query<(&Transform, &mut Hitter, Entity)>,
    spatial_query: SpatialQuery,
) {
    let mut direction = 0.0;
    let mut attack = false;
    for (entity, gamepad) in &gamepads {
        if gamepad.just_pressed(GamepadButton::South) {
            attack = true;
        }

        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
        if left_stick_x.abs() > 0.01 {
            direction = left_stick_x.signum();
        }
    }
    if (direction.abs() > 0.01 || attack) {
        for (transform, mut hitter, entity) in query.iter_mut() {
            if (direction.abs() > 0.0) {
                hitter.direction = direction;
            }
            if (attack) {
                hit(&spatial_query, &hitter, transform.translation.xy());
            }
        }
    }
}

fn hit(spatial_query: &SpatialQuery, hitter: &Hitter, origin: Vec2) {
    let intersections = spatial_query.shape_intersections(
        &Collider::rectangle(hitter.hit_box.x, hitter.hit_box.y),
        origin + hitter.offset * hitter.direction,
        0.0,
        &SpatialQueryFilter::default(),
    );
    for entity in intersections.iter() {
        println!("Entity: {}", entity);
    }
}
