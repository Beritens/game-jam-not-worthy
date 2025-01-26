use bevy::log::info;
use bevy::prelude::ResMut;
use bevy_pkv::PkvStore;

pub fn get_progress(pkv: &mut ResMut<PkvStore>, key: &str) -> i32 {
    if let Ok(progress) = pkv.get::<String>(key.clone()) {
        info!("{key} has level {progress}");
        return progress.parse::<i32>().unwrap();
    } else {
        pkv.set_string(key, "0")
            .expect("failed to store game state");
        info!("new value");
        return 0;
    }
}
pub fn store_progress(mut pkv: &mut ResMut<PkvStore>, key: &str, value: i32) {
    pkv.set(key, &value.to_string())
        .expect("failed to store game state");
    info!("{key} now has level {value}");
}
