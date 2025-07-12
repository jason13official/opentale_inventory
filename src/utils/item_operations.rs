use crate::world::inventory::components::HeldItem;
use crate::world::inventory::containers::{ContainerManager, ContainerType};
use crate::world::inventory::inventory::SlotContainer;
use crate::world::inventory::item_stack::ItemStack;

pub fn process_left_click(slot_index: usize, inventory: &mut SlotContainer, held_item: &mut HeldItem) {
    let slot_stack = inventory.take_slot(slot_index);

    match (&mut held_item.stack, slot_stack) {
        (None, Some(stack)) => {
            held_item.stack = Some(stack);
        }
        (Some(held_stack), Some(mut clicked_stack)) => {
            if held_stack.can_merge_with(&clicked_stack) {
                let fully_merged = held_stack.try_merge(&mut clicked_stack);
                inventory.set_slot(slot_index, Some(clicked_stack));
                if fully_merged {
                    held_item.stack = None;
                }
            } else {
                inventory.set_slot(slot_index, Some(held_stack.clone()));
                *held_stack = clicked_stack;
            }
        }
        (Some(held_stack), None) => {
            inventory.set_slot(slot_index, Some(held_stack.clone()));
            held_item.stack = None;
        }
        (None, None) => {}
    }
}

pub fn process_right_click(slot_index: usize, inventory: &mut SlotContainer, held_item: &mut HeldItem) {
    match (&mut held_item.stack, inventory.get_slot_mut(slot_index)) {
        (None, Some(slot_stack)) => {
            if let Some(half_stack) = slot_stack.split_half() {
                held_item.stack = Some(half_stack);
            } else {
                let removed_stack = inventory.take_slot(slot_index);
                held_item.stack = removed_stack;
            }
        }
        (Some(held_stack), Some(slot_stack)) => {
            if held_stack.can_merge_with(slot_stack) && held_stack.size > 0 {
                if slot_stack.size < slot_stack.item.unwrap().properties.max_stack_size {
                    held_stack.size -= 1;
                    slot_stack.size += 1;
                    if held_stack.size == 0 {
                        held_item.stack = None;
                    }
                }
            }
        }
        (Some(held_stack), None) => {
            if held_stack.size > 1 {
                held_stack.size -= 1;
                inventory.set_slot(slot_index, Some(ItemStack::new(
                    held_stack.item.unwrap().clone(),
                    1
                )));
            } else {
                inventory.set_slot(slot_index, Some(held_stack.clone()));
                held_item.stack = None;
            }
        }
        (None, None) => {}
    }
}

pub fn deposit_single_item(slot_index: usize, container: &mut SlotContainer, held_item: &mut HeldItem) {
    let Some(held_stack) = &mut held_item.stack else { return; };

    if held_stack.size == 0 {
        held_item.stack = None;
        return;
    }

    match container.get_slot_mut(slot_index) {
        None => {
            held_stack.size -= 1;
            let single_item = ItemStack::new(held_stack.item.unwrap(), 1);
            container.set_slot(slot_index, Some(single_item));
            if held_stack.size == 0 {
                held_item.stack = None;
            }
        }
        Some(slot_stack) => {
            if held_stack.can_merge_with(slot_stack) {
                let max_size = slot_stack.item.unwrap().properties.max_stack_size;
                if slot_stack.size < max_size {
                    held_stack.size -= 1;
                    slot_stack.size += 1;
                    if held_stack.size == 0 {
                        held_item.stack = None;
                    }
                }
            }
        }
    }
}

pub fn place_stack_in_slot(
    container: &mut SlotContainer,
    slot_index: usize,
    stack: ItemStack,
) -> Option<ItemStack> {
    match container.get_slot_mut(slot_index) {
        None => {
            // Empty slot - place the entire stack
            container.set_slot(slot_index, Some(stack));
            None
        }
        Some(existing_stack) => {
            // Slot has items - try to merge
            if stack.can_merge_with(existing_stack) {
                let max_size = existing_stack.item.unwrap().properties.max_stack_size;
                let available_space = max_size.saturating_sub(existing_stack.size);
                let to_add = available_space.min(stack.size);

                existing_stack.size += to_add;

                // Return leftover if any
                if to_add == stack.size {
                    None // All items placed
                } else {
                    Some(ItemStack::new(stack.item.unwrap(), stack.size - to_add))
                }
            } else {
                // Can't merge - return the entire stack
                Some(stack)
            }
        }
    }
}

pub fn filter_valid_distribution_slots(
    slots: &[(ContainerType, usize)],
    held_stack: &ItemStack,
    container_manager: &ContainerManager,
) -> Vec<(ContainerType, usize)> {
    slots.iter()
        .filter(|(container_type, slot_index)| {
            if let Some(container) = container_manager.get_container(container_type) {
                match container.get_slot(*slot_index) {
                    None => true, // Empty slot is always valid
                    Some(existing_stack) => {
                        // Check if items can merge and there's space
                        held_stack.can_merge_with(existing_stack) &&
                            existing_stack.size < existing_stack.item.unwrap().properties.max_stack_size
                    }
                }
            } else {
                false
            }
        })
        .cloned()
        .collect()
}