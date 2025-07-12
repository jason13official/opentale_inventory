use bevy::prelude::*;
use crate::world::inventory::containers::ContainerType;
use crate::world::inventory::item_stack::ItemStack;
use crate::world::item::item::{Item, ItemProperties};

pub const SLOT_COUNT: usize = 9;
pub const SLOT_SIZE: f32 = 60.0;
pub const SLOT_MARGIN: f32 = 5.0;

/// Used to mark a UI button as an inventory slot.
#[derive(Component)]
pub struct InventorySlot {
    pub index: usize,
    pub container_type: ContainerType,
}

/// Used to mark UI elements that follow the cursor when an item is held by the cursor
#[derive(Component)]
pub struct HeldItemDisplay;

/// Track item held by the player's cursor
#[derive(Resource, Default)]
pub struct HeldItem {
    pub stack: Option<ItemStack>,
}

/// Tracks whether the player is holding down the right mouse-button or not
#[derive(Resource, Default)]
pub struct DragState {
    pub is_right_dragging: bool,
    pub was_dragging_this_frame: bool,  // Track if we were dragging when button was released
    pub last_hovered_slot: Option<(ContainerType, usize)>,
}