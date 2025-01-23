use crate::animation::{Animation, AnimationManager};
use crate::asset_load::{EnemySprite, SkeletonSprite};
use crate::combat::{Direction, Health, Hitter, Opfer};
use crate::enemy::{
    AttackType, BacicEnemActiveState, BasicEnemStateMachine, HitComposer, Target, Walker,
};
use crate::game_state::GameState;
use crate::input_manager::{Action, BasicControl};
use crate::movement::{get_enemy_collision_layers, GameLayer};
use crate::player_states::WalkAnim;
use crate::summoning::{spawn_player, Deceased};
use avian2d::collision::{Collider, LayerMask};
use avian2d::prelude::{LockedAxes, MassPropertiesBundle, RigidBody, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::image::Image;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{
    default, in_state, AlphaMode, BuildChildren, ChildBuild, Circle, Commands, Component, Entity,
    IntoSystemConfigs, Query, Res, TextureAtlas, Time, Transform, Visibility, With,
};
use bevy::time::{Timer, TimerMode};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use leafwing_input_manager::action_state::ActionState;
use rand::Rng;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Duration;

pub struct SpawningPlugin;

impl Plugin for SpawningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            continuous_spawning_system.run_if(in_state(GameState::Main)),
        );
    }
}

#[derive(Component)]
pub struct Spawner {
    pub(crate) timer: Timer,
}

fn continuous_spawning_system(
    time: Res<Time>,
    mut commands: Commands,
    enemy_asset: Res<EnemySprite>,
    mut sprite_params: Sprite3dParams,
    mut spawner_query: Query<(&mut Spawner, &Transform)>,
) {
    for (mut spawner, transform) in spawner_query.iter_mut() {
        spawner.timer.tick(time.delta());
        if (spawner.timer.just_finished()) {
            spawn_enemy(
                &mut commands,
                transform.translation + Vec3::Z * rand::thread_rng().gen_range(-0.3..0.3),
                &enemy_asset,
                &mut sprite_params,
            );

            // spawn_player(
            //     &mut commands,
            //     transform.translation.x,
            //     &enemy_asset,
            //     &mut sprite_params,
            // );
            let duration = spawner.timer.duration().as_secs_f32();
            spawner
                .timer
                .set_duration(Duration::from_secs_f32((duration * 0.9).max(0.1)));
        }
    }
}

pub fn spawn_enemy(
    mut commands: &mut Commands,
    pos: Vec3,
    asset: &EnemySprite,
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
    let hit_composer = HitComposer {
        timer: Timer::new(Duration::from_secs_f32(0.4), TimerMode::Once),
        after_timer: Timer::new(Duration::from_secs_f32(0.1), TimerMode::Once),
        state: 0,
    };
    let parent = commands
        .spawn((
            Transform::from_translation(pos),
            RigidBody::Dynamic,
            get_enemy_collision_layers(),
            Target {
                pos: Vec2::new(0.0, 0.0),
            },
            BasicEnemStateMachine {
                stunne_time: 1.0,
                basic_attack: AttackType::BasicAttack,
            },
            Hitter {
                hit_box: Vec2::new(0.5, 1.0),
                offset: Vec2::new(0.5, 0.0),
                hit_mask: 2,
                spatial_query_filter: SpatialQueryFilter::from_mask(LayerMask::from(
                    GameLayer::Player,
                )),
            },
            Direction { direction: 1.0 },
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
        ))
        .insert((
            hit_composer,
            Visibility::default(),
            AnimationManager {
                running: 3,
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
                        end: 1,
                        repeating: false,
                        timer: Timer::new(Duration::from_secs_f32(0.08), TimerMode::Repeating),
                    },
                    Animation {
                        start: 2,
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
        ))
        .with_children(|parent| {
            parent.spawn((
                sprite.bundle_with_atlas(&mut sprite3d_params, texture_atlas),
                Transform::from_rotation(Quat::from_rotation_y(PI)),
            ));
        });
}
