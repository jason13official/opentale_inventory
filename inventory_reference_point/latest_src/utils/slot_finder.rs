use crate::world::inventory::components::InventorySlot;
use bevy::math::Vec2;
use bevy::prelude::{GlobalTransform, Node, Query};

pub fn find_slot_under_cursor<'a>(
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