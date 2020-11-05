#![warn(clippy::complexity)]

use bevy::prelude::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());
}

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_plugins(DefaultPlugins)
        .run();
}
