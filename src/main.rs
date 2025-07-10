use bevy::prelude::*;

mod inventory;
use inventory::{components::*, systems::*, ui::*};

fn main() {
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
        .insert_resource(Inventory::default())

        .add_systems(Startup, setup_game)

        .add_systems(Update, (
            handle_left_clicks,
            handle_right_clicks,
            update_slot_visuals,
            update_held_item_display,
        ))

        .run();

    println!("Closed the inventory demo successfully..")
}