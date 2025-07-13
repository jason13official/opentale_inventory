use crate::world::inventory::containers::{ContainerManager, ContainerType};
use crate::world::inventory::item_stack::ItemStack;
use bevy::prelude::{Entity, Query, Text};

/// Converts sprite coordinates (x, y) to atlas index for an 8x9 spritesheet
pub fn sprite_coords_to_atlas_index(sprite_x: u8, sprite_y: u8) -> usize {
    (sprite_y as usize * 8) + sprite_x as usize
}

/// Check if a slot can accept items from a held stack
pub fn can_slot_accept_items(
    container_type: &ContainerType,
    slot_index: usize,
    held_stack: &ItemStack,
    container_manager: &ContainerManager,
) -> bool {
    if let Some(container) = container_manager.get_container(container_type) {
        match container.get_slot(slot_index) {
            None => true, // Empty slot can accept items
            Some(existing_stack) => {
                held_stack.can_merge_with(existing_stack) &&
                    existing_stack.size < existing_stack.item.unwrap().properties.max_stack_size
            }
        }
    } else {
        false
    }
}

/// Calculate available space in a slot for a specific item stack
pub fn calculate_available_space(
    container_type: &ContainerType,
    slot_index: usize,
    held_stack: &ItemStack,
    container_manager: &ContainerManager,
) -> u32 {
    if let Some(container) = container_manager.get_container(container_type) {
        match container.get_slot(slot_index) {
            None => held_stack.size, // Empty slot can take the whole stack
            Some(existing_stack) => {
                if held_stack.can_merge_with(existing_stack) {
                    let max_size = existing_stack.item.unwrap().properties.max_stack_size;
                    max_size - existing_stack.size
                } else {
                    0 // Can't merge
                }
            }
        }
    } else {
        0
    }
}

/// Generic function to clear text content for any text entity
pub fn clear_text<Q: bevy::ecs::query::QueryFilter>(text_entity: Entity, text_query: &mut Query<&mut Text, Q>) {
    if let Ok(mut text) = text_query.get_mut(text_entity) {
        text.sections[0].value.clear();
    }
}

/// Filters out pickup slot from valid slots if other slots are available
pub fn filter_out_pickup_slot(
    valid_slots: Vec<(ContainerType, usize)>,
    pickup_slot: &Option<(ContainerType, usize)>,
) -> Vec<(ContainerType, usize)> {
    if let Some(pickup_slot) = pickup_slot {
        let non_pickup_slots: Vec<_> = valid_slots.iter()
            .filter(|slot| *slot != pickup_slot)
            .cloned()
            .collect();
        
        if !non_pickup_slots.is_empty() {
            non_pickup_slots
        } else {
            valid_slots
        }
    } else {
        valid_slots
    }
}