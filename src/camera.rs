use bevy::prelude::*;
use noisy_bevy::simplex_noise_2d;
use rand::{Rng, SeedableRng};
use std::f32::consts::TAU;

use crate::Player;

#[derive(Debug)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerCamera::default())
            .add_systems(Startup, startup);
        // .egui_resource::<ScreenShake>()
        // .insert_resource(ScreenShake {
        //     intensity: 42.0,
        //     duration: 0.5,
        //     start_time: 0.0,
        // })
        // .add_systems(Schedule::Update, shake_screen)
        // .add_systems(PostUpdate, update_camera);
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((MainCamera, Camera2dBundle::default()));
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ScreenShake {
    intensity: f32,
    duration: f32,
    start_time: f32,
    x_offset: Vec2,
    y_offset: Vec2,
}

impl ScreenShake {
    pub fn new(intensity: f32, duration: f32, start_time: f32) -> Self {
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let mut offset = move || {
            Vec2::new(
                rng.gen_range(-1000f32..1000f32),
                rng.gen_range(-1000f32..1000f32),
            )
        };

        Self {
            intensity,
            duration,
            start_time,
            x_offset: offset(),
            y_offset: offset(),
        }
    }
}

// fn shake_screen(
//     reader: EventReader<KeyInput>,
//     mut camera: ResMut<PlayerCamera>,
//     shake: Res<ScreenShake>,
//     delta: Res<DeltaTime>,
// ) {
//     for event in reader.peak_read() {
//         if matches!(
//             event,
//             KeyInput {
//                 code: KeyCode::I,
//                 state: KeyState::Pressed,
//                 ..
//             }
//         ) {
//             let mut shake = *shake;
//             shake.start_time = delta.wrapping_elapsed_as_seconds();
//             camera.push_screen_shake(shake)
//         }
//     }
// }

fn update_camera(
    mut player_camera: ResMut<PlayerCamera>,
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    time: Res<Time>,
    // mouse_position: Res<MousePosition>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    player_camera.apply_screen_shake(&time);
    player_camera.follow_player(
        player,
        // vel,
        &mut camera, //&mouse_position,
        &time,
    );
}

/// Handles main camera logic.
#[derive(Resource)]
pub struct PlayerCamera {
    screen_shake: Vec<ScreenShake>,
    max_smooth_factor: f32,
    min_smooth_factor: f32,
    max_distance: f32,
    delta: f32,
    lead_factor: f32,
    shake_offset: Vec3,
    follow_point: Vec3,
    // noise: Noise,
}

// #[derive(Debug)]
// pub struct Noise(noise::OpenSimplex);
//
// impl egui_widget::Widget for Noise {
//     fn display(&mut self, ui: &mut egui::Ui) {}
// }

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            screen_shake: Vec::new(),
            max_smooth_factor: 1.0,
            min_smooth_factor: 1.0,
            max_distance: 100.0,
            delta: 10.0,
            lead_factor: 1.0,
            follow_point: Vec3::new(0., 0., 0.),
            shake_offset: Vec3::ZERO,
            // noise: Noise(noise::OpenSimplex::new(1)),
        }
    }
}

impl PlayerCamera {
    pub fn follow_player(
        &mut self,
        // velocity: &DirectionalVelocity,
        player: &Transform,
        camera: &mut Transform,
        // mouse_position: &MousePosition,
        dt: &Time,
    ) {
        // let direction: Vec3 = mouse_position.0.into();
        // let offset = (direction - player.translation).normalize() * 100.;
        let offset = 0.;

        let target = player.translation + offset;

        let distance_to_target = (target - self.follow_point).length();

        let target: Vec3 = target; // Vec3::from(*velocity).normalize() + target;

        if distance_to_target < self.delta {
            self.follow_point = target;
        } else {
            // Calculate a dynamic smooth factor based on the distance
            let smooth_factor = if distance_to_target > self.max_distance {
                self.max_smooth_factor
            } else {
                let t = distance_to_target / self.max_distance;
                self.min_smooth_factor + (self.max_smooth_factor - self.min_smooth_factor) * t
            };

            // Move the camera smoothly towards the target position
            self.follow_point += (target - self.follow_point) * smooth_factor * dt.delta_seconds();
        }

        camera.translation = self.translation();
    }

    pub fn push_screen_shake(&mut self, shake: ScreenShake) {
        self.screen_shake.push(shake);
    }

    pub fn push_screen_shake_with(&mut self, intensity: f32, duration: f32, start_time: f32) {
        self.screen_shake
            .push(ScreenShake::new(intensity, duration, start_time));
    }

    fn apply_screen_shake(&mut self, dt: &Time) {
        for shake in self.screen_shake.iter_mut() {
            let elapsed = dt.elapsed_seconds_wrapped() - shake.start_time;
            if elapsed < shake.duration {
                let remaining = 1.0 - (elapsed / shake.duration);
                let current_intensity = shake.intensity * remaining;

                // Sample the simplex space in a circle
                let radius = 5.;

                let x_noise = simplex_noise_2d(Vec2::new(
                    shake.x_offset.x + (remaining * TAU).cos() * radius,
                    shake.x_offset.y + (remaining * TAU).sin() * radius,
                ));

                let y_noise = simplex_noise_2d(Vec2::new(
                    shake.y_offset.x + (remaining * TAU).cos() * radius,
                    shake.y_offset.y + (remaining * TAU).sin() * radius,
                ));

                self.shake_offset = Vec3::new(
                    x_noise as f32 * current_intensity,
                    y_noise as f32 * current_intensity,
                    0.0, // typically, we don't shake on the z-axis
                );
            } else {
                // Shake finished
                self.shake_offset = Vec3::new(0.0, 0.0, 0.0);
                shake.start_time = 0.0;
            }
        }

        self.screen_shake.retain(|s| s.start_time > 0.0);
    }

    fn translation(&self) -> Vec3 {
        self.follow_point + self.shake_offset
    }
}
