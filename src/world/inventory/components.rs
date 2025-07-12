use crate::world::inventory::containers::ContainerType;
use crate::world::inventory::item_stack::ItemStack;
use bevy::prelude::*;

pub const SLOT_SIZE: f32 = 60.0;
pub const SLOT_MARGIN: f32 = 5.0;

#[derive(Component)]
pub struct InventorySlot {
    pub index: usize,
    pub container_type: ContainerType,
}

#[derive(Component)]
pub struct HeldItemDisplay;

#[derive(Resource, Default)]
pub struct HeldItem {
    pub stack: Option<ItemStack>,
}

#[derive(Resource, Default)]
pub struct DragState {
    // Right-click drag (single-deposition)
    pub is_right_dragging: bool,
    pub was_right_dragging_this_frame: bool,
    pub right_last_hovered_slot: Option<(ContainerType, usize)>,

    // Left-click drag (even-distribution)
    pub is_left_dragging: bool,
    pub was_left_dragging_this_frame: bool,
    pub left_drag_slots: Vec<(ContainerType, usize)>,

    pub pickup_slot: Option<(ContainerType, usize)>,
}

impl DragState {
    pub fn add_left_drag_slot(&mut self, container_type: ContainerType, index: usize) {
        let slot = (container_type, index);
        if !self.left_drag_slots.contains(&slot) {
            self.left_drag_slots.push(slot);
        }
    }
}