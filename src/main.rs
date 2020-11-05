#![warn(clippy::complexity)]

use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::*;
use std::{
    collections::VecDeque,
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
            resizable: false,
            cursor_visible: false,
            ..Default::default()
        })
        .add_resource(DangerNoodleMoveTimer(Timer::new(
            Duration::from_millis(450),
            true,
        )))
        .add_resource(LastTailPosition::default())
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_system(danger_noodle_timer.system())
        .add_system(danger_noodle_moves.system())
        .add_system(food_spawner.system())
        .add_system(danger_noodle_eats.system())
        .add_system(danger_noodle_grows.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_system(game_over.system())
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::hex("F00D13").unwrap().into()),
        segment_material: materials.add(Color::hex("1E66ED").unwrap().into()),
        food_material: materials.add(Color::hex("BA6E15").unwrap().into()),
    });
}

fn game_setup(commands: Commands, materials: Res<Materials>) {
    spawn_new_danger_noodle(commands, &materials);
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

fn danger_noodle_moves(
    keyboard_input: Res<Input<KeyCode>>,
    danger_noodle_timer: ResMut<DangerNoodleMoveTimer>,
    mut last_tail_pos: ResMut<LastTailPosition>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    mut heads: Query<(&mut DangerNoodleHead, &mut Position)>,
    mut segments: Query<(&DangerNoodleSegment, &mut Position)>,
) {
    let dir: Option<Direction> = keyboard_input
        .get_just_pressed()
        .filter_map(|input| match input {
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            _ => None,
        })
        .next();
    for (mut head, mut pos) in heads.iter_mut() {
        let current_direction = head.direction;
        if let Some(dir) = dir {
            head.next_directions.push_back(dir);
        }
        if danger_noodle_timer.finished {
            // update head
            let mut next_seg_pos = *pos;
            let dir = loop {
                if let Some(dir) = head.next_directions.pop_front() {
                    if dir != current_direction && dir != -current_direction {
                        break dir;
                    }
                }
                break head.direction;
            };
            head.direction = dir;
            match dir {
                Direction::Left => pos.x -= 1,
                Direction::Right => pos.x += 1,
                Direction::Up => pos.y += 1,
                Direction::Down => pos.y -= 1,
            }
            if pos.x < 0 {
                pos.x = ARENA_WIDTH as i32 - 1;
            }
            if pos.x as u32 >= ARENA_WIDTH {
                pos.x = 0;
            }
            if pos.y < 0 {
                pos.y = ARENA_HEIGHT as i32 - 1
            }
            if pos.y as u32 >= ARENA_HEIGHT {
                pos.y = 0;
            }

            // update tails
            let head_pos = *pos;
            for (_segment, mut segment_pos) in segments.iter_mut() {
                if next_seg_pos == head_pos {
                    game_over_events.send(GameOverEvent);
                }
                next_seg_pos = std::mem::replace(&mut *segment_pos, next_seg_pos);
            }
            **last_tail_pos = next_seg_pos;

            if pos.x < 0 || pos.y < 0 || pos.x as u32 >= ARENA_WIDTH || pos.y as u32 >= ARENA_HEIGHT
            {
                game_over_events.send(GameOverEvent);
            }
        }
    }
}

fn danger_noodle_eats(
    mut commands: Commands,
    danger_noodle_timer: Res<DangerNoodleMoveTimer>,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    food_positions: Query<With<Food, (Entity, &Position)>>,
    head_positions: Query<With<DangerNoodleHead, &Position>>,
) {
    if !danger_noodle_timer.finished {
        return;
    }
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.despawn(ent);
                growth_events.send(GrowthEvent);
            }
        }
    }
}

fn danger_noodle_grows(
    mut commands: Commands,
    materials: Res<Materials>,
    growth_events: Res<Events<GrowthEvent>>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    last_tail_pos: Res<LastTailPosition>,
) {
    if growth_reader.iter(&growth_events).next().is_some() {
        let last_position = **last_tail_pos;
        spawn_segment(&mut commands, &materials.segment_material, last_position);
    }
}

fn spawn_segment(commands: &mut Commands, material: &Handle<ColorMaterial>, position: Position) {
    commands
        .spawn(SpriteComponents {
            material: material.clone(),
            ..Default::default()
        })
        .with(DangerNoodleSegment)
        .with(position)
        .with(Size::square(0.65));
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
    positions: Query<&Position>,
) {
    timer.tick(time.delta_seconds);
    if timer.finished {
        let mut attempts = 0;
        let pos = loop {
            let pos = Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            };
            if !positions.iter().any(|p| *p == pos) {
                break pos;
            }
            attempts += 1;
            if attempts > 42 {
                return;
            }
        };

        commands
            .spawn(SpriteComponents {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(pos)
            .with(Size::square(0.7));
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: Local<EventReader<GameOverEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    materials: Res<Materials>,
    segments: Query<(Entity, &DangerNoodleSegment)>,
    foods: Query<(Entity, &Food)>,
    heads: Query<(Entity, &DangerNoodleHead)>,
) {
    if reader.iter(&game_over_events).next().is_some() {
        for (ent, _) in segments.iter() {
            commands.despawn(ent);
        }
        for (ent, _) in foods.iter() {
            commands.despawn(ent);
        }
        for (ent, _) in heads.iter() {
            commands.despawn(ent);
        }
        spawn_new_danger_noodle(commands, &materials);
    }
}

fn spawn_new_danger_noodle(mut commands: Commands, materials: &Materials) {
    spawn_segment(
        &mut commands,
        &materials.segment_material,
        Position { x: 3, y: 2 },
    );
    commands
        .spawn(SpriteComponents {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(DangerNoodleHead::default())
        .with(Position { x: 3, y: 3 })
        .with(Size::square(0.8));
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
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

#[derive(Debug, Default)]
struct DangerNoodleHead {
    direction: Direction,
    next_directions: VecDeque<Direction>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Up
    }
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

struct DangerNoodleSegment;

struct Food;

struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(3000), true))
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

#[derive(Debug)]
struct GrowthEvent;

#[derive(Debug, Default)]
struct LastTailPosition(Position);

impl Deref for LastTailPosition {
    type Target = Position;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LastTailPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
struct GameOverEvent;
