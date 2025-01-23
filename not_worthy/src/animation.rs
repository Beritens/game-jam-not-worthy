use avian2d::parry::na::DimRange;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{
    Children, Component, Entity, IntoSystemConfigs, Query, Res, SystemSet, Time, Timer, With,
};
use bevy_sprite3d::Sprite3d;

pub struct SpriteAnimationPlugin;

impl Plugin for SpriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (animate_sprite_system, animation_system));
    }
}

#[derive(Component)]
pub struct AnimationTimer {
    pub start: usize,
    pub end: usize,
    pub timer: Timer,
}
fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite3d)>) {
    for (mut timer, mut sprite_3d) in query.iter_mut() {
        let start = timer.start;
        let end = timer.end;
        next_sprite(&mut timer.timer, &time, start, end, &mut sprite_3d, false);
    }
}

// fn walk_animation_system(
//     time: Res<Time>,
//     mut parent_query: Query<(&Children, &mut WalkingAnimationRunning)>,
//     mut query: Query<(&mut WalkingAnimation, &mut Sprite3d)>,
// ) {
//     for (children, mut anim) in parent_query.iter_mut() {
//         for &child in children.iter() {
//             if let Ok((mut timer, mut sprite_3d)) = query.get_mut(child) {
//                 let start = timer.start;
//                 let end = timer.end;
//                 next_sprite(
//                     &mut timer.timer,
//                     &time,
//                     start,
//                     end,
//                     &mut sprite_3d,
//                     anim.new,
//                 );
//             }
//         }
//         anim.new = false;
//     }
// }
//
// fn hit_animation_system(
//     time: Res<Time>,
//     mut parent_query: Query<(&Children, &mut HitAnimationRunning)>,
//     mut query: Query<(&mut HitAnimation, &mut Sprite3d)>,
// ) {
//     for (children, mut anim) in parent_query.iter_mut() {
//         for &child in children.iter() {
//             if let Ok((mut timer, mut sprite_3d)) = query.get_mut(child) {
//                 let start = timer.start;
//                 let end = timer.end;
//                 next_sprite_no_mod(
//                     &mut timer.timer,
//                     &time,
//                     start,
//                     end,
//                     &mut sprite_3d,
//                     anim.new,
//                 );
//             }
//         }
//         anim.new = false;
//     }
// }
//
// fn idle_animation_system(
//     time: Res<Time>,
//     mut parent_query: Query<(&Children, &mut IdleAnimationRunning)>,
//     mut query: Query<(&mut IdleAnimation, &mut Sprite3d)>,
// ) {
//     for (children, mut anim) in parent_query.iter_mut() {
//         for &child in children.iter() {
//             if let Ok((mut timer, mut sprite_3d)) = query.get_mut(child) {
//                 let start = timer.start;
//                 let end = timer.end;
//                 next_sprite(
//                     &mut timer.timer,
//                     &time,
//                     start,
//                     end,
//                     &mut sprite_3d,
//                     anim.new,
//                 );
//             }
//         }
//         anim.new = false;
//     }
// }
// fn telegraph_animation_system(
//     time: Res<Time>,
//     mut parent_query: Query<(&Children, &mut TelegraphAnimationRunning)>,
//     mut query: Query<(&mut TelegraphAnimation, &mut Sprite3d)>,
// ) {
//     for (children, mut anim) in parent_query.iter_mut() {
//         for &child in children.iter() {
//             if let Ok((mut timer, mut sprite_3d)) = query.get_mut(child) {
//                 let start = timer.start;
//                 let end = timer.end;
//                 next_sprite(
//                     &mut timer.timer,
//                     &time,
//                     start,
//                     end,
//                     &mut sprite_3d,
//                     anim.new,
//                 );
//             }
//         }
//         anim.new = false;
//     }
// }

fn next_sprite(
    timer: &mut Timer,
    time: &Time,
    start: usize,
    end: usize,
    sprite_3d: &mut Sprite3d,
    new: bool,
) {
    if (new) {
        timer.reset();
        let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
        atlas.index = start;
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        let length = end - start + 1;
        let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
        atlas.index = (atlas.index + 1 - start) % (length) + start;
    }
}

fn next_sprite_no_mod(
    timer: &mut Timer,
    time: &Time,
    start: usize,
    end: usize,
    sprite_3d: &mut Sprite3d,
    new: bool,
) -> bool {
    if (new) {
        timer.reset();
        let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
        atlas.index = start;
    }
    timer.tick(time.delta());
    let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
    if (atlas.index >= end) {
        return true;
    }
    if timer.just_finished() {
        let length = end - start + 1;
        atlas.index = (atlas.index + 1 - start) % (length) + start;
    }
    return atlas.index >= end;
}

// #[derive(Component)]
// pub struct WalkingAnimation {
//     pub start: usize,
//     pub end: usize,
//     pub timer: Timer,
// }
//
// #[derive(Component)]
// pub struct WalkingAnimationRunning {
//     pub new: bool,
// }
//
// #[derive(Component)]
// pub struct HitAnimation {
//     pub start: usize,
//     pub end: usize,
//     pub timer: Timer,
// }
//
// #[derive(Component)]
// pub struct HitAnimationRunning {
//     pub new: bool,
// }
//
// #[derive(Component)]
// pub struct IdleAnimation {
//     pub start: usize,
//     pub end: usize,
//     pub timer: Timer,
// }
//
// #[derive(Component)]
// pub struct IdleAnimationRunning {
//     pub new: bool,
// }
//
// #[derive(Component)]
// pub struct TelegraphAnimation {
//     pub start: usize,
//     pub end: usize,
//     pub timer: Timer,
// }
//
// #[derive(Component)]
// pub struct TelegraphAnimationRunning {
//     pub new: bool,
// }

pub struct Animation {
    pub start: usize,
    pub end: usize,
    pub repeating: bool,
    pub timer: Timer,
}
#[derive(Component)]
pub struct AnimationManager {
    pub running: usize,
    pub new: bool,
    pub done: bool,
    pub animations: Vec<Animation>,
}

fn animation_system(
    time: Res<Time>,
    mut parent_query: Query<(&Children, &mut AnimationManager)>,
    mut query: Query<(&mut Sprite3d)>,
) {
    for (children, mut anim) in parent_query.iter_mut() {
        for &child in children.iter() {
            if let Ok(mut sprite_3d) = query.get_mut(child) {
                let start = anim.animations[anim.running].start;
                let end = anim.animations[anim.running].end;
                let running = anim.running;
                let new = anim.new;
                if (anim.new) {
                    anim.done = false;
                }
                match anim.animations[anim.running].repeating {
                    true => {
                        next_sprite(
                            &mut anim.animations[running].timer,
                            &time,
                            start,
                            end,
                            &mut sprite_3d,
                            new,
                        );
                    }
                    false => {
                        anim.done = next_sprite_no_mod(
                            &mut anim.animations[running].timer,
                            &time,
                            start,
                            end,
                            &mut sprite_3d,
                            new,
                        );
                    }
                }
                anim.new = false;
            }
        }
        anim.new = false;
    }
}
