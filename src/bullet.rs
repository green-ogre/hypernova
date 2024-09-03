use crate::{
    build_mesh,
    camera::PlayerCamera,
    enemy::{Enemy, ENEMY_RADIUS},
    player::Player,
    Health, Velocity,
};
use bevy::{
    math::NormedVectorSpace, prelude::*, render::view::visibility, sprite::Mesh2dHandle,
    window::PrimaryWindow,
};
use std::collections::HashMap;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBullet>()
            .add_event::<DespawnBullet>()
            .add_systems(Startup, init_bullets)
            .add_systems(PreUpdate, spawn_bullets)
            .add_systems(Update, bullet_hit_enemy)
            .add_systems(PostUpdate, (cull_bullets, despawn_bullets));
    }
}

/// If you want to shoot a new bullet, use the `SpawnBullet` event.
///
/// If you want to destroy a bullet, use the `DespawnBullet` event.
#[derive(Component)]
pub struct Bullet;

#[derive(Event)]
pub struct SpawnBullet {
    pub ty: BulletType,
    pub position: Vec3,
    /// Does not have to be normalized.
    pub direction: Vec3,
}

#[derive(Event)]
pub struct DespawnBullet(Entity);

#[derive(Component, Debug, PartialEq, Eq, Hash)]
pub enum BulletType {
    Ball,
}

fn init_bullets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut bullet_map = HashMap::default();

    let ball_meta = BulletMeta {
        mesh: meshes.add(build_mesh(10., 5)).into(),
        speed: 1000.,
    };

    for _ in 0..10 {
        spawn_bullet(
            &mut commands,
            &ball_meta,
            BulletType::Ball,
            Some(InactiveBullet),
        );
    }

    bullet_map.insert(BulletType::Ball, ball_meta);

    commands.insert_resource(BulletMetas(bullet_map));
}

fn spawn_bullet(
    commands: &mut Commands,
    meta: &BulletMeta,
    ty: BulletType,
    inactive: Option<InactiveBullet>,
) {
    let visibility = if inactive.is_none() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let mut e = commands.spawn((
        ColorMesh2dBundle {
            mesh: meta.mesh.clone(),
            visibility,
            ..Default::default()
        },
        Velocity::default(),
        Bullet,
        ty,
    ));

    if inactive.is_some() {
        e.insert(InactiveBullet);
    }
}

#[derive(Component)]
struct InactiveBullet;

#[derive(Resource)]
struct BulletMetas(HashMap<BulletType, BulletMeta>);

struct BulletMeta {
    mesh: Mesh2dHandle,
    speed: f32,
}

fn spawn_bullets(
    mut commands: Commands,
    mut reader: EventReader<SpawnBullet>,
    meta: Res<BulletMetas>,
    mut balls: Query<
        (Entity, &mut Transform, &mut Velocity, &mut Visibility),
        With<InactiveBullet>,
    >,
) {
    for bullet in reader.read() {
        let speed = meta.0.get(&bullet.ty).unwrap().speed;
        let bullet_velocity = bullet.direction.normalize_or_zero() * speed;

        match bullet.ty {
            BulletType::Ball => {
                if let Some((e, mut t, mut v, mut vis)) = balls.iter_mut().next() {
                    t.translation = bullet.position;
                    v.0 = bullet_velocity;
                    *vis = Visibility::Visible;
                    commands.entity(e).remove::<InactiveBullet>().insert(Bullet);
                } else {
                    warn!(
                        "Spawning BulletType[`{:?}`]. Maybe increase initial buffer?",
                        bullet.ty
                    );

                    commands.spawn((
                        ColorMesh2dBundle {
                            mesh: meta.0.get(&BulletType::Ball).unwrap().mesh.clone(),
                            // visibility: Visibility::Hidden,
                            transform: Transform::from_translation(bullet.position),
                            ..Default::default()
                        },
                        // InactiveBullet,
                        // Velocity::default(),
                        Velocity(bullet_velocity),
                        Bullet,
                        BulletType::Ball,
                    ));
                }
            }
        }
    }
}

fn despawn_bullets(
    mut commands: Commands,
    mut bullets: Query<&mut Visibility, With<Bullet>>,
    mut reader: EventReader<DespawnBullet>,
) {
    for DespawnBullet(bullet) in reader.read() {
        if let Ok(mut vis) = bullets.get_mut(*bullet) {
            *vis = Visibility::Hidden;
            commands
                .entity(*bullet)
                .remove::<Bullet>()
                .insert(InactiveBullet);
        }
    }
}

fn cull_bullets(
    mut commands: Commands,
    mut bullets: Query<(Entity, &Transform, &mut Visibility), With<Bullet>>,
    player: Query<&Transform, With<Player>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let player_transform = player.single();
    let window = window.single();

    for (bullet, bullet_transform, mut visibility) in bullets.iter_mut() {
        if player_transform
            .translation
            .distance_squared(bullet_transform.translation)
            > (window.resolution.physical_width() * window.resolution.physical_width()) as f32
        {
            commands.entity(bullet).insert(InactiveBullet);
            *visibility = Visibility::Hidden;
        }
    }
}

fn bullet_hit_enemy(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    mut enemies: Query<(&Transform, &mut Health), With<Enemy>>,
    mut player_camera: ResMut<PlayerCamera>,
    time: Res<Time>,
    mut writer: EventWriter<DespawnBullet>,
) {
    for (enemy, mut health) in enemies.iter_mut() {
        for (bullet, transform) in bullets.iter() {
            let dist = enemy.translation.distance(transform.translation);
            if dist < ENEMY_RADIUS {
                writer.send(DespawnBullet(bullet));
                health.current -= 1.;

                if health.current <= 0.01 {
                    player_camera.push_screen_shake_with(10., 0.2, time.elapsed_seconds());
                }
            }
        }
    }
}
