use crate::asset_load::{EnemySprite, SkeletonSprite};
use crate::combat::{Dead, Direction, Health, Hitter, Opfer};
use crate::enemy::{BacicEnemActiveState, BasicEnemStateMachine, Target, Walker};
use crate::game_state::GameState;
use crate::input_manager::{Action, BasicControl};
use crate::movement::{
    get_enemy_collision_layers, get_player_collision_layers, Controllable, GameLayer,
};
use avian2d::collision::Collider;
use avian2d::prelude::{
    LayerMask, LockedAxes, MassPropertiesBundle, RigidBody, SpatialQueryFilter,
};
use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::image::Image;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{
    default, in_state, AlphaMode, BuildChildren, ChildBuild, Circle, Commands, Component,
    DespawnRecursiveExt, Entity, IntoSystemConfigs, PreUpdate, Query, Res, Transform, With,
};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use leafwing_input_manager::action_state::ActionState;
use std::collections::VecDeque;
use std::f32::consts::PI;

pub struct SummoningPlugin;

impl Plugin for SummoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, arise_system.run_if(in_state(GameState::Main)));
        app.add_systems(PreUpdate, die_system.run_if(in_state(GameState::Main)));
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
            Direction { direction: -1.0 },
            Opfer {
                hit_layer: 1,
                hits: VecDeque::new(),
            },
            Health::from_health(1.0),
            Hitter {
                hit_box: Vec2::new(1.0, 1.0),
                offset: Vec2::new(1.0, 0.0),
                hit_mask: 1,
                spatial_query_filter: SpatialQueryFilter::from_mask(LayerMask::from(
                    GameLayer::Enemy,
                )),
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
pub fn die_system(mut commands: Commands, query: Query<Entity, (With<Dead>, With<Controllable>)>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
