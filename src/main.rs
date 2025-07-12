mod world;
mod core;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};
use world::item::{items::*};
use crate::world::inventory::components::{HeldItem};
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
        .insert_resource(SlotContainer::new(9)) // our initial player hotbar inventory

        .add_systems(Startup, setup_game)

        .add_systems(Update, exit_on_esc)
        .add_systems(Update, toggle_fullscreen)
        .add_systems(Update, (
            handle_left_clicks,
            handle_right_clicks,
            update_slot_visuals,
            update_held_item_display,
        ))

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