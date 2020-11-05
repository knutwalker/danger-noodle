#![warn(clippy::complexity)]

use bevy::prelude::*;

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::hex("F00B42").unwrap().into()),
    });
}

fn game_setup(mut commands: Commands, materials: Res<Materials>) {
    commands
        .spawn(SpriteComponents {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(DangerNoodleHead);
}

struct DangerNoodleHead;
struct Materials {
    head_material: Handle<ColorMaterial>,
}
