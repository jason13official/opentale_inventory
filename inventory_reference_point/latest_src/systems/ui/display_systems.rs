use crate::world::inventory::components::{HeldItem, HeldItemDisplay, SelectedHotbarSlot, SelectedItemDisplay, InventorySlot, DragState};
use crate::world::inventory::containers::{ContainerManager, ContainerType};
use crate::world::inventory::systems::format_item_display;
use crate::systems::ui::slot_utils::clear_text;
use crate::systems::ui::drag_visuals::calculate_remaining_after_drag;
use bevy::hierarchy::Children;
use bevy::prelude::{Changed, Color, Query, Res, Style, Text, Val, Window, With};
use bevy::ui::{BorderColor, Interaction};

/// Updates the held item display that follows the cursor
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

/// Updates hover effects for inventory slots (currently unused but kept for future use)
#[allow(dead_code)]
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

/// Updates the selected item display showing the currently selected hotbar item
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