use crate::utils::item_operations::{process_left_click, process_right_click};
use crate::utils::slot_finder::find_slot_under_cursor;
use crate::world::inventory::components::{DragState, HeldItem, InventorySlot, SelectedHotbarSlot};
use crate::world::inventory::containers::{CloseChestEvent, CloseInventoryEvent, ContainerManager, OpenChestEvent, OpenInventoryEvent, SwitchChestEvent};
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::{ButtonInput, ButtonState};
use bevy::prelude::{EventReader, EventWriter, GlobalTransform, KeyCode, MouseButton, Node, Query, Res, ResMut, Window, Interaction, Changed, With};
use crate::world::inventory::ui::ChestButton;

pub fn handle_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut open_inventory_events: EventWriter<OpenInventoryEvent>,
    mut close_inventory_events: EventWriter<CloseInventoryEvent>,
    mut open_chest_events: EventWriter<OpenChestEvent>,
    mut close_chest_events: EventWriter<CloseChestEvent>,
    container_manager: Res<ContainerManager>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        match container_manager.ui_mode {
            crate::world::inventory::containers::UIMode::HotbarOnly => {
                open_inventory_events.send(OpenInventoryEvent);
            }
            crate::world::inventory::containers::UIMode::InventoryOpen => {
                close_inventory_events.send(CloseInventoryEvent);
            }
            crate::world::inventory::containers::UIMode::ChestOpen(_) => {
                close_inventory_events.send(CloseInventoryEvent);
            }
        }
    }

    if keys.just_pressed(KeyCode::KeyC) {
        match container_manager.ui_mode {
            crate::world::inventory::containers::UIMode::ChestOpen(_) => {
                close_chest_events.send(CloseChestEvent);
            }
            _ => {
                open_chest_events.send(OpenChestEvent { chest_id: 1 }); // Default to chest 1
            }
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        match container_manager.ui_mode {
            crate::world::inventory::containers::UIMode::ChestOpen(_) => {
                close_chest_events.send(CloseChestEvent);
            }
            crate::world::inventory::containers::UIMode::InventoryOpen => {
                close_inventory_events.send(CloseInventoryEvent);
            }
            _ => {}
        }
    }
}

pub fn handle_hotbar_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut selected_hotbar_slot: ResMut<SelectedHotbarSlot>,
) {
    // map input keys to slot indexes
    let key_mappings = [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ];

    for (key, slot_index) in key_mappings {
        if keys.just_pressed(key) {
            selected_hotbar_slot.slot_index = slot_index;
            break;
        }
    }

    // mouse wheel scrolling
    for scroll_event in scroll_events.read() {
        if scroll_event.y != 0.0 {
            let current_index = selected_hotbar_slot.slot_index as i32;
            let new_index = if scroll_event.y > 0.0 {
                // decrease index (with wrapping)
                if current_index == 0 { 8 } else { current_index - 1 }
            } else {
                // increase index (with wrapping)
                if current_index == 8 { 0 } else { current_index + 1 }
            };
            selected_hotbar_slot.slot_index = new_index as usize;
        }
    }
}

pub fn handle_left_clicks_updated(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    drag_state: Res<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    for event in mouse_events.read() {
        if event.button == MouseButton::Left && event.state == ButtonState::Released {
            if !drag_state.is_left_dragging && !drag_state.was_left_dragging_this_frame {
                if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
                    if let Some(container) = container_manager.get_container_mut(&slot.container_type) {
                        process_left_click(slot.index, container, &mut held_item);
                    }
                }
            }
        }
    }
}

pub fn handle_right_clicks_updated(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    drag_state: Res<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    for event in mouse_events.read() {
        if event.button == MouseButton::Right && event.state == ButtonState::Released {
            if !drag_state.is_right_dragging && !drag_state.was_right_dragging_this_frame {
                if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
                    if let Some(container) = container_manager.get_container_mut(&slot.container_type) {
                        process_right_click(slot.index, container, &mut held_item);
                    }
                }
            }
        }
    }
}

pub fn handle_chest_button_clicks(
    mut interaction_query: Query<(&Interaction, &ChestButton), (Changed<Interaction>, With<ChestButton>)>,
    mut switch_chest_events: EventWriter<SwitchChestEvent>,
) {
    for (interaction, chest_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            switch_chest_events.send(SwitchChestEvent {
                chest_id: chest_button.chest_id,
            });
        }
    }
}