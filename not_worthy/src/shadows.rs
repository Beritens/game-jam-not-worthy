use crate::game_state::GameState;
use crate::spawning::{spawn_enemy_system, spawn_fast_enemy_system};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{
    in_state, Commands, Component, Entity, GlobalTransform, IntoSystemConfigs, Query, Transform,
    Without,
};

pub struct ShadowPlugin;

impl Plugin for ShadowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (cast_shadow_system.run_if(in_state(GameState::InGame)),),
        );
    }
}

#[derive(Component)]
pub struct Shadow {
    pub caster: Entity,
}
fn cast_shadow_system(
    mut commands: Commands,
    transform_query: Query<&Transform, Without<Shadow>>,
    mut shadow_caster_query: Query<(&mut Transform, &Shadow, Entity)>,
) {
    for (mut transform, shadow, entity) in shadow_caster_query.iter_mut() {
        if let Ok(caster) = transform_query.get(shadow.caster) {
            transform.translation.x = caster.translation.x;
        } else {
            commands.entity(entity).despawn();
        }
    }
}
