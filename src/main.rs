mod world;
mod core;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};
use world::item::{items::*};
use crate::world::inventory::components::{HeldItem};
use crate::world::inventory::containers::{CloseChestEvent, ContainerManager, OpenChestEvent, SwitchContainerEvent};
use crate::world::inventory::inventory::{SlotContainer};
use crate::world::inventory::systems::*;
use crate::world::inventory::ui::*;

fn main() {

    for (id, item) in ITEMS {
        println!("{} -> {:?}", id, item.properties);
    }

    println!("DIAMOND durability: {:?}", DIAMOND.properties.durability);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenTale Inventory Demo".into(),
                // resolution: (1920.0, 1080.0).into(),
                resolution: (1280.0, 720.0).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))

        .insert_resource(HeldItem::default())
        .insert_resource(ContainerManager::default())

        .add_event::<SwitchContainerEvent>()
        .add_event::<OpenChestEvent>()
        .add_event::<CloseChestEvent>()

        .add_systems(Startup, setup_game)

        .add_systems(Update, exit_on_esc)
        .add_systems(Update, toggle_fullscreen)
        .add_systems(Update, (
            // Handle input first
            handle_keyboard_input,
            // Then process container events
            handle_container_events,
            // Then handle UI rebuilding
            handle_ui_rebuild,
        ).chain()) // Run these in order
        .add_systems(Update, (
            // These can run in parallel after UI is stable
            handle_left_clicks,
            handle_right_clicks,
            update_slot_visuals,
            update_held_item_display,
        ).after(handle_ui_rebuild)) // Run after UI rebuild is complete

        .run();

    println!("Closed the inventory demo successfully.")
}

fn exit_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::F11) {
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