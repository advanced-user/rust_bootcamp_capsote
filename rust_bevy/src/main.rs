use bevy::prelude::*;

fn main() {
    App::new().add_plugins(DefaultPlugins).add_systems(Update, hello).run();
}

pub fn hello() {
    println!("Hello");
}