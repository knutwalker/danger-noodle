#![warn(clippy::complexity)]

use bevy::prelude::*;

fn main() {
    App::build()
        .add_startup_system(set_title.system())
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_system(danger_noodle_movement.system())
        .add_plugins(DefaultPlugins)
        .run();
}

fn set_title(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_title("Danger! noooodle".into());
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

fn danger_noodle_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut head_positions: Query<(&DangerNoodleHead, &mut Transform)>,
) {
    for (_head, mut transform) in head_positions.iter_mut() {
        if keyboard_input.pressed(KeyCode::Left) {
            *transform.translation.x_mut() -= 2.;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            *transform.translation.x_mut() += 2.;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            *transform.translation.y_mut() -= 2.;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            *transform.translation.y_mut() += 2.;
        }
    }
}
struct DangerNoodleHead;
struct Materials {
    head_material: Handle<ColorMaterial>,
}
