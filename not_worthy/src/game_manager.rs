use crate::game_state::GameState;
use crate::spawning::Enemy;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{
    in_state, IntoSystemConfigs, NextState, OnEnter, OnExit, Query, ResMut, Transform, Update, With,
};

pub struct GameManagerPlugin;

impl Plugin for GameManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (check_lose_system).run_if(in_state(GameState::InGame)),
        );
        //debug
        // app.add_systems(Update, (asset_placer_sytem));
    }
}

fn check_lose_system(
    enemy_query: Query<&Transform, With<Enemy>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (transform) in enemy_query.iter() {
        if (transform.translation.x.abs() <= 0.1) {
            game_state.set(GameState::CutScene);
        }
    }
}
