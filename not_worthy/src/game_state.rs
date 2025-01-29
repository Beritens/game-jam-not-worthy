use bevy::prelude::States;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    CompilingShaders,
    Menu,
    InGame,
    CutScene,
    Shop,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum PauseState {
    #[default]
    Paused,
    Running,
}
