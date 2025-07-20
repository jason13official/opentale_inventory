use crate::systems::ui::container::UIRebuildNeeded;
use crate::world::inventory::components::{HeldItem, InventorySlot, SelectedHotbarSlot, ItemSprite, ItemCountText, DragState, SlotPreviewText};
use crate::world::inventory::containers::{ContainerManager, ContainerType, ContainerUI};
use crate::world::inventory::ui::ItemSpritesheet;
use crate::systems::ui::slot_utils::{sprite_coords_to_atlas_index, clear_text, can_slot_accept_items};
use crate::systems::ui::drag_visuals::{SlotDragContext, determine_slot_border_color, calculate_drag_preview};
use bevy::hierarchy::Children;
use bevy::prelude::{BackgroundColor, Color, Entity, Query, Res, With, Without, UiImage, Visibility, TextureAtlas};
use bevy::ui::BorderColor;

/// Container for slot child entities to avoid repeated traversal
#[derive(Default)]
pub struct SlotChildren {
    pub sprite: Option<Entity>,
    pub count_text: Option<Entity>,
    pub preview_text: Option<Entity>,
}

impl SlotChildren {
    pub fn find_from_slot_children(
        container_child: Entity,
        children_query: &Query<&Children, Without<InventorySlot>>,
        sprite_query: &Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
        text_query: &Query<&mut bevy::prelude::Text, With<ItemCountText>>,
        preview_text_query: &Query<&mut bevy::prelude::Text, (With<SlotPreviewText>, Without<ItemCountText>)>,
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

/// Main system for updating slot visuals including sprites, text, borders, and drag previews
pub fn update_slot_visuals(
    container_manager: Res<ContainerManager>,
    selected_hotbar_slot: Res<SelectedHotbarSlot>,
    drag_state: Res<DragState>,
    held_item: Res<HeldItem>,
    spritesheet: Res<ItemSpritesheet>,
    mut slot_query: Query<(&InventorySlot, &Children, &mut BackgroundColor, &mut BorderColor)>,
    mut sprite_query: Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
    mut text_query: Query<&mut bevy::prelude::Text, With<ItemCountText>>,
    mut preview_text_query: Query<&mut bevy::prelude::Text, (With<SlotPreviewText>, Without<ItemCountText>)>,
    children_query: Query<&Children, Without<InventorySlot>>,
    ui_query: Query<Entity, With<ContainerUI>>,
    rebuild_query: Query<Entity, With<UIRebuildNeeded>>,
) {
    if !rebuild_query.is_empty() || ui_query.is_empty() {
        return;
    }

    for (slot, children, mut bg_color, mut border_color) in &mut slot_query {
        if let Some(container) = container_manager.get_container(&slot.container_type) {
            let is_selected = is_slot_selected(slot, &selected_hotbar_slot);
            let slot_children = find_slot_children(children, &children_query, &sprite_query, &text_query, &preview_text_query);
            
            update_slot_content_visuals(
                slot,
                container,
                &slot_children,
                &spritesheet,
                &mut sprite_query,
                &mut text_query,
                &mut bg_color,
            );

            let drag_context = SlotDragContext::new(slot, &drag_state);
            let preview_count = calculate_slot_preview_count(&drag_context, &held_item, &container_manager, slot);
            
            update_slot_preview_text(
                &slot_children,
                preview_count,
                &drag_context,
                &mut preview_text_query,
            );
            
            *border_color = determine_slot_border_color(is_selected, &drag_context, preview_count).into();
        }
    }
}

fn is_slot_selected(slot: &InventorySlot, selected_hotbar_slot: &SelectedHotbarSlot) -> bool {
    slot.container_type == ContainerType::Hotbar && slot.index == selected_hotbar_slot.slot_index
}

fn find_slot_children(
    children: &Children,
    children_query: &Query<&Children, Without<InventorySlot>>,
    sprite_query: &Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
    text_query: &Query<&mut bevy::prelude::Text, With<ItemCountText>>,
    preview_text_query: &Query<&mut bevy::prelude::Text, (With<SlotPreviewText>, Without<ItemCountText>)>,
) -> SlotChildren {
    if let Some(&container_child) = children.first() {
        SlotChildren::find_from_slot_children(
            container_child,
            children_query,
            sprite_query,
            text_query,
            preview_text_query,
        )
    } else {
        SlotChildren::default()
    }
}

fn update_slot_content_visuals(
    slot: &InventorySlot,
    container: &crate::world::inventory::inventory::SlotContainer,
    slot_children: &SlotChildren,
    spritesheet: &ItemSpritesheet,
    sprite_query: &mut Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
    text_query: &mut Query<&mut bevy::prelude::Text, With<ItemCountText>>,
    bg_color: &mut BackgroundColor,
) {
    if let Some(item_stack) = container.get_slot(slot.index) {
        if let Some(item) = item_stack.item {
            update_slot_sprite(slot_children, item, spritesheet, sprite_query);
            update_slot_count_text(slot_children, item_stack, text_query);
        }
        *bg_color = Color::rgb(0.3, 0.3, 0.7).into();
    } else {
        hide_slot_sprite(slot_children, sprite_query);
        clear_slot_count_text(slot_children, text_query);
        *bg_color = Color::rgb(0.4, 0.4, 0.4).into();
    }
}

fn update_slot_sprite(
    slot_children: &SlotChildren,
    item: crate::world::item::item::Item,
    spritesheet: &ItemSpritesheet,
    sprite_query: &mut Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
) {
    if let Some(sprite_ent) = slot_children.sprite {
        if let Ok((mut ui_image, mut texture_atlas, mut visibility)) = sprite_query.get_mut(sprite_ent) {
            *ui_image = UiImage::new(spritesheet.texture.clone());
            texture_atlas.layout = spritesheet.texture_atlas.clone();
            texture_atlas.index = sprite_coords_to_atlas_index(item.sprite_coords.0, item.sprite_coords.1);
            *visibility = Visibility::Visible;
        }
    }
}

fn update_slot_count_text(
    slot_children: &SlotChildren,
    item_stack: &crate::world::inventory::item_stack::ItemStack,
    text_query: &mut Query<&mut bevy::prelude::Text, With<ItemCountText>>,
) {
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

fn hide_slot_sprite(
    slot_children: &SlotChildren,
    sprite_query: &mut Query<(&mut UiImage, &mut TextureAtlas, &mut Visibility), With<ItemSprite>>,
) {
    if let Some(sprite_ent) = slot_children.sprite {
        if let Ok((_, _, mut visibility)) = sprite_query.get_mut(sprite_ent) {
            *visibility = Visibility::Hidden;
        }
    }
}

fn clear_slot_count_text(
    slot_children: &SlotChildren,
    text_query: &mut Query<&mut bevy::prelude::Text, With<ItemCountText>>,
) {
    if let Some(text_ent) = slot_children.count_text {
        clear_text(text_ent, text_query);
    }
}

fn calculate_slot_preview_count(
    drag_context: &SlotDragContext,
    held_item: &HeldItem,
    container_manager: &ContainerManager,
    slot: &InventorySlot,
) -> u32 {
    if !drag_context.is_dragging() || held_item.stack.is_none() || !drag_context.show_drag_highlighting {
        return 0;
    }

    if drag_context.is_left_dragging {
        calculate_drag_preview(&drag_context.current_slot, &DragState::default(), held_item, container_manager)
    } else if drag_context.is_right_dragging && drag_context.is_right_drag_target {
        if let Some(held_stack) = &held_item.stack {
            if can_slot_accept_items(&slot.container_type, slot.index, held_stack, container_manager) {
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
}

fn update_slot_preview_text(
    slot_children: &SlotChildren,
    preview_count: u32,
    drag_context: &SlotDragContext,
    preview_text_query: &mut Query<&mut bevy::prelude::Text, (With<SlotPreviewText>, Without<ItemCountText>)>,
) {
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
                clear_text(preview_ent, preview_text_query);
            }
        }
    }
}