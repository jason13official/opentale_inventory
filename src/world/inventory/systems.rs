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

    // Handle inventory opening/closing
    for _event in open_inventory_events.read() {
        container_manager.open_inventory();
        needs_rebuild = true;
    }

    for _event in close_inventory_events.read() {
        container_manager.close_inventory();
        needs_rebuild = true;
    }

    // Handle chest opening
    for _event in open_chest_events.read() {
        container_manager.open_chest();
        needs_rebuild = true;
    }

    // Handle chest closing
    for _event in close_chest_events.read() {
        container_manager.close_chest();
        needs_rebuild = true;
    }

    // Only trigger UI rebuild if needed
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
        // Despawn rebuild markers
        for entity in rebuild_query.iter() {
            commands.entity(entity).despawn();
        }

        // Despawn existing container UI
        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Create new UI for all active containers
        create_minecraft_ui(&mut commands, &asset_server, &container_manager);
    }
}

/// Handle keyboard input for container switching
pub fn handle_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut open_inventory_events: EventWriter<OpenInventoryEvent>,
    mut close_inventory_events: EventWriter<CloseInventoryEvent>,
    mut open_chest_events: EventWriter<OpenChestEvent>,
    mut close_chest_events: EventWriter<CloseChestEvent>,
    container_manager: Res<ContainerManager>,
) {
    // E key to open/close inventory
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

    // C key to open/close chest (for testing)
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

    // Escape key to close any open container
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

/// Updated left click handler
pub fn handle_left_clicks(
    mut interaction_query: Query<(&Interaction, &InventorySlot), (Changed<Interaction>, With<Button>)>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
) {
    for (interaction, slot) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(container) = container_manager.get_container_mut(&slot.container_type) {
                process_left_click(slot.index, container, &mut held_item);
            }
        }
    }
}

/// Updated right click handler
pub fn handle_right_clicks(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    for event in mouse_events.read() {
        if event.button == MouseButton::Right && event.state.is_pressed() {
            if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
                let active_container = container_manager.get_container_mut(&slot.container_type);
                process_right_click(slot.index, active_container.unwrap(), &mut held_item);
            }
        }
    }
}

/// Updated slot visuals - only run when UI is stable
pub fn update_slot_visuals(
    container_manager: Res<ContainerManager>,
    mut slot_query: Query<(&InventorySlot, &mut Children, &mut BackgroundColor)>,
    mut text_query: Query<&mut Text>,
    ui_query: Query<Entity, With<ContainerUI>>,
    rebuild_query: Query<Entity, With<UIRebuildNeeded>>,
) {
    // Don't update visuals if UI is being rebuilt
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

/// Updates the display of the item being held by the cursor
pub fn update_held_item_display(
    held_item: Res<HeldItem>, // the item we're dragging around
    mut display_query: Query<(&mut Style, &Children), With<HeldItemDisplay>>, // query for our UI element that shows the held item
    mut text_query: Query<&mut Text>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    // update the position and content of the held item
    for (mut style, children) in &mut display_query {

        // Position the display near the cursor
        style.left = Val::Px(cursor_pos.x + 10.0);
        style.top = Val::Px(cursor_pos.y + 10.0);

        // Update the text content
        if let Some(text_entity) = children.first() {
            if let Ok(mut text) = text_query.get_mut(*text_entity) {
                if let Some(stack) = &held_item.stack {
                    text.sections[0].value = format_item_display(stack);
                }
                else {
                    // We're not holding anything, so show nothing
                    // Revolutionary UI design right here
                    text.sections[0].value.clear();
                }
            }
        }
    }
}

/// Finds which inventory slot is under the cursor position
///
/// CAUTION: MATH, RECTANGLES, GEOMETRIC HORROR
// fn find_slot_under_cursor<'a>(
//     cursor_pos: Vec2,
//     // query for all of our inventory slots
//     slot_query: &'a Query<(&InventorySlot, &GlobalTransform, &Node)>,
// ) -> Option<&'a InventorySlot> {
//
//     // if we had thousands of slots we would probably do something more efficient here,
//     // but typically we'll have under 100 per menu and pagination would help
//
//     for (slot, transform, node) in slot_query {
//
//         // get the slot's position as a Vec2
//         let slot_pos = transform.translation().truncate();
//
//         // get the slot's width and size as a Vec2
//         let slot_size = node.size();
//
//         // create a rectangle representing the clickable area
//         let slot_rect = bevy::math::Rect::from_center_size(slot_pos, slot_size);
//
//         // is the cursor in the slot? if so then return the slot.
//         if slot_rect.contains(cursor_pos) {
//             return Some(slot);
//         }
//     }
//
//     // we checked every slot, and the cursor wasn't over any of them.
//     None
// }

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

/// Formats an item stack for display (handles singular vs plural)
fn format_item_display(stack: &ItemStack) -> String {
    if stack.size > 1 {
        format!("{}\n{}", stack.item.unwrap().display_name, stack.size)
    } else {
        stack.item.unwrap().display_name.to_string()
    }
}

/// Processes a left-click on an inventory slot with 4 different scenarios:
/// 1. Pick up an item from a slot (empty hand + full slot)
/// 2. Put down an item onto a slot (full hand + empty slot)
/// 3. Swap items (full hand + full slot, different items)
/// 4. Merge items (full hand + full slot, same items)
fn process_left_click(slot_index: usize, inventory: &mut SlotContainer, held_item: &mut HeldItem) {
    // grab whatever is in the slot (this empties the slot)
    let slot_stack = inventory.take_slot(slot_index);

    // Now we have a 2x2 matrix of possibilities:
    // Hand empty/full vs Slot empty/full
    // Let's handle each case using pattern matching!
    match (&mut held_item.stack, slot_stack) {

        // Case 1: Empty hand, full slot
        // Translation: "Pick up the item"
        (None, Some(stack)) => {
            held_item.stack = Some(stack); // Yoink! It's ours now.
        }

        // Case 2: Full hand, full slot
        // Translation: "This is where things get complicated"
        (Some(held_stack), Some(mut clicked_stack)) => {

            // Can we merge these items together?
            if held_stack.can_merge_with(&clicked_stack) {

                // merge partially or completely
                let fully_merged = held_stack.try_merge(&mut clicked_stack);

                // Put the (possibly modified) clicked stack back in the slot
                inventory.set_slot(slot_index, Some(clicked_stack));

                if fully_merged {
                    held_item.stack = None; // We are now empty-handed and free to pick up more
                }

                // If we didn't fully merge, we still have some items in our hand
            }
            else {
                // Items can't merge, so we'll just swap them
                // "I'll trade you my sword for your apple"
                inventory.set_slot(slot_index, Some(held_stack.clone()));
                *held_stack = clicked_stack; // Now we're holding what used to be in the slot
            }
        }

        // Case 3: Full hand, empty slot
        // Translation: "Put the item down"
        (Some(held_stack), None) => {
            // Place our held item in the empty slot
            inventory.set_slot(slot_index, Some(held_stack.clone()));
            held_item.stack = None; // We're no longer holding anything
        }

        // Case 4: Empty hand, empty slot
        (None, None) => {
            // There's literally nothing to do here, so we do nothing
            // This is peak efficiency: doing nothing perfectly
        }
    }
}

/// Processes a right-click on an inventory slot, very contextual.
/// - Empty hand + full slot = pick up half the stack
/// - Full hand + compatible slot = place one item
/// - Full hand + empty slot = place one item
/// - Empty hand + empty slot = do nothing (obviously)
fn process_right_click(slot_index: usize, inventory: &mut SlotContainer, held_item: &mut HeldItem) {

    // More pattern matching
    match (&mut held_item.stack, inventory.get_slot_mut(slot_index)) {

        // Case 1: Empty hand, full slot
        // Translation: "Give me half of what's in there"
        (None, Some(slot_stack)) => {
            // Try to split the stack in half
            if let Some(half_stack) = slot_stack.split_half() {
                held_item.stack = Some(half_stack); // We now hold half the items
            }
            else {
                // if splitting failed, just grab the item into our empty hand
                let removed_stack = inventory.take_slot(slot_index);
                held_item.stack = removed_stack; // Yoink! It's ours now.
            }
        }

        // Case 2: Full hand, full slot (compatible items)
        // Translation: "Let me put one item from my hand into that stack"
        (Some(held_stack), Some(slot_stack)) => {

            // Can we merge these items? And do we have any to give?
            if held_stack.can_merge_with(slot_stack) && held_stack.size > 0 {

                // Is there room in the slot for one more item?
                if slot_stack.size < slot_stack.item.unwrap().properties.max_stack_size {

                    // Transfer one item from hand to slot
                    held_stack.size -= 1;  // We have one less
                    slot_stack.size += 1;  // Slot has one more

                    // If we just gave away our last item, clear our hand
                    if held_stack.size == 0 {
                        held_item.stack = None; // We are now empty-handed
                    }
                }

                // If the slot is full, we can't add anything, so we do nothing
            }

            // If the items can't merge, we also do nothing
            // Right-click doesn't do swapping, that's left-click's job
        }

        // Case 3: Full hand, empty slot
        // Translation: "Put one item from my hand into this empty slot"
        (Some(held_stack), None) => {

            if held_stack.size > 1 {
                // We have multiple items, so we can spare one
                held_stack.size -= 1; // We keep the rest

                // Create a new stack with just one item for the slot
                inventory.set_slot(slot_index, Some(ItemStack::new(
                    held_stack.item.unwrap().clone(),
                    1
                )));
            } else {
                // We only have one item, so we give it all up
                // This is the "place the last item" case
                inventory.set_slot(slot_index, Some(held_stack.clone()));
                held_item.stack = None; // We are now empty-handed
            }
        }

        // Case 4: Empty hand, empty slot
        (None, None) => {
            // doing nothing perfectly
        }
    }
}

pub fn handle_drag_deposit(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut container_manager: ResMut<ContainerManager>,
    mut held_item: ResMut<HeldItem>,
    mut drag_state: ResMut<DragState>,
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    // Reset the frame flag at the start of each frame
    drag_state.was_dragging_this_frame = false;

    // Handle mouse button events
    for event in mouse_events.read() {
        if event.button == MouseButton::Right {
            match event.state {
                ButtonState::Pressed => {
                    // Start dragging if we have an item
                    if held_item.stack.is_some() {
                        drag_state.is_right_dragging = true;
                        drag_state.last_hovered_slot = None;
                    }
                }
                ButtonState::Released => {
                    // Mark that we were dragging when the button was released
                    if drag_state.is_right_dragging {
                        drag_state.was_dragging_this_frame = true;
                    }

                    // Stop dragging
                    drag_state.is_right_dragging = false;
                    drag_state.last_hovered_slot = None;
                }
            }
        }
    }

    // If we're dragging with an item, check for slot hovers
    if drag_state.is_right_dragging && held_item.stack.is_some() {
        if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
            let current_slot = (slot.container_type.clone(), slot.index);

            // Only deposit if this is a new slot we haven't visited yet
            if drag_state.last_hovered_slot != Some(current_slot.clone()) {
                drag_state.last_hovered_slot = Some(current_slot);

                // Try to deposit one item
                if let Some(container) = container_manager.get_container_mut(&slot.container_type) {
                    deposit_single_item(slot.index, container, &mut held_item);
                }
            }
        }
    }
}

fn deposit_single_item(slot_index: usize, container: &mut SlotContainer, held_item: &mut HeldItem) {
    let Some(held_stack) = &mut held_item.stack else { return; };

    // We need at least one item to deposit
    if held_stack.size == 0 {
        held_item.stack = None;
        return;
    }

    match container.get_slot_mut(slot_index) {
        // Slot is empty - place one item
        None => {
            held_stack.size -= 1;

            // Create a new stack with just one item for the slot
            let single_item = ItemStack::new(held_stack.item.unwrap(), 1);
            container.set_slot(slot_index, Some(single_item));

            // Clear held item if we just placed our last one
            if held_stack.size == 0 {
                held_item.stack = None;
            }
        }

        // Slot has an item - try to add one if compatible
        Some(slot_stack) => {
            // Check if items are compatible and slot isn't full
            if held_stack.can_merge_with(slot_stack) {
                let max_size = slot_stack.item.unwrap().properties.max_stack_size;

                if slot_stack.size < max_size {
                    // Add one item to the slot
                    held_stack.size -= 1;
                    slot_stack.size += 1;

                    // Clear held item if we just placed our last one
                    if held_stack.size == 0 {
                        held_item.stack = None;
                    }
                }
            }
            // If items aren't compatible or slot is full, do nothing
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
            // Only process single right-clicks (not drag releases)
            // Check both current state and if we were dragging this frame
            if !drag_state.is_right_dragging && !drag_state.was_dragging_this_frame {
                if let Some(slot) = find_slot_under_cursor(cursor_pos, &slot_query) {
                    if let Some(container) = container_manager.get_container_mut(&slot.container_type) {
                        process_right_click(slot.index, container, &mut held_item);
                    }
                }
            }
        }
    }
}