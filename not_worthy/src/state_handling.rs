use bevy::log::info;
use bevy::prelude::ResMut;
use bevy_pkv::PkvStore;

pub fn get_sotred_value(pkv: &mut ResMut<PkvStore>, key: &str) -> i32 {
    if let Ok(progress) = pkv.get::<String>(key.clone()) {
        return progress.parse::<i32>().unwrap();
    } else {
        pkv.set_string(key, "0")
            .expect("failed to store game state");
        info!("new value");
        return 0;
    }
}
pub fn store_value(mut pkv: &mut ResMut<PkvStore>, key: &str, value: i32) {
    pkv.set(key, &value.to_string())
        .expect("failed to store game state");
}
