use crate::asset_load::{EnemySprite, SkeletonSprite};
use crate::combat::{Direction, Health, Hitter, Opfer};
use crate::enemy::{
    AttackType, BacicEnemActiveState, BasicEnemStateMachine, HitComposer, Target, Walker,
};
use crate::game_state::GameState;
use crate::input_manager::{Action, BasicControl};
use crate::movement::{get_enemy_collision_layers, GameLayer};
use crate::summoning::Deceased;
use avian2d::collision::{Collider, LayerMask};
use avian2d::prelude::{LockedAxes, MassPropertiesBundle, RigidBody, SpatialQueryFilter};
use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::image::Image;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    default, in_state, AlphaMode, Circle, Commands, Component, Entity, IntoSystemConfigs, Query,
    Res, Time, Transform, With,
};
use bevy::time::{Timer, TimerMode};
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dParams};
use leafwing_input_manager::action_state::ActionState;
use std::collections::VecDeque;
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
                transform.translation.x,
                &enemy_asset.idle,
                &mut sprite_params,
            );
            let duration = spawner.timer.duration().as_secs_f32();
            spawner
                .timer
                .set_duration(Duration::from_secs_f32((duration * 0.9).max(0.1)));
        }
    }
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
    let hit_composer = HitComposer {
        timer: Timer::new(Duration::from_secs_f32(0.4), TimerMode::Once),
    };
    commands
        .spawn((
            Transform::from_translation(Vec3::new(pos, 1.0, 0.0)),
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
            sprite.bundle(sprite3d_params),
        ))
        .insert(hit_composer);
}
