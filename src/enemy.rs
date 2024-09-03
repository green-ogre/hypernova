use std::{sync::Arc, time::Duration};

use crate::{add_health_bar, build_mesh, bullet::Bullet, Health, Velocity};
use bevy::{
    prelude::*,
    sprite::{Material2d, Mesh2dHandle},
    utils::HashMap,
};
use rand::Rng;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy);
    }
}

#[derive(Component)]
pub struct Enemy;

pub const ENEMY_RADIUS: f32 = 40.;

pub fn spawn_enemy(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = build_mesh(ENEMY_RADIUS, 4);

    let x = rand::thread_rng().gen_range(-960.0 / 2.0..960.0 / 2.0);
    let y = rand::thread_rng().gen_range(-540.0 / 2.0..540.0 / 2.0);

    let enemy = commands
        .spawn((
            Enemy,
            ColorMesh2dBundle {
                mesh: meshes.add(mesh).into(),
                transform: Transform::from_xyz(x, y, 0.),
                ..Default::default()
            },
            Health::from_max(3.),
            // BulletSpawner::from_seconds(1.0, spawn_in_random_dir),
        ))
        .id();

    add_health_bar(&mut commands, enemy, 70.);
}

// fn spawn_in_random_dir(arena: &mut BulletArena, commands: &mut Commands, position: Vec3) {
//     // let velocity = Vec3::Y * BULLET_SPEED;
//     // Bullet::spawn(commands, BulletType::Ball, velocity, position);
// }

pub fn update_enemy(
    // mut enemies: Query<(&Transform, &mut BulletSpawner), With<Enemy>>,
    time: Res<Time>,
) {
    // for (position, mut spawner) in enemies.iter_mut() {
    //     spawner.timer.tick(time.delta());
    //     if spawner.timer.just_finished() {
    //         (spawner.f)(position.translation);
    //     }
    // }
}
