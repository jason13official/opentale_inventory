use crate::systems::container::UIRebuildNeeded;
use crate::world::inventory::components::{HeldItem, HeldItemDisplay, InventorySlot, SelectedHotbarSlot, SelectedItemDisplay, ItemSprite, ItemCountText, DragState, SlotPreviewText};
use crate::world::inventory::containers::{ContainerManager, ContainerType, ContainerUI};
use crate::world::inventory::systems::format_item_display;
use crate::world::inventory::ui::ItemSpritesheet;
use bevy::hierarchy::Children;
use crate::utils::item_operations::filter_valid_distribution_slots;
use bevy::prelude::{BackgroundColor, Changed, Color, Entity, Query, Res, Style, Text, Val, Window, With, Without, UiImage, Visibility, TextureAtlas};
use bevy::ui::{BorderColor, Interaction};

/// Converts sprite coordinates (x, y) to atlas index for an 8x9 spritesheet
fn sprite_coords_to_atlas_index(sprite_x: u8, sprite_y: u8) -> usize {
    (sprite_y as usize * 8) + sprite_x as usize
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
            // Calculate how much can actually fit in this slot
            if let Some(container) = container_manager.get_container(&current_slot.0) {
                match container.get_slot(current_slot.1) {
                    None => held_stack.size, // Empty slot can take the whole stack
                    Some(existing_stack) => {
                        if held_stack.can_merge_with(existing_stack) {
                            let max_size = existing_stack.item.unwrap().properties.max_stack_size;
                            let available_space = max_size - existing_stack.size;
                            held_stack.size.min(available_space)
                        } else {
                            0 // Can't merge
                        }
                    }
                }
            } else {
                0
            }
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
        if let Some(container) = container_manager.get_container(&current_slot.0) {
            match container.get_slot(current_slot.1) {
                None => intended_amount, // Empty slot can take the intended amount
                Some(existing_stack) => {
                    if held_stack.can_merge_with(existing_stack) {
                        let max_size = existing_stack.item.unwrap().properties.max_stack_size;
                        let available_space = max_size - existing_stack.size;
                        intended_amount.min(available_space)
                    } else {
                        0 // Can't merge
                    }
                }
            }
        } else {
            0
        }
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

            // Find the sprite and text children by traversing the hierarchy
            let mut sprite_entity = None;
            let mut text_entity = None;
            let mut preview_text_entity = None;
            
            // First child should be the container node
            if let Some(&container_child) = children.first() {
                // This container child should have children that are sprite and text
                if let Ok(sprite_children) = children_query.get(container_child) {
                    for &grandchild in sprite_children.iter() {
                        if sprite_query.get(grandchild).is_ok() {
                            sprite_entity = Some(grandchild);
                        }
                        if text_query.get(grandchild).is_ok() {
                            text_entity = Some(grandchild);
                        }
                        if preview_text_query.get(grandchild).is_ok() {
                            preview_text_entity = Some(grandchild);
                        }
                    }
                }
            }

            if let Some(item_stack) = container.get_slot(slot.index) {
                if let Some(item) = item_stack.item {
                    // Update sprite
                    if let Some(sprite_ent) = sprite_entity {
                        if let Ok((mut ui_image, mut texture_atlas, mut visibility)) = sprite_query.get_mut(sprite_ent) {
                            *ui_image = UiImage::new(spritesheet.texture.clone());
                            texture_atlas.layout = spritesheet.texture_atlas.clone();
                            texture_atlas.index = sprite_coords_to_atlas_index(item.sprite_coords.0, item.sprite_coords.1);
                            *visibility = Visibility::Visible;
                        }
                    }

                    // Update count text
                    if let Some(text_ent) = text_entity {
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
                if let Some(sprite_ent) = sprite_entity {
                    if let Ok((_, _, mut visibility)) = sprite_query.get_mut(sprite_ent) {
                        *visibility = Visibility::Hidden;
                    }
                }

                // Clear count text
                if let Some(text_ent) = text_entity {
                    if let Ok(mut text) = text_query.get_mut(text_ent) {
                        text.sections[0].value.clear();
                    }
                }

                *bg_color = Color::rgb(0.4, 0.4, 0.4).into();
            }

            // Determine slot state for drag highlighting
            let current_slot = (slot.container_type.clone(), slot.index);
            let is_left_drag_target = drag_state.left_drag_slots.contains(&current_slot);
            let is_right_drag_target = drag_state.right_drag_slots.contains(&current_slot);
            let is_currently_hovered = drag_state.current_hovered_slot == Some(current_slot.clone());
            let is_left_dragging = drag_state.is_left_dragging;
            let is_right_dragging = drag_state.is_right_dragging;
            let is_dragging = is_left_dragging || is_right_dragging;
            
            // Calculate drag preview if dragging
            if is_dragging && held_item.stack.is_some() {
                let preview_count = if is_left_dragging {
                    calculate_drag_preview(&current_slot, &drag_state, &held_item, &container_manager)
                } else if is_right_dragging && is_right_drag_target {
                    1 // Right-click drag always deposits 1 item per slot
                } else {
                    0
                };
                
                // Update preview text
                if let Some(preview_ent) = preview_text_entity {
                    if let Ok(mut preview_text) = preview_text_query.get_mut(preview_ent) {
                        if preview_count > 0 {
                            preview_text.sections[0].value = format!("+{}", preview_count);
                            preview_text.sections[0].style.color = if is_right_dragging {
                                Color::rgb(0.0, 0.8, 1.0) // Light blue for right-click
                            } else {
                                Color::rgb(0.0, 1.0, 0.0) // Green for left-click
                            };
                        } else {
                            preview_text.sections[0].value.clear();
                        }
                    }
                }
            } else {
                // Clear preview text when not dragging
                if let Some(preview_ent) = preview_text_entity {
                    if let Ok(mut preview_text) = preview_text_query.get_mut(preview_ent) {
                        preview_text.sections[0].value.clear();
                    }
                }
            }

            // Set border color based on state priority
            if is_selected {
                *border_color = Color::rgb(1.0, 1.0, 0.0).into(); // Yellow for selected (highest priority)
            }
            else if is_currently_hovered && is_dragging {
                if is_right_dragging {
                    *border_color = Color::rgb(0.0, 0.8, 1.0).into(); // Light blue for right-click hover
                }
                else {
                    *border_color = Color::rgb(0.0, 1.0, 0.0).into(); // Green for left-click hover
                }
            }
            else if is_left_drag_target && is_left_dragging {
                *border_color = Color::rgb(0.0, 0.8, 0.0).into(); // Darker green for left-drag target slots
            }
            else if is_right_drag_target && is_right_dragging {
                *border_color = Color::rgb(0.0, 0.6, 0.8).into(); // Darker blue for right-drag target slots
            }
            else {
                *border_color = Color::rgb(0.6, 0.6, 0.6).into(); // Default border
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