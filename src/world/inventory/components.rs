use bevy::prelude::*;
use crate::world::inventory::containers::ContainerType;
use crate::world::inventory::item_stack::ItemStack;

pub const SLOT_SIZE: f32 = 60.0;
pub const SLOT_MARGIN: f32 = 5.0;

/// Used to mark a UI button as an inventory slot.
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
    // Right-click drag (existing functionality)
    pub is_right_dragging: bool,
    pub was_right_dragging_this_frame: bool,
    pub right_last_hovered_slot: Option<(ContainerType, usize)>,

    // Left-click drag (enhanced)
    pub is_left_dragging: bool,
    pub was_left_dragging_this_frame: bool,
    pub left_drag_slots: Vec<(ContainerType, usize)>,
    pub left_drag_start_amount: u32,

    // Pickup tracking
    pub just_picked_up_item: bool,
    pub pickup_slot: Option<(ContainerType, usize)>,
}

impl DragState {
    pub fn start_left_drag(&mut self, start_amount: u32) {
        self.is_left_dragging = true;
        self.left_drag_slots.clear();
        self.left_drag_start_amount = start_amount;
        self.was_left_dragging_this_frame = false;
    }

    pub fn end_left_drag(&mut self) {
        self.was_left_dragging_this_frame = true;
        self.is_left_dragging = false;
    }

    pub fn add_left_drag_slot(&mut self, container_type: ContainerType, index: usize) {
        let slot = (container_type, index);
        if !self.left_drag_slots.contains(&slot) {
            self.left_drag_slots.push(slot);
        }
    }

    #[allow(unused)] /// unused is BS it's being used
    pub fn reset_left_drag(&mut self) {
        self.left_drag_slots.clear();
        self.left_drag_start_amount = 0;
        self.was_left_dragging_this_frame = false;
        self.is_left_dragging = false;
        // Note: Don't clear pickup info here as it might be needed
    }

    // Pickup tracking methods
    pub fn mark_pickup(&mut self, container_type: ContainerType, index: usize) {
        self.just_picked_up_item = true;
        self.pickup_slot = Some((container_type, index));
    }

    pub fn clear_pickup_flag(&mut self) {
        self.just_picked_up_item = false;
        // Don't clear pickup_slot here - we need it for drag end processing
    }

    pub fn clear_pickup_slot(&mut self) {
        self.pickup_slot = None;
    }

    // Helper methods
    pub fn is_dragging_to_pickup_slot_only(&self) -> bool {
        if let Some(pickup_slot) = &self.pickup_slot {
            self.left_drag_slots.len() == 1 &&
                self.left_drag_slots.first() == Some(pickup_slot)
        } else {
            false
        }
    }

    pub fn get_distribution_slots(&self) -> Vec<(ContainerType, usize)> {
        let mut slots = self.left_drag_slots.clone();

        // Remove pickup slot from distribution
        if let Some(pickup_slot) = &self.pickup_slot {
            slots.retain(|slot| slot != pickup_slot);
        }

        slots
    }
}