use crate::world::inventory::components::{HeldItem, DragState, InventorySlot};
use crate::world::inventory::containers::{ContainerManager, ContainerType};
use crate::world::inventory::item_stack::ItemStack;
use crate::utils::item_operations::filter_valid_distribution_slots;
use crate::systems::ui::slot_utils::{can_slot_accept_items, calculate_available_space, filter_out_pickup_slot};
use bevy::prelude::Color;

/// Encapsulates drag state for a specific slot
pub struct SlotDragContext {
    pub current_slot: (ContainerType, usize),
    pub is_left_drag_target: bool,
    pub is_right_drag_target: bool,
    pub is_currently_hovered: bool,
    pub is_left_dragging: bool,
    pub is_right_dragging: bool,
    pub show_drag_highlighting: bool,
}

impl SlotDragContext {
    pub fn new(slot: &InventorySlot, drag_state: &DragState) -> Self {
        let current_slot = (slot.container_type.clone(), slot.index);
        let is_left_drag_target = drag_state.left_drag_slots.contains(&current_slot);
        let is_right_drag_target = drag_state.right_drag_slots.contains(&current_slot);
        let is_currently_hovered = drag_state.current_hovered_slot == Some(current_slot.clone());
        let is_left_dragging = drag_state.is_left_dragging;
        let is_right_dragging = drag_state.is_right_dragging;
        
        let show_drag_highlighting = (is_left_dragging && drag_state.left_drag_slots.len() > 1) ||
            (is_right_dragging && drag_state.right_drag_slots.len() > 1);
            
        Self {
            current_slot,
            is_left_drag_target,
            is_right_drag_target,
            is_currently_hovered,
            is_left_dragging,
            is_right_dragging,
            show_drag_highlighting,
        }
    }
    
    pub fn is_dragging(&self) -> bool {
        self.is_left_dragging || self.is_right_dragging
    }
}

/// Determines the appropriate border color for a slot based on its state
pub fn determine_slot_border_color(
    is_selected: bool,
    drag_context: &SlotDragContext,
    preview_count: u32,
) -> Color {
    if is_selected {
        Color::rgb(1.0, 1.0, 0.0) // Yellow for selected (highest priority)
    } else if drag_context.is_currently_hovered && drag_context.is_dragging() && drag_context.show_drag_highlighting {
        if preview_count > 0 {
            if drag_context.is_right_dragging {
                Color::rgb(0.0, 0.8, 1.0) // Light blue for right-click hover
            } else {
                Color::rgb(0.0, 1.0, 0.0) // Green for left-click hover
            }
        } else {
            Color::rgb(0.6, 0.6, 0.6) // Default border if slot can't accept items
        }
    } else if drag_context.is_left_drag_target && drag_context.is_left_dragging && drag_context.show_drag_highlighting {
        if preview_count > 0 {
            Color::rgb(0.0, 0.8, 0.0) // Darker green for left-drag target slots
        } else {
            Color::rgb(0.6, 0.6, 0.6) // Default border for invalid slots
        }
    } else if drag_context.is_right_drag_target && drag_context.is_right_dragging && drag_context.show_drag_highlighting {
        if preview_count > 0 {
            Color::rgb(0.0, 0.6, 0.8) // Darker blue for right-drag target slots
        } else {
            Color::rgb(0.6, 0.6, 0.6) // Default border for invalid slots
        }
    } else {
        Color::rgb(0.6, 0.6, 0.6) // Default border
    }
}

/// Calculate how many items would be deposited in this slot during drag distribution
pub fn calculate_drag_preview(
    current_slot: &(ContainerType, usize),
    drag_state: &DragState,
    held_item: &HeldItem,
    container_manager: &ContainerManager,
) -> u32 {
    let Some(held_stack) = &held_item.stack else { return 0; };
    
    if drag_state.left_drag_slots.is_empty() {
        return 0;
    }

    // Single slot case - check actual capacity
    if drag_state.left_drag_slots.len() == 1 {
        if drag_state.left_drag_slots.contains(current_slot) {
            let available_space = calculate_available_space(
                &current_slot.0,
                current_slot.1,
                held_stack,
                container_manager
            );
            held_stack.size.min(available_space)
        } else {
            0
        }
    } else {
        // Multi-slot distribution case
        let valid_slots = filter_valid_distribution_slots(
            &drag_state.left_drag_slots,
            held_stack,
            container_manager
        );

        let filtered_slots = filter_out_pickup_slot(valid_slots, &drag_state.pickup_slot);

        if !filtered_slots.contains(current_slot) {
            return 0;
        }

        // Calculate what the distribution logic would try to place
        let valid_slot_count = filtered_slots.len() as u32;
        let items_per_slot = held_stack.size / valid_slot_count;
        let remainder = held_stack.size % valid_slot_count;
        
        let intended_amount = if let Some(slot_index) = filtered_slots.iter().position(|s| s == current_slot) {
            if slot_index < remainder as usize {
                items_per_slot + 1
            } else {
                items_per_slot
            }
        } else {
            0
        };

        // Now check how much can actually fit in this slot
        let available_space = calculate_available_space(
            &current_slot.0,
            current_slot.1,
            held_stack,
            container_manager
        );
        intended_amount.min(available_space)
    }
}

/// Calculate how many items would remain in hand after drag distribution
pub fn calculate_remaining_after_drag(
    held_stack: &ItemStack,
    drag_state: &DragState,
    container_manager: &ContainerManager,
) -> u32 {
    let total_items = held_stack.size;
    
    // Determine which drag slots to use based on active drag type
    let drag_slots = if drag_state.is_right_dragging {
        &drag_state.right_drag_slots
    } else {
        &drag_state.left_drag_slots
    };
    
    // For right-click drag, calculate items that would be placed (1 per valid slot)
    if drag_state.is_right_dragging {
        let mut can_place_count = 0;
        
        for &(ref container_type, slot_index) in drag_slots {
            if can_slot_accept_items(container_type, slot_index, held_stack, container_manager) {
                can_place_count += 1;
            }
        }
        
        return total_items.saturating_sub(can_place_count);
    }
    
    // Left-click drag logic (existing logic)
    // Single slot case - all items would be placed if possible
    if drag_slots.len() == 1 {
        if let Some((container_type, slot_index)) = drag_slots.first() {
            let available_space = calculate_available_space(
                container_type,
                *slot_index,
                held_stack,
                container_manager
            );
            let can_place = held_stack.size.min(available_space);
            total_items - can_place
        } else {
            total_items // No slot found, keep all items
        }
    } else {
        // Multi-slot distribution case
        let valid_slots = filter_valid_distribution_slots(
            drag_slots,
            held_stack,
            container_manager
        );

        let filtered_slots = filter_out_pickup_slot(valid_slots, &drag_state.pickup_slot);

        if filtered_slots.is_empty() {
            return total_items; // No valid slots, keep all items
        }

        let valid_slot_count = filtered_slots.len() as u32;
        let items_per_slot = total_items / valid_slot_count;
        let remainder = total_items % valid_slot_count;
        let mut total_can_place = 0;

        // Calculate how much can actually be placed in each slot
        for (i, &(ref container_type, slot_index)) in filtered_slots.iter().enumerate() {
            if let Some(_container) = container_manager.get_container(container_type) {
                // Items intended for this slot
                let intended_amount = if i < remainder as usize {
                    items_per_slot + 1
                } else {
                    items_per_slot
                };

                // Check how much can actually fit
                let available_space = calculate_available_space(
                    container_type,
                    slot_index,
                    held_stack,
                    container_manager
                );
                let can_place = intended_amount.min(available_space);
                
                total_can_place += can_place;
            }
        }

        total_items.saturating_sub(total_can_place)
    }
}