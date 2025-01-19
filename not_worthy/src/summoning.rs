use crate::asset_load::{EnemySprite, SkeletonSprite};
use crate::combat::{Health, Hitter, Opfer};
use crate::enemy::{BacicEnemActiveState, BasicEnemStateMachine, Target, Walker};
use crate::game_state::GameState;
use crate::input_manager::{Action, BasicControl};
use crate::movement::{get_enemy_collision_layers, get_player_collision_layers, Controllable};
use avian2d::collision::Collider;
use avian2d::prelude::{LockedAxes, MassPropertiesBundle, RigidBody};
use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::image::Image;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{
    default, in_state, AlphaMode, BuildChildren, ChildBuild, Circle, Commands, Component, Entity,
    IntoSystemConfigs, Query, Res, Transform, With,
};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use leafwing_input_manager::action_state::ActionState;
use std::collections::VecDeque;
use std::f32::consts::PI;

pub struct SummoningPlugin;

impl Plugin for SummoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, arise_system.run_if(in_state(GameState::Main)));
        // add things to your app here
    }
}

fn arise_system(
    mut commands: Commands,
    input_query: Query<(&ActionState<Action>), With<BasicControl>>,
    deceased_query: Query<(Entity, &Transform), With<Deceased>>,
    skelet_asset: Res<SkeletonSprite>,
    mut sprite_params: Sprite3dParams,
) {
    let mut summon = false;
    for (action) in &input_query {
        if (action.just_pressed(&Action::Special)) {
            summon = true;
        }
        if (summon) {
            for (entity, transform) in deceased_query.iter() {
                spawn_player(
                    &mut commands,
                    transform.translation.x,
                    &skelet_asset.idle,
                    &mut sprite_params,
                );
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn spawn_player(
    mut commands: &mut Commands,
    pos: f32,
    image: &Handle<Image>,
    mut sprite3d_params: &mut Sprite3dParams,
) {
    let sprite = Sprite3dBuilder {
        image: image.clone(),
        pixels_per_metre: 500.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    };
    let parent = commands
        .spawn((
            Transform::from_translation(Vec3::new(pos, 1.0, 0.0)),
            RigidBody::Dynamic,
            Collider::circle(0.5),
            Controllable { speed: 6.0 },
            get_player_collision_layers(),
            Hitter {
                hit_box: Vec2::new(1.0, 1.0),
                offset: Vec2::new(1.0, 0.0),
                hit_mask: 1,
                direction: 1.0,
            },
            LockedAxes::ROTATION_LOCKED,
            MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                sprite.bundle(sprite3d_params),
                Transform::from_rotation(Quat::from_rotation_y(PI)),
            ));
        });
}
pub fn spawn_enemy(
    mut commands: &mut Commands,
    pos: f32,
    image: &Handle<Image>,
    mut sprite3d_params: &mut Sprite3dParams,
) {
    let sprite = Sprite3dBuilder {
        image: image.clone(),
        pixels_per_metre: 500.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    };
    commands.spawn((
        Transform::from_translation(Vec3::new(pos, 1.0, 0.0)),
        RigidBody::Dynamic,
        get_enemy_collision_layers(),
        Target {
            pos: Vec2::new(0.0, 0.0),
        },
        BasicEnemStateMachine { stunne_time: 1.0 },
        BacicEnemActiveState { new: true },
        Walker { speed: 2.0 },
        Health::from_health(4.0),
        Opfer {
            hit_layer: 0,
            hits: VecDeque::new(),
        },
        Collider::circle(0.5),
        LockedAxes::ROTATION_LOCKED,
        MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        sprite.bundle(sprite3d_params),
    ));
}
#[derive(Component)]
pub struct Deceased {}

pub fn spawn_deceased(
    mut commands: &mut Commands,
    pos: f32,
    image: &Handle<Image>,
    mut sprite3d_params: &mut Sprite3dParams,
) {
    let sprite = Sprite3dBuilder {
        image: image.clone(),
        pixels_per_metre: 500.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    };
    commands.spawn((
        Transform::from_translation(Vec3::new(pos, -0.5, 0.0))
            .with_rotation(Quat::from_rotation_z(PI / 2.0)),
        Deceased {},
        sprite.bundle(sprite3d_params),
    ));
}
