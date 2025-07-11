use bevy::prelude::*;
use bevy::input::mouse::MouseButtonInput;
use super::components::*;

/// handle left-clicking on inventory slots
pub fn handle_left_clicks(
    // query to check for buttons marked as InventorySlot and have been interacted with
    mut interaction_query: Query<(&Interaction, &InventorySlot), (Changed<Interaction>, With<Button>)>,
    mut inventory: ResMut<Inventory>,
    mut held_item: ResMut<HeldItem>,
) {

    // loop through our interaction query (interaction_query might return multiple interactions if buttons overlap)
    for (interaction, slot) in &mut interaction_query {

        // if our interaction was clicking, process the click
        if *interaction == Interaction::Pressed {
            process_left_click(slot.index, &mut inventory, &mut held_item);
        }
    }
}

/// handle right-clicking on inventory slots, using manual cursor detection
pub fn handle_right_clicks(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut inventory: ResMut<Inventory>,
    mut held_item: ResMut<HeldItem>,
    // query to get all inventory slots and their positions
    slot_query: Query<(&InventorySlot, &GlobalTransform, &Node)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    for event in mouse_events.read() {

        // are we clicking with the right mouse button?
        if event.button == MouseButton::Right && event.state.is_pressed() {

            // if we're clicking on an inventory slot, process the click
            if let Some(slot_index) = find_slot_under_cursor(cursor_pos, &slot_query) {
                process_right_click(slot_index, &mut inventory, &mut held_item);
            }
        }
    }
}

/// Update text and background color properties on slot
pub fn update_slot_visuals(
    inventory: Res<Inventory>, // read-only.
    mut slot_query: Query<(&InventorySlot, &mut Children, &mut BackgroundColor)>, // all inventory slots and their text element children
    mut text_query: Query<&mut Text>, // all the text elements we're aware of
) {

    // iterate over and update every slot
    for (slot, children, mut bg_color) in &mut slot_query {

        // make sure the slot actually has an item
        if let Some(item) = inventory.get_slot(slot.index) {

            // find the text element belonging to the slot (hard coded to 0 for now) TODO: add array of text elements to display? tooltips?
            if let Some(text_entity) = children.first() {
                if let Ok(mut text) = text_query.get_mut(*text_entity) {

                    // update the text to show what item this is and how many we have
                    text.sections[0].value = format_item_display(item);
                }
            }

            // Make the slot look "occupied" by changing its color
            // Blue-ish because blue means "has stuff" apparently
            // Don't ask me why, I didn't make the rules
            *bg_color = Color::rgb(0.3, 0.3, 0.7).into();
        }
        else {

            // Clear text for empty slots

            if let Some(text_entity) = children.first() {
                if let Ok(mut text) = text_query.get_mut(*text_entity) {
                    text.sections[0].value.clear();
                }
            }

            // Make the slot look "empty" with a boring gray color
            *bg_color = Color::rgb(0.4, 0.4, 0.4).into();
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
fn find_slot_under_cursor(
    cursor_pos: Vec2,
    // query for all of our inventory slots
    slot_query: &Query<(&InventorySlot, &GlobalTransform, &Node)>,
) -> Option<usize> {

    // if we had thousands of slots we would probably do something more efficient here,
    // but typically we'll have under 100 per menu and pagination would help

    for (slot, transform, node) in slot_query {

        // get the slot's position as a Vec2
        let slot_pos = transform.translation().truncate();

        // get the slot's width and size as a Vec2
        let slot_size = node.size();

        // create a rectangle representing the clickable area
        let slot_rect = bevy::math::Rect::from_center_size(slot_pos, slot_size);

        // is the cursor in the slot? if so then return the slot.
        if slot_rect.contains(cursor_pos) {
            return Some(slot.index);
        }
    }

    // we checked every slot, and the cursor wasn't over any of them.
    None
}



/// Formats an item stack for display (handles singular vs plural)
fn format_item_display(stack: &ItemStack) -> String {
    if stack.size > 1 {
        // multiple items, so we display the name and amount
        format!("{}\n{}", stack.item.display_name, stack.size)
    }
    else {
        // just one item, so only display the name
        stack.item.display_name.to_string()
    }
}

/// Processes a left-click on an inventory slot with 4 different scenarios:
/// 1. Pick up an item from a slot (empty hand + full slot)
/// 2. Put down an item onto a slot (full hand + empty slot)
/// 3. Swap items (full hand + full slot, different items)
/// 4. Merge items (full hand + full slot, same items)
fn process_left_click(slot_index: usize, inventory: &mut Inventory, held_item: &mut HeldItem) {
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
fn process_right_click(slot_index: usize, inventory: &mut Inventory, held_item: &mut HeldItem) {

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
                if slot_stack.size < slot_stack.item.properties.max_stack_size {

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
                    held_stack.item.clone(),
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