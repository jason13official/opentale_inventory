use bevy::app::{App, PluginGroup, Update};
use bevy::DefaultPlugins;
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, Query, Res, States, With};
use bevy::utils::default;
use bevy::window::{MonitorSelection, PrimaryWindow, Window, WindowMode, WindowPlugin, WindowPosition};
use crate::core::plugin::InventoryManagementPlugin;
use crate::core::ui::scaling::UIScale;

/// Tracks our UI-facing game state. Hotbar, Inventory, Chest, etc.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UIGameState {
    #[default]
    Playing,
    InventoryOpen,
    AnvilOpen,
}

/// Create our Bevy game app, add our InventoryPlugin for display and management of inventories,
/// such as the player's hotbar, internal inventory, and accessed chests.
pub fn create_app() -> App {

    // create a raw instance of a game app
    let mut app = App::new();

    app
        // add the default/recommended plugins for game apps from Bevy, and customize our initial window
        .add_plugins(DefaultPlugins.set(WindowPlugin {

            // create our game app's window with a title, resolution, and initial position
            primary_window: Some(Window {
                title: "OpenTale Inventory Demo".into(),
                resolution: (1280.0, 720.0).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))

        .init_resource::<UIScale>()

        .init_state::<UIGameState>()

        // This is the main entrypoint for handling inventories
        .add_plugins(InventoryManagementPlugin)
        .add_systems(Update, toggle_fullscreen);

    // return the initialized game app instance
    app
}

fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::F11) {
        let mut window = windows.single_mut();
        if window.mode == WindowMode::Windowed {
            println!("Windowed -> BorderlessFullscreen");
            window.mode = WindowMode::BorderlessFullscreen;
            window.resolution.set(1920.0, 1080.0);
        }
        else {
            println!("BorderlessFullscreen -> Windowed");
            window.mode = WindowMode::Windowed;
            window.resolution.set(1280.0, 720.0);
        }
    }
}