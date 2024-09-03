use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

use crate::{
    add_health_bar, build_mesh,
    bullet::{Bullet, BulletType, SpawnBullet},
    camera::MainCamera,
    Friction, Health, Velocity,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<MoveAction>::default(),
            InputManagerPlugin::<FireAction>::default(),
        ))
        .add_systems(Startup, spawn_player)
        .add_systems(Update, (move_player, fire_bullets));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Debug, Actionlike, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
enum MoveAction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Actionlike, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
pub struct FireAction;

fn spawn_player(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let move_input_map = InputMap::new([
        (MoveAction::Left, KeyCode::KeyA),
        (MoveAction::Right, KeyCode::KeyD),
        (MoveAction::Up, KeyCode::KeyW),
        (MoveAction::Down, KeyCode::KeyS),
    ]);

    let fire_input_map = InputMap::new([(FireAction, MouseButton::Left)]);

    let mesh = build_mesh(50., 8);

    let player = commands
        .spawn((
            Player,
            ColorMesh2dBundle {
                mesh: meshes.add(mesh).into(),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                ..Default::default()
            },
            Health::from_max(10.),
            Velocity(Vec3::ZERO),
            Friction(PLAYER_FRICTION),
        ))
        .insert(InputManagerBundle::with_map(move_input_map))
        .insert(InputManagerBundle::with_map(fire_input_map))
        .id();

    add_health_bar(&mut commands, player, 70.);
}

const PLAYER_MAX_SPEED: f32 = 1000.;
const PLAYER_SPEED: f32 = 1200.;
const PLAYER_FRICTION: f32 = 10000.;

fn move_player(mut player: Query<(&mut Velocity, &ActionState<MoveAction>), With<Player>>) {
    let Ok((mut velocity, action)) = player.get_single_mut() else {
        return;
    };

    for action in action.get_pressed() {
        velocity.add_velocity_clamped(
            match action {
                MoveAction::Left => Vec3::ZERO.with_x(-1. * PLAYER_SPEED),
                MoveAction::Right => Vec3::ZERO.with_x(1. * PLAYER_SPEED),
                MoveAction::Up => Vec3::ZERO.with_y(1. * PLAYER_SPEED),
                MoveAction::Down => Vec3::ZERO.with_y(-1. * PLAYER_SPEED),
            },
            PLAYER_MAX_SPEED,
        );
    }

    // if action.get_pressed().is_empty() {
    //     velocity.0 = Vec3::ZERO;
    // }
}

fn fire_bullets(
    mut commands: Commands,
    mut player: Query<(&Transform, &ActionState<FireAction>, &mut Velocity), With<Player>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut writer: EventWriter<SpawnBullet>,
) {
    let Ok((player_transform, action, mut velocity)) = player.get_single_mut() else {
        return;
    };

    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        if action.just_pressed(&FireAction) {
            let mut bullet_velocity =
                Vec3::new(world_position.x, world_position.y, 0.) - player_transform.translation;

            velocity.0 -= bullet_velocity.normalize_or_zero() * 1000.;

            writer.send(SpawnBullet {
                ty: BulletType::Ball,
                position: player_transform.translation,
                direction: bullet_velocity,
            });
        }
    }
}
