use crate::utils::item_operations::{deposit_single_item, filter_valid_distribution_slots, place_stack_in_slot, process_left_click};
use crate::utils::slot_finder::find_slot_under_cursor;
use crate::world::inventory::components::{DragState, HeldItem, InventorySlot};
use crate::world::inventory::containers::ContainerManager;
use crate::world::inventory::item_stack::ItemStack;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::{EventReader, GlobalTransform, MouseButton, Node, Query, ResMut, Window};

pub fn handle_left_drag_deposit(
    mouse_events: EventReader<MouseButtonInput>,
    container_manager: ResMut<ContainerManager>,
    held_item: ResMut<HeldItem>,
    drag_state: ResMut<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    handle_drag_deposit(
        mouse_events,
        container_manager,
        held_item,
        drag_state,
        slot_query,
        windows,
        MouseButton::Left,
    );
}

pub fn handle_right_drag_deposit(
    mouse_events: EventReader<MouseButtonInput>,
    container_manager: ResMut<ContainerManager>,
    held_item: ResMut<HeldItem>,
    drag_state: ResMut<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    handle_drag_deposit(
        mouse_events,
        container_manager,
        held_item,
        drag_state,
        slot_query,
        windows,
        MouseButton::Right,
    );
}

fn handle_drag_deposit(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    mut drag_state: ResMut<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
    button: MouseButton,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    
    let is_left = button == MouseButton::Left;
    
    if is_left {
        drag_state.was_left_dragging_this_frame = false;
    } else {
        drag_state.was_right_dragging_this_frame = false;
    }

    for event in mouse_events.read() {
        if event.button == button {
            match event.state {
                ButtonState::Pressed => {
                    if held_item.stack.is_some() {
                        if is_left {
                            drag_state.is_left_dragging = true;
                            drag_state.left_drag_slots.clear();
                        } else {
                            drag_state.is_right_dragging = true;
                            drag_state.right_drag_slots.clear();
                        }
                    }
                }
                ButtonState::Released => {
                    let is_dragging = if is_left { drag_state.is_left_dragging } else { drag_state.is_right_dragging };
                    
                    if is_dragging {
                        if is_left {
                            drag_state.was_left_dragging_this_frame = true;
                            process_drag_end(&mut container_manager, &mut held_item, &mut drag_state);
                        } else {
                            drag_state.was_right_dragging_this_frame = true;
                            process_right_drag_end(&mut container_manager, &mut held_item, &drag_state);
                        }
                    }
                    
                    if is_left {
                        drag_state.is_left_dragging = false;
                        drag_state.left_drag_slots.clear();
                    } else {
                        drag_state.is_right_dragging = false;
                        drag_state.right_drag_slots.clear();
                    }
                    drag_state.current_hovered_slot = None;
                }
            }
        }
    }
    
    let is_dragging = if is_left { drag_state.is_left_dragging } else { drag_state.is_right_dragging };
    
    if is_dragging && held_item.stack.is_some() {
        if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
            if is_left {
                drag_state.add_left_drag_slot(slot.container_type.clone(), slot.index);
            } else {
                drag_state.add_right_drag_slot(slot.container_type.clone(), slot.index);
            }
            drag_state.current_hovered_slot = Some((slot.container_type.clone(), slot.index));
        } else {
            drag_state.current_hovered_slot = None;
        }
    } else {
        drag_state.current_hovered_slot = None;
    }
}

pub(crate) fn process_right_drag_end(
    container_manager: &mut ContainerManager,
    held_item: &mut HeldItem,
    drag_state: &DragState,
) {
    let Some(_) = &held_item.stack else {
        return;
    };

    // If no slots were dragged over, do nothing (keep holding item)
    if drag_state.right_drag_slots.is_empty() {
        return;
    }

    // Deposit one item per slot that was dragged over
    for (container_type, slot_index) in &drag_state.right_drag_slots {
        if let Some(container) = container_manager.get_container_mut(container_type) {
            deposit_single_item(*slot_index, container, held_item);
            
            // If we run out of items, stop
            if held_item.stack.is_none() {
                break;
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

    let valid_slots = get_filtered_distribution_slots(drag_state, held_stack, container_manager);
    if valid_slots.is_empty() {
        return;
    }

    let distribution = calculate_item_distribution(total_items, valid_slots.len() as u32);
    let total_distributed = execute_distribution(container_manager, held_stack, &valid_slots, &distribution);
    
    update_held_item_after_distribution(held_item, total_items, total_distributed);
}

fn get_filtered_distribution_slots(
    drag_state: &DragState,
    held_stack: &ItemStack,
    container_manager: &ContainerManager,
) -> Vec<(crate::world::inventory::containers::ContainerType, usize)> {
    let mut valid_slots = filter_valid_distribution_slots(
        &drag_state.left_drag_slots,
        held_stack,
        container_manager
    );

    // Only exclude pickup slot if we have other valid slots to distribute to
    if let Some(pickup_slot) = &drag_state.pickup_slot {
        let non_pickup_slots: Vec<_> = valid_slots.iter()
            .filter(|slot| *slot != pickup_slot)
            .cloned()
            .collect();

        if !non_pickup_slots.is_empty() {
            valid_slots = non_pickup_slots;
        }
    }

    valid_slots
}

fn calculate_item_distribution(total_items: u32, slot_count: u32) -> (u32, u32) {
    let items_per_slot = total_items / slot_count;
    let remainder = total_items % slot_count;
    (items_per_slot, remainder)
}

fn execute_distribution(
    container_manager: &mut ContainerManager,
    held_stack: &ItemStack,
    valid_slots: &[(crate::world::inventory::containers::ContainerType, usize)],
    distribution: &(u32, u32),
) -> u32 {
    let (items_per_slot, remainder) = *distribution;
    let mut total_distributed = 0;

    for (i, &(ref container_type, slot_index)) in valid_slots.iter().enumerate() {
        if let Some(container) = container_manager.get_container_mut(container_type) {
            let items_for_this_slot = if i < remainder as usize {
                items_per_slot + 1
            } else {
                items_per_slot
            };

            if items_for_this_slot > 0 {
                let stack_to_place = ItemStack::new(held_stack.item.unwrap(), items_for_this_slot);
                let leftover = place_stack_in_slot(container, slot_index, stack_to_place);
                let actually_placed = items_for_this_slot - leftover.map(|s| s.size).unwrap_or(0);
                total_distributed += actually_placed;
            }
        }
    }

    total_distributed
}

fn update_held_item_after_distribution(held_item: &mut HeldItem, total_items: u32, total_distributed: u32) {
    if total_distributed >= total_items {
        held_item.stack = None;
    } else if let Some(ref mut held_stack) = held_item.stack {
        held_stack.size = total_items - total_distributed;
        if held_stack.size == 0 {
            held_item.stack = None;
        }
    }
}