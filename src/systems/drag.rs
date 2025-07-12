use crate::utils::item_operations::{deposit_single_item, filter_valid_distribution_slots, place_stack_in_slot, process_left_click};
use crate::utils::slot_finder::find_slot_under_cursor;
use crate::world::inventory::components::{DragState, HeldItem, InventorySlot};
use crate::world::inventory::containers::ContainerManager;
use crate::world::inventory::item_stack::ItemStack;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::{EventReader, GlobalTransform, MouseButton, Node, Query, ResMut, Window};

pub fn handle_left_drag_deposit(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    mut drag_state: ResMut<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    
    drag_state.was_left_dragging_this_frame = false;

    for event in mouse_events.read() {
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => {
                    // Start dragging if we have an item
                    if held_item.stack.is_some() {
                        drag_state.is_left_dragging = true;
                        drag_state.left_drag_slots.clear();
                    }
                }
                ButtonState::Released => {
                    // End dragging
                    if drag_state.is_left_dragging {
                        drag_state.was_left_dragging_this_frame = true;
                        process_drag_end(&mut container_manager, &mut held_item, &mut drag_state);
                    }
                    drag_state.is_left_dragging = false;
                    drag_state.left_drag_slots.clear();
                }
            }
        }
    }
    
    if drag_state.is_left_dragging && held_item.stack.is_some() {
        if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
            drag_state.add_left_drag_slot(slot.container_type.clone(), slot.index);
        }
    }
}

pub fn handle_right_drag_deposit(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    mut drag_state: ResMut<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    drag_state.was_right_dragging_this_frame = false;

    for event in mouse_events.read() {
        if event.button == MouseButton::Right {
            match event.state {
                ButtonState::Pressed => {
                    if held_item.stack.is_some() {
                        drag_state.is_right_dragging = true;
                        drag_state.right_last_hovered_slot = None;
                    }
                }
                ButtonState::Released => {
                    if drag_state.is_right_dragging {
                        drag_state.was_right_dragging_this_frame = true;
                    }
                    drag_state.is_right_dragging = false;
                    drag_state.right_last_hovered_slot = None;
                }
            }
        }
    }

    if drag_state.is_right_dragging && held_item.stack.is_some() {
        if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
            let current_slot = (slot.container_type.clone(), slot.index);

            if drag_state.right_last_hovered_slot != Some(current_slot.clone()) {
                drag_state.right_last_hovered_slot = Some(current_slot);

                if let Some(container) = container_manager.get_container_mut(&slot.container_type) {
                    deposit_single_item(slot.index, container, &mut held_item);
                }
            }
        }
    }
}

pub(crate) fn process_drag_end(
    container_manager: &mut ContainerManager,
    held_item: &mut HeldItem,
    drag_state: &mut DragState,
) {
    let Some(_) = &held_item.stack else {
        return;
    };

    // If no slots were dragged over, do nothing (keep holding item)
    if drag_state.left_drag_slots.is_empty() {
        return;
    }

    // Single-deposition
    if drag_state.left_drag_slots.len() == 1 {
        if let Some((container_type, slot_index)) = drag_state.left_drag_slots.first() {
            if let Some(container) = container_manager.get_container_mut(container_type) {
                process_left_click(*slot_index, container, held_item);
            }
        }
    } else {
        // Even-distribution
        distribute_items_evenly(container_manager, held_item, drag_state);
    }
}

fn distribute_items_evenly(
    container_manager: &mut ContainerManager,
    held_item: &mut HeldItem,
    drag_state: &DragState,
) {
    let Some(held_stack) = &held_item.stack else { return; };
    let total_items = held_stack.size;

    // Get all valid slots for distribution
    let mut valid_slots = filter_valid_distribution_slots(
        &drag_state.left_drag_slots,
        held_stack,
        container_manager
    );

    // Only exclude pickup slot if we have OTHER valid slots to distribute to
    if let Some(pickup_slot) = &drag_state.pickup_slot {
        let non_pickup_slots: Vec<_> = valid_slots.iter()
            .filter(|slot| *slot != pickup_slot)
            .cloned()
            .collect();

        // If we have other slots besides the pickup slot, exclude the pickup slot
        // This prevents "distributing" to the slot we picked up from when there are alternatives
        if !non_pickup_slots.is_empty() {
            valid_slots = non_pickup_slots;
        }
        // If non_pickup_slots is empty, we keep the pickup slot as a valid target
    }

    if valid_slots.is_empty() {
        return; // No valid slots to distribute to
    }

    let valid_slot_count = valid_slots.len() as u32;
    let items_per_slot = total_items / valid_slot_count;
    let remainder = total_items % valid_slot_count;
    let mut total_distributed = 0;

    // Distribute items evenly across valid slots
    for (i, &(ref container_type, slot_index)) in valid_slots.iter().enumerate() {
        if let Some(container) = container_manager.get_container_mut(container_type) {
            // Give extra items to first 'remainder' slots
            let items_for_this_slot = if i < remainder as usize {
                items_per_slot + 1
            } else {
                items_per_slot
            };

            if items_for_this_slot > 0 {
                let stack_to_place = ItemStack::new(held_stack.item.unwrap(), items_for_this_slot);

                // Track what was actually placed
                let leftover = place_stack_in_slot(container, slot_index, stack_to_place);
                let actually_placed = items_for_this_slot - leftover.map(|s| s.size).unwrap_or(0);
                total_distributed += actually_placed;
            }
        }
    }

    // Update held item based on what was actually distributed
    if total_distributed >= total_items {
        held_item.stack = None;
    } else if let Some(ref mut held_stack) = held_item.stack {
        held_stack.size = total_items - total_distributed;
        // If we have 0 items left, clear the stack
        if held_stack.size == 0 {
            held_item.stack = None;
        }
    }
}