use crate::game_state::GameState;
use crate::level_loading::SceneObject;
use crate::spawning::Enemy;
use crate::state_handling::{get_sotred_value, store_value};
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{
    in_state, Commands, Component, Entity, IntoSystemConfigs, NextState, OnEnter, OnExit, Query,
    ResMut, Transform, Update, With,
};
use bevy_pkv::PkvStore;
use std::collections::VecDeque;

pub struct GameManagerPlugin;

impl Plugin for GameManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (check_lose_system, handle_score_system).run_if(in_state(GameState::InGame)),
        );

        app.add_systems(
            OnEnter(GameState::InGame),
            (setup_score_manager).run_if(in_state(GameState::InGame)),
        );

        app.add_systems(OnExit(GameState::InGame), (update_point_system));
        //debug
        // app.add_systems(Update, (asset_placer_sytem));
    }
}

#[derive(Component)]
pub struct Scorer {
    pub incoming: VecDeque<i32>,
    pub current: i32,
}

fn setup_score_manager(mut commands: Commands) {
    commands.spawn((Scorer {
        current: 0,
        incoming: VecDeque::new(),
    },));
}

fn update_point_system(
    scorer_query: Query<(Entity, &Scorer)>,
    mut commands: Commands,
    mut pkv: ResMut<PkvStore>,
) {
    if let Ok((entity, scorer)) = scorer_query.get_single() {
        let points_so_far = get_sotred_value(&mut pkv, "score");
        store_value(&mut pkv, "score", points_so_far + scorer.current);
        commands.entity(entity).despawn();
    }
}

fn handle_score_system(mut scorer_query: Query<&mut Scorer>) {
    if let Ok(mut scorer) = scorer_query.get_single_mut() {
        while let Some(income) = scorer.incoming.pop_front() {
            scorer.current += income;
        }
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
