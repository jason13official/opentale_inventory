use crate::systems::ui::container::UIRebuildNeeded;
use crate::world::inventory::components::{HeldItem, HeldItemDisplay, InventorySlot, SelectedHotbarSlot, SelectedItemDisplay, ItemSprite, ItemCountText, DragState, SlotPreviewText};
use crate::world::inventory::containers::{ContainerManager, ContainerType, ContainerUI};
use crate::world::inventory::systems::format_item_display;
use crate::world::inventory::ui::ItemSpritesheet;
use bevy::hierarchy::Children;
use crate::utils::item_operations::filter_valid_distribution_slots;
use bevy::prelude::{BackgroundColor, Changed, Color, Entity, Query, Res, Style, Text, Val, Window, With, Without, UiImage, Visibility, TextureAtlas};
use bevy::ui::{BorderColor, Interaction};
use crate::world::inventory::item_stack::ItemStack;

/// Converts sprite coordinates (x, y) to atlas index for an 8x9 spritesheet
fn sprite_coords_to_atlas_index(sprite_x: u8, sprite_y: u8) -> usize {
    (sprite_y as usize * 8) + sprite_x as usize
}

/// Check if a slot can accept items from a held stack
fn can_slot_accept_items(
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
fn calculate_available_space(
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
fn clear_text<Q: bevy::ecs::query::QueryFilter>(text_entity: Entity, text_query: &mut Query<&mut Text, Q>) {
    if let Ok(mut text) = text_query.get_mut(text_entity) {
        text.sections[0].value.clear();
    }
}

/// Container for slot child entities to avoid repeated traversal
#[derive(Default)]
struct SlotChildren {
    sprite: Option<Entity>,
    count_text: Option<Entity>,
    preview_text: Option<Entity>,
}

impl SlotChildren {
    fn find_from_slot_children(
        container_child: Entity,
        children_query: &Query<&Children, Without<InventorySlot>>,
        sprite_query: &Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
        text_query: &Query<&mut Text, With<ItemCountText>>,
        preview_text_query: &Query<&mut Text, (With<SlotPreviewText>, Without<ItemCountText>)>,
    ) -> Self {
        let mut children = Self::default();
        
        if let Ok(sprite_children) = children_query.get(container_child) {
            for &grandchild in sprite_children.iter() {
                if sprite_query.get(grandchild).is_ok() {
                    children.sprite = Some(grandchild);
                }
                if text_query.get(grandchild).is_ok() {
                    children.count_text = Some(grandchild);
                }
                if preview_text_query.get(grandchild).is_ok() {
                    children.preview_text = Some(grandchild);
                }
            }
        }
        
        children
    }
}

/// Encapsulates drag state for a specific slot
struct SlotDragContext {
    current_slot: (ContainerType, usize),
    is_left_drag_target: bool,
    is_right_drag_target: bool,
    is_currently_hovered: bool,
    is_left_dragging: bool,
    is_right_dragging: bool,
    show_drag_highlighting: bool,
}

impl SlotDragContext {
    fn new(slot: &InventorySlot, drag_state: &DragState) -> Self {
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
    
    fn is_dragging(&self) -> bool {
        self.is_left_dragging || self.is_right_dragging
    }
}

/// Filters out pickup slot from valid slots if other slots are available
fn filter_out_pickup_slot(
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

/// Determines the appropriate border color for a slot based on its state
fn determine_slot_border_color(
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
fn calculate_drag_preview(
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

pub fn update_slot_visuals(
    container_manager: Res<ContainerManager>,
    selected_hotbar_slot: Res<SelectedHotbarSlot>,
    drag_state: Res<DragState>,
    held_item: Res<HeldItem>,
    spritesheet: Res<ItemSpritesheet>,
    mut slot_query: Query<(&InventorySlot, &Children, &mut BackgroundColor, &mut BorderColor)>,
    mut sprite_query: Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
    mut text_query: Query<&mut Text, With<ItemCountText>>,
    mut preview_text_query: Query<&mut Text, (With<SlotPreviewText>, Without<ItemCountText>)>,
    children_query: Query<&Children, Without<InventorySlot>>,
    ui_query: Query<Entity, With<ContainerUI>>,
    rebuild_query: Query<Entity, With<UIRebuildNeeded>>,
) {
    if !rebuild_query.is_empty() || ui_query.is_empty() {
        return;
    }

    for (slot, children, mut bg_color, mut border_color) in &mut slot_query {
        if let Some(container) = container_manager.get_container(&slot.container_type) {
            // slot belongs to hotbar and matches selected index
            let is_selected = slot.container_type == ContainerType::Hotbar 
                && slot.index == selected_hotbar_slot.slot_index;

            // Find child entities for this slot
            let slot_children = if let Some(&container_child) = children.first() {
                SlotChildren::find_from_slot_children(
                    container_child,
                    &children_query,
                    &sprite_query,
                    &text_query,
                    &preview_text_query,
                )
            } else {
                SlotChildren::default()
            };

            if let Some(item_stack) = container.get_slot(slot.index) {
                if let Some(item) = item_stack.item {
                    // Update sprite
                    if let Some(sprite_ent) = slot_children.sprite {
                        if let Ok((mut ui_image, mut texture_atlas, mut visibility)) = sprite_query.get_mut(sprite_ent) {
                            *ui_image = UiImage::new(spritesheet.texture.clone());
                            texture_atlas.layout = spritesheet.texture_atlas.clone();
                            texture_atlas.index = sprite_coords_to_atlas_index(item.sprite_coords.0, item.sprite_coords.1);
                            *visibility = Visibility::Visible;
                        }
                    }

                    // Update count text
                    if let Some(text_ent) = slot_children.count_text {
                        if let Ok(mut text) = text_query.get_mut(text_ent) {
                            if item_stack.size > 1 {
                                text.sections[0].value = item_stack.size.to_string();
                            } else {
                                text.sections[0].value.clear();
                            }
                        }
                    }
                }

                *bg_color = Color::rgb(0.3, 0.3, 0.7).into();
            } else {
                // Hide sprite when no item
                if let Some(sprite_ent) = slot_children.sprite {
                    if let Ok((_, _, mut visibility)) = sprite_query.get_mut(sprite_ent) {
                        *visibility = Visibility::Hidden;
                    }
                }

                // Clear count text
                if let Some(text_ent) = slot_children.count_text {
                    clear_text(text_ent, &mut text_query);
                }

                *bg_color = Color::rgb(0.4, 0.4, 0.4).into();
            }

            // Create drag context for this slot
            let drag_context = SlotDragContext::new(slot, &drag_state);
            
            // Calculate and display drag preview
            let preview_count = if drag_context.is_dragging() && held_item.stack.is_some() && drag_context.show_drag_highlighting {
                if drag_context.is_left_dragging {
                    calculate_drag_preview(&drag_context.current_slot, &drag_state, &held_item, &container_manager)
                } else if drag_context.is_right_dragging && drag_context.is_right_drag_target {
                    if let Some(held_stack) = &held_item.stack {
                        if can_slot_accept_items(&slot.container_type, slot.index, held_stack, &container_manager) {
                            1
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            };
            
            // Update preview text
            if let Some(preview_ent) = slot_children.preview_text {
                if let Ok(mut preview_text) = preview_text_query.get_mut(preview_ent) {
                    if preview_count > 0 {
                        preview_text.sections[0].value = format!("+{}", preview_count);
                        preview_text.sections[0].style.color = if drag_context.is_right_dragging {
                            Color::rgb(0.0, 0.8, 1.0) // Light blue for right-click
                        } else {
                            Color::rgb(0.0, 1.0, 0.0) // Green for left-click
                        };
                    } else {
                        clear_text(preview_ent, &mut preview_text_query);
                    }
                }
            }
            
            // Set border color based on slot state
            *border_color = determine_slot_border_color(is_selected, &drag_context, preview_count).into();
        }
    }
}

/// Calculate how many items would remain in hand after drag distribution
fn calculate_remaining_after_drag(
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

        // Filter out pickup slot if we have other valid slots
        let filtered_slots = if let Some(pickup_slot) = &drag_state.pickup_slot {
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
        };

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

pub fn update_held_item_display(
    held_item: Res<HeldItem>,
    drag_state: Res<DragState>,
    container_manager: Res<ContainerManager>,
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
                    let is_actively_dragging = (drag_state.is_left_dragging && !drag_state.left_drag_slots.is_empty()) 
                        || (drag_state.is_right_dragging && !drag_state.right_drag_slots.is_empty());
                    
                    let display_count = if is_actively_dragging {
                        let remaining = calculate_remaining_after_drag(stack, &drag_state, &container_manager);
                        // Never show 0 during active dragging - minimum of 1
                        // remaining.max(1)
                        remaining
                    } else {
                        stack.size
                    };
                    
                    if let Some(item) = stack.item {
                        if display_count > 1 {
                            text.sections[0].value = format!("{} ({})", item.display_name, display_count);
                        } else if display_count == 1 {
                            text.sections[0].value = item.display_name.to_string();
                        } else {
                            clear_text(*text_entity, &mut text_query);
                        }
                    } else {
                        clear_text(*text_entity, &mut text_query);
                    }
                } else {
                    clear_text(*text_entity, &mut text_query);
                }
            }
        }
    }
}

#[allow(dead_code)] // todo use? lol
pub fn update_slot_hover_effects(
    mut slot_query: Query<(&InventorySlot, &Interaction, &mut BorderColor), Changed<Interaction>>,
) {
    for (_slot, interaction, mut border_color) in &mut slot_query {
        match *interaction {
            Interaction::Hovered => {
                *border_color = Color::rgb(1.0, 1.0, 1.0).into();
            }
            Interaction::None => {
                *border_color = Color::rgb(0.6, 0.6, 0.6).into();
            }
            Interaction::Pressed => {
                *border_color = Color::rgb(0.8, 0.8, 0.8).into();
            }
        }
    }
}

pub fn update_selected_item_display(
    container_manager: Res<ContainerManager>,
    selected_hotbar_slot: Res<SelectedHotbarSlot>,
    mut display_query: Query<&Children, With<SelectedItemDisplay>>,
    mut text_query: Query<&mut Text>,
) {
    for children in &mut display_query {
        if let Some(text_entity) = children.first() {
            if let Ok(mut text) = text_query.get_mut(*text_entity) {
                if let Some(hotbar) = container_manager.get_container(&ContainerType::Hotbar) {
                    if let Some(item) = hotbar.get_slot(selected_hotbar_slot.slot_index) {
                        text.sections[0].value = format_item_display(item);
                    } else {
                        text.sections[0].value = "Empty".to_string();
                    }
                }
                else {
                    text.sections[0].value = "No Hotbar".to_string();
                }
            }
        }
    }
}