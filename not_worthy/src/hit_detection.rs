use crate::game_state::GameState;
use crate::level_loading::SceneObject;
use crate::summoning::Player;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{
    in_state, Commands, Component, IntoSystemConfigs, OnEnter, Query, SystemSet, Transform, With,
};

const LEFT_MOST: f32 = -10.0;
const RIGHT_MOST: f32 = 10.0;
const RESOLUTION: usize = 100;

pub struct HitDetectionPlugin;

impl Plugin for HitDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            setup_hit_detection.run_if(in_state(GameState::InGame)),
        );
        app.add_systems(
            Update,
            ((build_detection_vector_system).in_set(crate::combat::CombatSet)),
        );
        // app.add_systems(Update, (player_hit));
    }
}

#[derive(Component)]
pub struct HitDetection {
    pub space: [usize; RESOLUTION],
}
pub fn get_space_coord(pos: f32) -> usize {
    return (((pos - LEFT_MOST) * ((RESOLUTION as f32) / (RIGHT_MOST - LEFT_MOST))) as usize)
        .clamp(0, RESOLUTION - 1);
}
fn fill_space(space: &mut [usize; RESOLUTION], pos: f32) {
    let coord: usize = get_space_coord(pos);

    space[coord] += 1;
    let left = coord.saturating_sub(1);
    let right = coord.saturating_add(1).min(RESOLUTION - 1);

    space[left] += (left != coord) as usize;
    space[right] += (right != coord) as usize;
}

pub fn test_point(space: &[usize; 100], pos: f32) -> bool {
    if (pos < LEFT_MOST || pos > RIGHT_MOST) {
        return false;
    }
    let coord = get_space_coord(pos);
    if (space[coord] > 0) {
        return true;
    }
    return false;
}

fn setup_hit_detection(mut commands: Commands) {
    commands.spawn((
        HitDetection {
            space: [0; RESOLUTION],
        },
        SceneObject,
    ));
}
fn build_detection_vector_system(
    mut hit_detection_query: Query<&mut HitDetection>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(mut hit_detection) = hit_detection_query.get_single_mut() {
        for elem in hit_detection.space.iter_mut() {
            *elem = 0;
        }
        for (transform) in player_query.iter() {
            fill_space(&mut hit_detection.space, transform.translation.x);
        }
    }
}
