use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{Camera2dBundle, Commands};
use crate::core::ui::interface::inventory_init;

pub struct InventoryManagementPlugin;

impl Plugin for InventoryManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, inventory_init);
        app.add_systems(Startup, spawn_camera);
    }
}

pub fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}