use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Component, IntoSystemConfigs, Query, Res, SystemSet, Time, Timer};
use bevy_sprite3d::Sprite3d;

pub struct SpriteAnimationPlugin;

impl Plugin for SpriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprite);
    }
}

#[derive(Component)]
pub struct AnimationTimer {
    pub timer: Timer,
}
fn animate_sprite(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite3d)>) {
    for (mut timer, mut sprite_3d) in query.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            let length = sprite_3d.texture_atlas_keys.as_ref().unwrap().len();
            let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
            atlas.index = (atlas.index + 1) % length;
        }
    }
}
