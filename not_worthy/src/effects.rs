use crate::game_state::GameState;
use crate::level_loading::SceneObject;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::{
    default, in_state, Alpha, Commands, Component, LinearRgba, OnEnter, Transform, Vec3,
};
use bevy_firework::bevy_utilitarian::prelude::{RandF32, RandValue, RandVec3};
use bevy_firework::core::{BlendMode, ParticleSpawner};
use bevy_firework::curve::{FireworkCurve, FireworkGradient};
use bevy_firework::emission_shape::EmissionShape;
use bevy_firework::plugin::ParticleSystemPlugin;
use std::f32::consts::PI;

pub struct EffectPlugin;

pub const THE_GREEN: LinearRgba = LinearRgba::new(0.16, 0.74, 0.26, 1.0);

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_particles_system);
        // app.add_systems(Update, (player_hit));
    }
}

#[derive(Component)]
pub struct AriseCooldownEffect;

#[derive(Component)]
pub struct AriseEffect;

#[derive(Component)]
pub struct AttackEffect;
fn spawn_particles_system(mut commands: Commands) {
    commands.spawn((
        AriseCooldownEffect,
        SceneObject,
        ParticleSpawner {
            one_shot: false,
            rate: 80.0,
            emission_shape: EmissionShape::Sphere { 0: 0.1 },
            lifetime: RandF32 { min: 0.4, max: 1.2 },
            inherit_parent_velocity: true,
            initial_velocity: RandVec3 {
                magnitude: RandF32 { min: 0., max: 0.6 },
                direction: Vec3::Y,
                spread: PI / 2.0,
            },
            initial_scale: RandF32 {
                min: 0.02,
                max: 0.08,
            },
            scale_curve: FireworkCurve::uneven_samples(vec![(0., 4.0), (1., 0.0)]),
            color: FireworkGradient::uneven_samples(vec![
                (0., THE_GREEN.clone().with_alpha(0.9)),
                (1., THE_GREEN.clone().with_alpha(0.0)),
            ]),
            blend_mode: BlendMode::Add,
            linear_drag: 2.0,
            acceleration: Vec3::Y * 6.,
            pbr: false,
            starts_enabled: false,
            ..default()
        },
        Transform::from_xyz(0., -0.25, -1.1),
    ));
    commands.spawn((
        AriseEffect,
        SceneObject,
        ParticleSpawner {
            one_shot: true,
            rate: 150.0,
            emission_shape: EmissionShape::Sphere { 0: 0.1 },
            lifetime: RandF32 { min: 0.4, max: 0.7 },
            inherit_parent_velocity: true,
            initial_velocity: RandVec3 {
                magnitude: RandF32 { min: 6., max: 12. },
                direction: Vec3::Y,
                spread: PI / 2.0,
            },
            initial_scale: RandF32 {
                min: 0.02,
                max: 0.08,
            },
            scale_curve: FireworkCurve::uneven_samples(vec![(0., 4.0), (1., 0.0)]),
            color: FireworkGradient::uneven_samples(vec![
                (0., THE_GREEN.clone().with_alpha(0.9)),
                (1., THE_GREEN.clone().with_alpha(0.0)),
            ]),
            blend_mode: BlendMode::Add,
            linear_drag: 0.0,
            acceleration: Vec3::Y * 6.,
            pbr: false,
            starts_enabled: false,
            ..default()
        },
        Transform::from_xyz(0., -0.25, -1.1),
    ));
    commands.spawn((
        AttackEffect,
        SceneObject,
        ParticleSpawner {
            one_shot: true,
            rate: 20.0,
            emission_shape: EmissionShape::Circle {
                normal: Vec3::Y,
                radius: 0.1,
            },
            lifetime: RandF32 { min: 0.2, max: 0.3 },
            inherit_parent_velocity: true,
            initial_velocity_radial: RandF32 { min: 2.0, max: 4.0 },
            initial_scale: RandF32 {
                min: 0.02,
                max: 0.08,
            },
            scale_curve: FireworkCurve::uneven_samples(vec![(0., 3.0), (1., 0.0)]),
            color: FireworkGradient::uneven_samples(vec![
                (0., THE_GREEN.clone().with_alpha(0.9)),
                (1., THE_GREEN.clone().with_alpha(0.0)),
            ]),
            blend_mode: BlendMode::Add,
            linear_drag: 0.0,
            acceleration: Vec3::Y * 6.,
            pbr: false,
            starts_enabled: false,
            ..default()
        },
        Transform::from_xyz(0., -0.25, -1.),
    ));
}
