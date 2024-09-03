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
use camera::{MainCamera, PlayerCamera};
use enemy::spawn_enemy;
use iyes_perf_ui::{entries::PerfUiBundle, PerfUiPlugin};
use leafwing_input_manager::prelude::*;
use player::{FireAction, Player};
use rand::Rng;
use std::f32::consts::PI;

mod bullet;
mod camera;
mod enemy;
mod player;

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
            camera::CameraPlugin,
            player::PlayerPlugin,
            bullet::BulletPlugin,
            enemy::EnemyPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (exit_on_esc, update_health_bar, despawn_with_no_health), // .chain(),
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

fn exit_on_esc(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for input in reader.read() {
        if input.state == ButtonState::Pressed && input.key_code == KeyCode::Escape {
            writer.send(AppExit::Success);
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(PerfUiBundle::default());
}

#[derive(Component, Default, Clone, Copy)]
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

pub fn add_health_bar(commands: &mut Commands, entity: Entity, y: f32) {
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

fn update_velocity(mut entities: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in entities.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
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
