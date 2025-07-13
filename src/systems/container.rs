use crate::world::inventory::containers::{CloseChestEvent, CloseInventoryEvent, ContainerManager, ContainerUI, OpenChestEvent, OpenInventoryEvent, SwitchChestEvent};
use crate::world::inventory::ui::create_minecraft_ui;
use bevy::asset::AssetServer;
use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, EventReader, Query, Res, ResMut, With};

#[derive(Component)]
pub struct UIRebuildNeeded;

pub fn handle_container_events(
    mut container_manager: ResMut<ContainerManager>,
    mut open_inventory_events: EventReader<OpenInventoryEvent>,
    mut close_inventory_events: EventReader<CloseInventoryEvent>,
    mut open_chest_events: EventReader<OpenChestEvent>,
    mut close_chest_events: EventReader<CloseChestEvent>,
    mut switch_chest_events: EventReader<SwitchChestEvent>,
    mut commands: Commands,
) {
    let mut needs_rebuild = false;

    for _event in open_inventory_events.read() {
        container_manager.open_inventory();
        needs_rebuild = true;
    }

    for _event in close_inventory_events.read() {
        container_manager.close_inventory();
        needs_rebuild = true;
    }

    for event in open_chest_events.read() {
        container_manager.open_chest(event.chest_id);
        needs_rebuild = true;
    }

    for _event in close_chest_events.read() {
        container_manager.close_chest();
        needs_rebuild = true;
    }

    for event in switch_chest_events.read() {
        container_manager.switch_chest(event.chest_id);
        needs_rebuild = true;
    }

    if needs_rebuild {
        commands.spawn(UIRebuildNeeded);
    }
}

pub fn handle_ui_rebuild(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    container_manager: Res<ContainerManager>,
    ui_query: Query<Entity, With<ContainerUI>>,
    rebuild_query: Query<Entity, With<UIRebuildNeeded>>,
) {
    if !rebuild_query.is_empty() {
        for entity in rebuild_query.iter() {
            commands.entity(entity).despawn();
        }

        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        create_minecraft_ui(&mut commands, &asset_server, &container_manager);
    }
}