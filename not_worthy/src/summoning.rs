use crate::animation::{Animation, AnimationManager};
use crate::asset_load::{EnemySprite, SkeletonSprite};
use crate::combat::{Dead, Direction, Health, Hitter, Opfer};
use crate::enemy::{BacicEnemActiveState, BasicEnemStateMachine, Target, Walker};
use crate::game_state::GameState;
use crate::input_manager::{Action, BasicControl};
use crate::movement::{
    get_enemy_collision_layers, get_player_collision_layers, Controllable, GameLayer,
};
use crate::player_states::{PlayerIdleState, PlayerStateMaschine, WalkAnim};
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
    DespawnRecursiveExt, Entity, IntoSystemConfigs, PreUpdate, Query, Res, TextureAtlasLayout,
    Timer, Transform, Visibility, With,
};
use bevy::sprite::TextureAtlas;
use bevy::time::TimerMode;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use leafwing_input_manager::action_state::ActionState;
use rand::Rng;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Duration;

pub struct SummoningPlugin;

impl Plugin for SummoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, arise_system.run_if(in_state(GameState::Main)));
        // app.add_systems(PreUpdate, die_system.run_if(in_state(GameState::Main)));
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
                    Vec3::new(
                        transform.translation.x,
                        1.0,
                        rand::thread_rng().gen_range(-0.3..0.3),
                    ),
                    &skelet_asset,
                    &mut sprite_params,
                );
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn spawn_player(
    commands: &mut Commands,
    pos: Vec3,
    asset: &SkeletonSprite,
    mut sprite3d_params: &mut Sprite3dParams,
) {
    let sprite = Sprite3dBuilder {
        image: asset.image.clone(),
        pixels_per_metre: 128.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        pivot: Option::from(Vec2::new(0.4, 0.5)),
        ..default()
    };

    let texture_atlas = TextureAtlas {
        layout: asset.layout.clone(),
        index: 0,
    };
    commands
        .spawn((
            PlayerStateMaschine { attack_time: 0.1 },
            PlayerIdleState { new: true },
            WalkAnim { active: false },
            AnimationManager {
                running: 0,
                new: true,
                done: false,
                animations: vec![
                    Animation {
                        start: 0,
                        end: 0,
                        repeating: true,
                        timer: Default::default(),
                    },
                    Animation {
                        start: 1,
                        end: 3,
                        repeating: false,
                        timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                    },
                    Animation {
                        start: 4,
                        end: 10,
                        repeating: true,
                        timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                    },
                ],
            },
            Transform::from_translation(pos),
            RigidBody::Dynamic,
            Collider::circle(0.5),
            Controllable { speed: 3.0 },
            get_player_collision_layers(),
            Direction { direction: -1.0 },
            Opfer {
                hit_layer: 1,
                hits: VecDeque::new(),
            },
            Health::from_health(1.0),
            Hitter {
                knockback: 5.0,
                damage: 2.0,
                hit_box: Vec2::new(1.0, 1.0),
                offset: Vec2::new(0.5, 0.0),
                hit_mask: 1,
                spatial_query_filter: SpatialQueryFilter::from_mask(LayerMask::from(
                    GameLayer::Enemy,
                )),
            },
            LockedAxes::ROTATION_LOCKED,
            MassPropertiesBundle::from_shape(&Circle::new(0.5), 1.0),
        ))
        .insert((Visibility::default(),))
        .with_children(|parent| {
            parent.spawn((
                sprite.bundle_with_atlas(&mut sprite3d_params, texture_atlas),
                Transform::from_rotation(Quat::from_rotation_y(PI)),
            ));
        });
}
#[derive(Component)]
pub struct Deceased {}

pub fn spawn_deceased(
    commands: &mut Commands,
    pos: f32,
    image: &Handle<Image>,
    texture_atlas_layout: &Handle<TextureAtlasLayout>,
    sprite3d_params: &mut Sprite3dParams,
) {
    let texture_atlas = TextureAtlas {
        layout: texture_atlas_layout.clone(),
        index: 0,
    };
    let sprite = Sprite3dBuilder {
        image: image.clone(),
        pixels_per_metre: 128.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        pivot: Option::from(Vec2::new(0.4, 0.5)),
        ..default()
    };
    commands.spawn((
        Transform::from_translation(Vec3::new(pos, -0.5, 0.0))
            .with_rotation(Quat::from_rotation_z(PI / 2.0)),
        Deceased {},
        sprite.bundle_with_atlas(sprite3d_params, texture_atlas.clone()),
    ));
}
// pub fn die_system(mut commands: Commands, query: Query<Entity, (With<Dead>, With<Controllable>)>) {
//     for entity in query.iter() {
//         commands.entity(entity).despawn_recursive();
//     }
// }
