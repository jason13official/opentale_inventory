use crate::systems::container::UIRebuildNeeded;
use crate::world::inventory::components::{HeldItem, HeldItemDisplay, InventorySlot, SelectedHotbarSlot, SelectedItemDisplay, ItemSprite, ItemCountText};
use crate::world::inventory::containers::{ContainerManager, ContainerType, ContainerUI};
use crate::world::inventory::systems::format_item_display;
use crate::world::inventory::ui::ItemSpritesheet;
use bevy::hierarchy::Children;
use bevy::prelude::{BackgroundColor, Changed, Color, Entity, Query, Res, Style, Text, Val, Window, With, Without, UiImage, Visibility, TextureAtlas};
use bevy::ui::{BorderColor, Interaction};

/// Converts sprite coordinates (x, y) to atlas index for an 8x9 spritesheet
fn sprite_coords_to_atlas_index(sprite_x: u8, sprite_y: u8) -> usize {
    (sprite_y as usize * 8) + sprite_x as usize
}

pub fn update_slot_visuals(
    container_manager: Res<ContainerManager>,
    selected_hotbar_slot: Res<SelectedHotbarSlot>,
    spritesheet: Res<ItemSpritesheet>,
    mut slot_query: Query<(&InventorySlot, &Children, &mut BackgroundColor, &mut BorderColor)>,
    mut sprite_query: Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
    mut text_query: Query<&mut Text, With<ItemCountText>>,
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

            if is_selected {
                *border_color = Color::rgb(1.0, 1.0, 0.0).into(); // Yellow for selected
            } else {
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