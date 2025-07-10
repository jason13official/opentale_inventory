use bevy::prelude::*;

pub const SLOT_COUNT: usize = 9;
pub const SLOT_SIZE: f32 = 60.0;
pub const SLOT_MARGIN: f32 = 5.0;

/// Used to mark a UI button as an inventory slot.
#[derive(Component)]
pub struct InventorySlot {
    pub index: usize,
}

/// Used to mark UI elements that follow the cursor when an item is held by the cursor
#[derive(Component)]
pub struct HeldItemDisplay;

/// Track item held by the player's cursor
#[derive(Resource, Default)]
pub struct HeldItem {
    pub stack: Option<ItemStack>,
}

/// ItemStack -> a stack of items with a size limit that can be stored in inventories
#[derive(Clone, Debug)]
pub struct ItemStack {
    pub name: String,
    pub size: u32,
    pub max_stack_size: u32,
}

/// ItemStack implementation
impl ItemStack {

    // "constructor"; I know this isn't Java,
    // but I don't know what else to call this
    // TODO: construct ItemStacks from registered Items, default to stack size 1 unless set otherwise.
    pub fn new(name: &str, size: u32, max_stack_size: u32) -> Self {
        Self {
            name: name.to_string(),
            size,
            max_stack_size, // shorthand for self_variable = parameter_variable
        }
    }

    /// Same name same max size? maybe make this more robust.
    ///
    /// maybe check item IDs, types, enchantments, etc.
    pub fn can_merge_with(&self, other: &ItemStack) -> bool {
        self.name == other.name && self.max_stack_size == other.max_stack_size
    }

    /// Try to dump held items onto another item stack
    ///
    /// `self` would be the cursor-held item stack in some instances.
    ///
    /// This method returns `true` if merging onto the item stack was successful,
    /// or `false` if some items were left on `self`.
    pub fn try_merge(&mut self, compared_item_stack: &mut ItemStack) -> bool {

        // are the stacks even similar?
        if !self.can_merge_with(compared_item_stack) {
            return false;
        }

        // how many total
        let combined_stack_size = self.size + compared_item_stack.size;

        // can we can fit everything into the target stack?
        if combined_stack_size <= compared_item_stack.max_stack_size {

            // dump everything into the other stack
            compared_item_stack.size = combined_stack_size;
            self.size = 0;
            true // success
        }
        else {

            // other stack can't take all the items

            // how many items don't fit
            let overflow: u32 = combined_stack_size - compared_item_stack.max_stack_size;

            // dump as many as possible
            compared_item_stack.size = compared_item_stack.max_stack_size;

            // keep the leftovers
            self.size = overflow;

            false // partial success
        }
    }

    /// Splits this stack in half, returning the split portion
    ///
    /// math is hard, but we're just dividing by 2 so don't stress
    pub fn split_half(&mut self) -> Option<ItemStack> {

        if self.size <= 1 {
            return None;
        }

        let half: u32 = self.size / 2;
        self.size -= half;

        Some(ItemStack {
            name: self.name.clone(), // same name bc it's literally the same thing
            size: half,
            max_stack_size: self.max_stack_size,
        })
    }

    /// Checks if this stack is empty (count = 0)
    #[allow(dead_code)] // this should get used... eventually?
    pub fn is_empty(&self) -> bool {
        self.size == 0 // revolutionary logic: if size is 0, it's empty.
    }
}

/// Holds an array of optional item stacks.
/// This is THE INVENTORY.
/// (literally just an array of "maybe there's an item here, maybe there isn't")
/// wrapped it in a struct bc that's what fancy developers do.
#[derive(Resource)]
pub struct Inventory {
    // fixed-size array bc dynamic lists are scary
    slots: [Option<ItemStack>; SLOT_COUNT],
}

impl Default for Inventory {

    // just like real life, we start with nothing. (most of us, anyway)
    fn default() -> Self {
        Self {
            // from_fn<T, const N: usize, F>(f: F) -> [T; N]
            // from_fn documentation: Creates an array where each element is produced by calling f with that element's index while walking forward through the array.

            // all we're doing is filling an array with None values
            slots: std::array::from_fn(|_| None),
        }
    }
}

impl Inventory {

    /// Gets a reference to the item in a specific slot
    /// you can look but don't touch!
    pub fn get_slot(&self, index: usize) -> Option<&ItemStack> {

        // double-optional
        // does this slot exist?
        // is there an item in this slot?

        self.slots.get(index)?.as_ref()
    }

    /// Gets a mutable reference to the item in a specific slot
    /// window shopping with a crowbar
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut ItemStack> {

        // double-optional
        // same thing as above basically
        // with great power, comes great responsibility

        self.slots.get_mut(index)?.as_mut()
    }

    /// Sets the contents of a specific slot
    /// Overrides the original content, use with caution.
    pub fn set_slot(&mut self, index: usize, stack: Option<ItemStack>) {

        // does the slot exist? if so, RIP to its original contents
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = stack;
        }
        else {
            std::panic!("Out-of-bounds index! Attempted to get index {} but inventory had max size of {} (max index of {})", index, self.slots.len(), self.slots.len() - 1)
        }
    }

    /// Removes and returns the item from a specific slot
    pub fn take_slot(&mut self, index: usize) -> Option<ItemStack> {
        // .take() replaces the value with None, and gives use the old value
        self.slots.get_mut(index)?.take()
    }
}