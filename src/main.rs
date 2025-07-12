mod world;
mod core;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};
use world::item::{items::*};
use crate::world::inventory::components::{DragState, HeldItem};
use crate::world::inventory::containers::*;
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
                resolution: (1280.0, 720.0).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))

        .insert_resource(HeldItem::default())
        .insert_resource(DragState::default())
        .insert_resource(ContainerManager::default())

        .add_event::<OpenInventoryEvent>()
        .add_event::<CloseInventoryEvent>()
        .add_event::<OpenChestEvent>()
        .add_event::<CloseChestEvent>()

        .add_systems(Startup, setup_game)

        .add_systems(Update, exit_handler)
        .add_systems(Update, toggle_fullscreen)

        .add_systems(Update, (
            handle_keyboard_input,
            handle_container_events,
            handle_ui_rebuild,
        ).chain()) // Run these in order

        .add_systems(Update, (
            // Left-click handling
            handle_left_clicks,
            handle_left_mouse_release,
            handle_left_drag_movement,

            // Right-click systems
            handle_right_clicks_updated,
            handle_right_drag_deposit,

            // Visual updates
            update_slot_visuals,
            update_held_item_display,
        ).after(handle_ui_rebuild)) // run after UI is rebuilt

        .run();

    println!("Closed the inventory demo successfully.")
}

fn exit_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    container_manager: Res<ContainerManager>,
) {
    if keys.just_pressed(KeyCode::Escape) && container_manager.ui_mode == UIMode::HotbarOnly {
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