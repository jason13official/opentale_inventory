use crate::systems::container::UIRebuildNeeded;
use crate::world::inventory::components::{HeldItem, HeldItemDisplay, InventorySlot, SelectedHotbarSlot, SelectedItemDisplay};
use crate::world::inventory::containers::{ContainerManager, ContainerType, ContainerUI};
use crate::world::inventory::systems::format_item_display;
use bevy::hierarchy::Children;
use bevy::prelude::{BackgroundColor, Changed, Color, Entity, Query, Res, Style, Text, Val, Window, With};
use bevy::ui::{BorderColor, Interaction};

pub fn update_slot_visuals(
    container_manager: Res<ContainerManager>,
    selected_hotbar_slot: Res<SelectedHotbarSlot>,
    mut slot_query: Query<(&InventorySlot, &mut Children, &mut BackgroundColor, &mut BorderColor)>,
    mut text_query: Query<&mut Text>,
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

            if let Some(item) = container.get_slot(slot.index) {
                if let Some(text_entity) = children.first() {
                    if let Ok(mut text) = text_query.get_mut(*text_entity) {
                        text.sections[0].value = format_item_display(item);
                    }
                }
                *bg_color = Color::rgb(0.3, 0.3, 0.7).into();
            }
            else {
                if let Some(text_entity) = children.first() {
                    if let Ok(mut text) = text_query.get_mut(*text_entity) {
                        text.sections[0].value.clear();
                    }
                }
                *bg_color = Color::rgb(0.4, 0.4, 0.4).into();
            }


            if is_selected {
                *border_color = Color::rgb(1.0, 1.0, 0.0).into(); // Yellow for selected
            }
            else {
                *border_color = Color::rgb(0.6, 0.6, 0.6).into(); // Default border // todo should we have colors defined globally somewhere or have it defined by the underlying rendered sprite??
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