use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    math::VectorSpace,
    prelude::*,
    reflect::List,
    render::{
        camera::CameraPlugin,
        mesh::{Indices, Mesh, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    sprite::Mesh2dHandle,
    window::{PresentMode, PrimaryWindow},
};
use camera::PlayerCamera;
use iyes_perf_ui::{entries::PerfUiBundle, PerfUiPlugin};
use leafwing_input_manager::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

mod camera;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hypernova".into(),
                resolution: [1920., 1080.].into(),
                present_mode: PresentMode::Immediate,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins((
            PerfUiPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins((
            InputManagerPlugin::<MoveAction>::default(),
            InputManagerPlugin::<FireAction>::default(),
        ))
        .add_plugins(camera::CameraPlugin)
        .add_systems(
            Startup,
            (setup, spawn_player, spawn_enemy, build_bullet_mesh),
        )
        .add_systems(
            Update,
            (
                exit_on_esc,
                move_player,
                fire_bullets,
                update_health_bar,
                bullet_hit_enemy,
                despawn_with_no_health,
            )
                .chain(),
        )
        .add_systems(FixedPostUpdate, (apply_friction, update_velocity))
        .run();
}

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

impl Health {
    pub fn from_max(max: f32) -> Self {
        Self { max, current: max }
    }
}

#[derive(Component)]
struct HealthBar;

#[derive(Debug, Actionlike, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
enum MoveAction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Actionlike, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
struct FireAction;

fn exit_on_esc(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for input in reader.read() {
        if input.state == ButtonState::Pressed && input.key_code == KeyCode::Escape {
            writer.send(AppExit::Success);
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Friction(f32);

impl Velocity {
    pub fn add_velocity_clamped(&mut self, velocity: Vec3, max_len: f32) {
        self.0 += velocity;
        self.0 = self.0.clamp_length(0., max_len);
    }
}

fn apply_friction(mut entities: Query<(&mut Velocity, &Friction)>, time: Res<Time>) {
    for (mut velocity, friction) in entities.iter_mut() {
        let dir = velocity.0.normalize_or_zero();
        let friction =
            (dir * friction.0 * time.delta_seconds()).clamp_length_max(velocity.0.length());
        velocity.0 -= friction;
    }
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn(PerfUiBundle::default());
}

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

const ENEMY_RADIUS: f32 = 40.;

fn spawn_enemy(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = build_mesh(ENEMY_RADIUS, 4);

    let x = rand::thread_rng().gen_range(-960.0..960.0);
    let y = rand::thread_rng().gen_range(-540.0..540.0);

    let enemy = commands
        .spawn((
            Enemy,
            ColorMesh2dBundle {
                mesh: meshes.add(mesh).into(),
                transform: Transform::from_xyz(x, y, 0.),
                ..Default::default()
            },
            Health::from_max(3.),
        ))
        .id();

    add_health_bar(&mut commands, enemy, 70.);
}

fn add_health_bar(commands: &mut Commands, entity: Entity, y: f32) {
    commands.entity(entity).with_children(|parent| {
        parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(100.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, y, 0.0),
            ..default()
        });

        parent.spawn((
            HealthBar,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(75.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(-50.0, y, 1.0),
                ..default()
            },
        ));
    });
}

fn update_health_bar(
    health: Query<(&Health, &Children)>,
    mut health_bars: Query<(&mut Transform, &mut Sprite), With<HealthBar>>,
) {
    for (health, children) in health.iter() {
        for child in children.iter() {
            if let Ok((mut transform, mut sprite)) = health_bars.get_mut(*child) {
                let health_percentage = health.current / health.max;
                sprite.custom_size = Some(Vec2::new(100.0 * health_percentage, 20.0));
                transform.translation.x = -50.0 + (50.0 * health_percentage);
            }
        }
    }
}

fn build_bullet_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(build_mesh(10., 5));
    commands.insert_resource(BulletMesh(mesh.into()));
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

const BULLET_SPEED: f32 = 1000.;

// TODO: Pool bullets to avoid lag caused by spawning Mesh2dBundle.
#[derive(Component)]
struct Bullet;

fn fire_bullets(
    mut commands: Commands,
    mut player: Query<(&Transform, &ActionState<FireAction>, &mut Velocity), With<Player>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    bullet_mesh: Res<BulletMesh>,
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
            bullet_velocity = bullet_velocity.normalize_or_zero() * BULLET_SPEED;

            velocity.0 -= bullet_velocity;

            commands.spawn((
                ColorMesh2dBundle {
                    mesh: bullet_mesh.0.clone(),
                    transform: Transform::from_translation(player_transform.translation)
                        .with_rotation(Quat::from_rotation_z(PI / 2.0)),
                    ..Default::default()
                },
                Velocity(bullet_velocity),
                Bullet,
            ));
        }
    }
}

fn update_velocity(mut entities: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in entities.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

fn bullet_hit_enemy(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    mut enemies: Query<(&Transform, &mut Health), With<Enemy>>,
    mut player_camera: ResMut<PlayerCamera>,
    time: Res<Time>,
) {
    for (enemy, mut health) in enemies.iter_mut() {
        for (bullet, transform) in bullets.iter() {
            let dist = enemy.translation.distance(transform.translation);
            if dist < ENEMY_RADIUS {
                commands.entity(bullet).despawn();
                health.current -= 1.;

                if health.current <= 0.01 {
                    player_camera.push_screen_shake_with(10., 0.2, time.elapsed_seconds());
                }
            }
        }
    }
}

fn despawn_with_no_health(
    mut commands: Commands,
    entities: Query<(Entity, &Health)>,
    meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, health) in entities.iter() {
        if health.current <= 0.01 {
            commands.entity(entity).despawn_recursive();
            spawn_enemy(commands, meshes);
            return;
        }
    }
}

#[derive(Resource)]
struct BulletMesh(Mesh2dHandle);

const PI2: f32 = std::f32::consts::PI * 2.0;

pub fn build_mesh(radius: f32, vertices: usize) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let n_vertices = vertices + 1;
    let n_triangles = vertices as u32;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);
    let mut indices: Vec<u32> = Vec::with_capacity((n_triangles as usize) * 3);
    for i in 0..vertices {
        let angle = (i as f32) / (vertices as f32) * PI2;
        let c = angle.cos();
        let s = angle.sin();
        let x = radius * c;
        let y = radius * s;
        let u = 0.5 * c + 0.5;
        let v = -0.5 * s + 0.5;
        positions.push([x, y, 0.]);
        normals.push([0., 0., 1.]);
        uvs.push([u, v]);
    }
    positions.push([0., 0., 0.]);
    normals.push([0., 0., 1.]);
    uvs.push([0.5, 0.5]);
    for i in 0..n_triangles {
        indices.push(i % n_triangles);
        indices.push((i + 1) % n_triangles);
        indices.push(n_triangles);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
