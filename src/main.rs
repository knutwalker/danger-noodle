#![warn(clippy::complexity)]

use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::*;
use std::{
    ops::Neg,
    ops::{Deref, DerefMut},
    time::Duration,
};

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

fn main() {
    App::build()
        .add_resource(ClearColor(Color::hex("BADA55").unwrap()))
        .add_resource(WindowDescriptor {
            title: "Danger! noooodle".to_string(),
            width: 2000,
            height: 2000,
            ..Default::default()
        })
        .add_resource(DangerNoodleMoveTimer(Timer::new(
            Duration::from_millis(450),
            true,
        )))
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_system(danger_noodle_timer.system())
        .add_system(danger_noodle_movement.system())
        .add_system(food_spawner.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::hex("F00B42").unwrap().into()),
        food_material: materials.add(Color::hex("BA6E15").unwrap().into()),
    });
}

fn game_setup(mut commands: Commands, materials: Res<Materials>) {
    commands
        .spawn(SpriteComponents {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(DangerNoodleHead {
            direction: Direction::Up,
        })
        .with(Position { x: 3, y: 3 })
        .with(Size::square(0.8));
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    for (size, mut sprite) in q.iter_mut() {
        let window = windows.get_primary().unwrap();
        sprite.size = Vec2::new(
            size.width as f32 / ARENA_WIDTH as f32 * window.width() as f32,
            size.height as f32 / ARENA_HEIGHT as f32 * window.height() as f32,
        )
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(p: f32, bound_window: f32, bound_game: f32) -> f32 {
        p / bound_game * bound_window - (bound_window / 2.0) + (bound_window / bound_game / 2.0)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        )
    }
}

fn danger_noodle_timer(time: Res<Time>, mut danger_noodle_timer: ResMut<DangerNoodleMoveTimer>) {
    danger_noodle_timer.tick(time.delta_seconds)
}

fn danger_noodle_movement(
    keyboard_input: Res<Input<KeyCode>>,
    danger_noodle_timer: ResMut<DangerNoodleMoveTimer>,
    mut heads: Query<(Entity, &mut DangerNoodleHead)>,
    mut positions: Query<&mut Position>,
) {
    if !danger_noodle_timer.finished {
        return;
    }
    for (head_entity, mut head) in heads.iter_mut() {
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        let current_direction = head.direction;
        let dir = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else {
            current_direction
        };
        if dir != -current_direction && dir != current_direction {
            head.direction = dir;
        }

        match head.direction {
            Direction::Left => head_pos.x -= 1,
            Direction::Right => head_pos.x += 1,
            Direction::Up => head_pos.y += 1,
            Direction::Down => head_pos.y -= 1,
        }
    }
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    timer.tick(time.delta_seconds);
    if timer.finished {
        commands
            .spawn(SpriteComponents {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            })
            .with(Size::square(0.7));
    }
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

struct DangerNoodleHead {
    direction: Direction,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

struct DangerNoodleMoveTimer(Timer);

impl Deref for DangerNoodleMoveTimer {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DangerNoodleMoveTimer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct Food;

struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}

impl Deref for FoodSpawnTimer {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FoodSpawnTimer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
