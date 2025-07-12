use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::input::mouse::MouseButtonInput;
use crate::world::inventory::containers::{CloseChestEvent, CloseInventoryEvent, ContainerManager, ContainerType, ContainerUI, OpenChestEvent, OpenInventoryEvent};
use crate::world::inventory::inventory::SlotContainer;
use crate::world::inventory::item_stack::ItemStack;
use crate::world::inventory::ui::{create_container_ui, create_minecraft_ui};
use super::components::*;

#[derive(Component)]
pub struct UIRebuildNeeded;

pub fn handle_container_events(
    mut container_manager: ResMut<ContainerManager>,
    mut open_inventory_events: EventReader<OpenInventoryEvent>,
    mut close_inventory_events: EventReader<CloseInventoryEvent>,
    mut open_chest_events: EventReader<OpenChestEvent>,
    mut close_chest_events: EventReader<CloseChestEvent>,
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

    for _event in open_chest_events.read() {
        container_manager.open_chest();
        needs_rebuild = true;
    }

    for _event in close_chest_events.read() {
        container_manager.close_chest();
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
            crate::world::inventory::containers::UIMode::ChestOpen => {
                close_inventory_events.send(CloseInventoryEvent);
            }
        }
    }

    if keys.just_pressed(KeyCode::KeyC) {
        match container_manager.ui_mode {
            crate::world::inventory::containers::UIMode::ChestOpen => {
                close_chest_events.send(CloseChestEvent);
            }
            _ => {
                open_chest_events.send(OpenChestEvent);
            }
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        match container_manager.ui_mode {
            crate::world::inventory::containers::UIMode::ChestOpen => {
                close_chest_events.send(CloseChestEvent);
            }
            crate::world::inventory::containers::UIMode::InventoryOpen => {
                close_inventory_events.send(CloseInventoryEvent);
            }
            _ => {}
        }
    }
}

pub fn update_slot_visuals(
    container_manager: Res<ContainerManager>,
    mut slot_query: Query<(&InventorySlot, &mut Children, &mut BackgroundColor)>,
    mut text_query: Query<&mut Text>,
    ui_query: Query<Entity, With<ContainerUI>>,
    rebuild_query: Query<Entity, With<UIRebuildNeeded>>,
) {
    if !rebuild_query.is_empty() || ui_query.is_empty() {
        return;
    }

    for (slot, children, mut bg_color) in &mut slot_query {
        if let Some(container) = container_manager.get_container(&slot.container_type) {
            if let Some(item) = container.get_slot(slot.index) {
                if let Some(text_entity) = children.first() {
                    if let Ok(mut text) = text_query.get_mut(*text_entity) {
                        text.sections[0].value = format_item_display(item);
                    }
                }
                *bg_color = Color::rgb(0.3, 0.3, 0.7).into();
            } else {
                if let Some(text_entity) = children.first() {
                    if let Ok(mut text) = text_query.get_mut(*text_entity) {
                        text.sections[0].value.clear();
                    }
                }
                *bg_color = Color::rgb(0.4, 0.4, 0.4).into();
            }
        }
    }
}

pub fn update_held_item_display(
    held_item: Res<HeldItem>,
    mut display_query: Query<(&mut Style, &Children), With<HeldItemDisplay>>,
    mut text_query: Query<&mut Text>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    for (mut style, children) in &mut display_query {
        style.left = Val::Px(cursor_pos.x + 10.0);
        style.top = Val::Px(cursor_pos.y + 10.0);

        if let Some(text_entity) = children.first() {
            if let Ok(mut text) = text_query.get_mut(*text_entity) {
                if let Some(stack) = &held_item.stack {
                    text.sections[0].value = format_item_display(stack);
                } else {
                    text.sections[0].value.clear();
                }
            }
        }
    }
}

// RIGHT-CLICK SYSTEMS (unchanged)
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

fn process_right_click(slot_index: usize, inventory: &mut SlotContainer, held_item: &mut HeldItem) {
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

fn deposit_single_item(slot_index: usize, container: &mut SlotContainer, held_item: &mut HeldItem) {
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

// FIXED: Even distribution with better filtering and math
fn distribute_items_evenly(
    container_manager: &mut ContainerManager,
    held_item: &mut HeldItem,
    drag_state: &DragState,
) {
    let Some(held_stack) = &held_item.stack else { return; };
    let total_items = held_stack.size;

    // FIXED: Filter valid slots and exclude pickup slot
    let mut valid_slots = filter_valid_distribution_slots(
        &drag_state.left_drag_slots,
        held_stack,
        container_manager
    );

    // Remove pickup slot from distribution
    if let Some(pickup_slot) = &drag_state.pickup_slot {
        valid_slots.retain(|slot| slot != pickup_slot);
    }

    if valid_slots.is_empty() {
        return;
    }

    let valid_slot_count = valid_slots.len() as u32;
    let items_per_slot = total_items / valid_slot_count;
    let remainder = total_items % valid_slot_count;
    let mut total_distributed = 0;

    // FIXED: Better distribution logic
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

                // FIXED: Track what was actually placed
                let leftover = place_stack_in_slot(container, slot_index, stack_to_place);
                let actually_placed = items_for_this_slot - leftover.map(|s| s.size).unwrap_or(0);
                total_distributed += actually_placed;
            }
        }
    }

    // FIXED: Update held item based on what was actually distributed
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

// FIXED: Better validation for distribution slots
fn filter_valid_distribution_slots(
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

// FIXED: Better stack placement with overflow handling
fn place_stack_in_slot(
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

// Helper function (unchanged)
fn find_slot_under_cursor<'a>(
    cursor_pos: Vec2,
    slot_query: &'a Query<(&InventorySlot, &GlobalTransform, &Node)>,
) -> Option<&'a InventorySlot> {
    for (slot, transform, node) in slot_query {
        let slot_pos = transform.translation().truncate();
        let slot_size = node.size();
        let slot_rect = bevy::math::Rect::from_center_size(slot_pos, slot_size);

        if slot_rect.contains(cursor_pos) {
            return Some(slot);
        }
    }
    None
}

// Other helper functions (unchanged)
fn format_item_display(stack: &ItemStack) -> String {
    if stack.size > 1 {
        format!("{}\n{}", stack.item.unwrap().display_name, stack.size)
    } else {
        stack.item.unwrap().display_name.to_string()
    }
}

fn process_left_click(slot_index: usize, inventory: &mut SlotContainer, held_item: &mut HeldItem) {
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

// Updated system to clear the "was dragging this frame" flag at the end of each frame
pub fn clear_drag_frame_flags(mut drag_state: ResMut<DragState>) {
    // Clear the frame flags at the end of each frame
    drag_state.was_left_dragging_this_frame = false;
    drag_state.was_right_dragging_this_frame = false;
}

pub fn handle_left_clicks(
    mut interaction_query: Query<(&Interaction, &InventorySlot), (Changed<Interaction>, With<Button>)>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    mut drag_state: ResMut<DragState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    // Clear pickup flag each frame
    drag_state.clear_pickup_flag();

    for (interaction, slot) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(container) = container_manager.get_container_mut(&slot.container_type) {
                let had_item_before = held_item.stack.is_some();
                let slot_has_item = container.get_slot(slot.index).is_some();

                if !had_item_before && slot_has_item {
                    // Picking up an item
                    process_left_click(slot.index, container, &mut held_item);
                    drag_state.mark_pickup(slot.container_type.clone(), slot.index);
                } else if had_item_before {
                    // Already holding item - check if we should start dragging or place immediately
                    if mouse_input.pressed(MouseButton::Left) {
                        // Mouse is pressed - start or continue drag
                        if !drag_state.is_left_dragging {
                            drag_state.start_left_drag(held_item.stack.as_ref().unwrap().size);
                        }
                        drag_state.add_left_drag_slot(slot.container_type.clone(), slot.index);
                    }
                    // Note: We don't place items on single click anymore - only on mouse release
                }
            }
        }
    }
}

pub fn handle_left_mouse_release(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    mut drag_state: ResMut<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    for event in mouse_events.read() {
        if event.button == MouseButton::Left && event.state == ButtonState::Released {
            // Only process if we have an item
            if held_item.stack.is_some() {
                if drag_state.is_left_dragging {
                    // We were dragging - process distribution
                    process_drag_end(&mut container_manager, &mut held_item, &mut drag_state);
                } else {
                    // Single click - place entire stack in slot under cursor
                    let Ok(window) = windows.get_single() else {
                        drag_state.reset_left_drag();
                        return;
                    };
                    let Some(cursor_pos) = window.cursor_position() else {
                        drag_state.reset_left_drag();
                        return;
                    };

                    if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
                        if let Some(container) = container_manager.get_container_mut(&slot.container_type) {
                            process_left_click(slot.index, container, &mut held_item);
                        }
                    }
                }
            }
            drag_state.reset_left_drag();
        }
    }
}

pub fn handle_left_drag_movement(
    mut drag_state: ResMut<DragState>,
    held_item: Res<HeldItem>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    // Only track movement if we have an item and mouse is pressed
    if held_item.stack.is_none() || !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    // If we're moving with mouse pressed and haven't started dragging yet, start now
    if !drag_state.is_left_dragging {
        drag_state.start_left_drag(held_item.stack.as_ref().unwrap().size);
    }

    if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
        drag_state.add_left_drag_slot(slot.container_type.clone(), slot.index);
    }
}

// Simplified drag end processing
fn process_drag_end(
    container_manager: &mut ContainerManager,
    held_item: &mut HeldItem,
    drag_state: &mut DragState,
) {
    let Some(held_stack) = &held_item.stack else {
        return;
    };

    // If no slots were dragged over, do nothing (keep holding item)
    if drag_state.left_drag_slots.is_empty() {
        return;
    }

    // If dragging back to pickup slot only, do nothing (keep holding item)
    let is_single_slot_back_to_pickup = if let Some(pickup_slot) = &drag_state.pickup_slot {
        drag_state.left_drag_slots.len() == 1 &&
            drag_state.left_drag_slots.first() == Some(pickup_slot)
    } else {
        false
    };

    if is_single_slot_back_to_pickup {
        return; // Keep holding the item
    }

    // Single slot distribution
    if drag_state.left_drag_slots.len() == 1 {
        if let Some((container_type, slot_index)) = drag_state.left_drag_slots.first() {
            // Skip if this is the pickup slot
            if Some((container_type.clone(), *slot_index)) != drag_state.pickup_slot {
                if let Some(container) = container_manager.get_container_mut(container_type) {
                    process_left_click(*slot_index, container, held_item);
                }
            }
        }
    } else {
        // Multi-slot distribution
        distribute_items_evenly(container_manager, held_item, drag_state);
    }
}