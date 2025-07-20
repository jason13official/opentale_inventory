use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};
use crate::core::plugin::*;

pub fn create_app() -> App {

    println!("Constructing our game app.");

    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenTale Inventory Demo".into(),
                resolution: (1280.0, 720.0).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        })
    )
    .add_plugins(InventoryPlugin)

    .add_systems(Update, exit_handler)
    .add_systems(Update, toggle_fullscreen);

    println!("Returning constructed game app.");

    app
}

fn exit_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        println!("Exiting game app via ESC keypress.");
        exit.send(AppExit);
    }
}

fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::F11) {

        println!("Toggling fullscreen 1920x1080 or windowed 1280x720 resolution via F11 keypress.");

        let mut window = windows.single_mut();
        if window.mode == WindowMode::Windowed {
            window.mode = WindowMode::BorderlessFullscreen;
            window.resolution.set(1920.0, 1080.0);
        }
        else {
            window.mode = WindowMode::Windowed;
            window.resolution.set(1280.0, 720.0);
        }
    }
}