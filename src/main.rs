use bevy::{input::{keyboard::KeyboardInput, ButtonState}, prelude::*};

fn main() {
    App::default().add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
        title: "hypernova".into(),
        resolution: [1920., 1080.].into(),
        ..Default::default()
        }),
        ..Default::default()
    }))
        .add_systems(Update, exit_on_esc)
        .run();
}

fn exit_on_esc(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for input in reader.read() {
        if matches!(input, KeyboardInput { state, key_code, .. } if *state == ButtonState::Pressed && *key_code == KeyCode::Escape) {
        writer.send(AppExit::Success);
        }
    }
}
