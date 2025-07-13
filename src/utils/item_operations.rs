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

pub fn process_shift_click(
    slot_index: usize,
    source_container_type: &ContainerType,
    container_manager: &mut ContainerManager,
) {
    // Get the item stack from the source slot
    let source_container = match container_manager.get_container_mut(source_container_type) {
        Some(container) => container,
        None => return,
    };
    
    let Some(mut item_stack) = source_container.take_slot(slot_index) else {
        return; // No item to move
    };

    // Determine target containers based on source and current UI mode
    let target_containers = get_shift_click_targets(source_container_type, &container_manager.ui_mode);
    
    // Try to place the item stack in target containers
    for target_type in target_containers {
        if let Some(target_container) = container_manager.get_container_mut(&target_type) {
            if let Some(remaining_stack) = try_place_stack_in_container(target_container, item_stack) {
                item_stack = remaining_stack;
            } else {
                // All items placed successfully
                return;
            }
        }
    }

    // If there are leftover items, put them back in the original slot
    if let Some(source_container) = container_manager.get_container_mut(source_container_type) {
        source_container.set_slot(slot_index, Some(item_stack));
    }
}

fn get_shift_click_targets(source_type: &ContainerType, ui_mode: &crate::world::inventory::containers::UIMode) -> Vec<ContainerType> {
    use crate::world::inventory::containers::UIMode;
    
    match (source_type, ui_mode) {
        // From hotbar to player inventory
        (ContainerType::Hotbar, UIMode::InventoryOpen) => {
            vec![ContainerType::PlayerInventory]
        }
        // From hotbar to chest (when chest is open)
        (ContainerType::Hotbar, UIMode::ChestOpen(chest_id)) => {
            vec![ContainerType::Chest(*chest_id), ContainerType::PlayerInventory]
        }
        // From player inventory to hotbar
        (ContainerType::PlayerInventory, UIMode::InventoryOpen) => {
            vec![ContainerType::Hotbar]
        }
        // From player inventory to chest (when chest is open)
        (ContainerType::PlayerInventory, UIMode::ChestOpen(chest_id)) => {
            vec![ContainerType::Chest(*chest_id)]
        }
        // From chest to player inventory then hotbar
        (ContainerType::Chest(_), UIMode::ChestOpen(_)) => {
            vec![ContainerType::PlayerInventory, ContainerType::Hotbar]
        }
        // Default case - no valid targets
        _ => vec![]
    }
}

fn try_place_stack_in_container(container: &mut SlotContainer, mut stack: ItemStack) -> Option<ItemStack> {
    let slot_count = container.len();
    
    // First pass: try to merge with existing stacks
    for slot_index in 0..slot_count {
        if let Some(existing_stack) = container.get_slot_mut(slot_index) {
            if stack.can_merge_with(existing_stack) {
                let max_size = existing_stack.item.unwrap().properties.max_stack_size;
                let available_space = max_size.saturating_sub(existing_stack.size);
                let to_add = available_space.min(stack.size);
                
                existing_stack.size += to_add;
                stack.size -= to_add;
                
                if stack.size == 0 {
                    return None; // All items placed
                }
            }
        }
    }
    
    // Second pass: place remaining items in empty slots
    for slot_index in 0..slot_count {
        if container.get_slot(slot_index).is_none() {
            container.set_slot(slot_index, Some(stack));
            return None; // All items placed
        }
    }
    
    // Return remaining items if container is full
    Some(stack)
}