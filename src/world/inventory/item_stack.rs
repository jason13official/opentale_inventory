use crate::world::item::item::{Item, ItemProperties};

/// ItemStack -> a stack of items with a size limit that can be stored in inventories
#[derive(Debug, Copy, Clone)]
pub struct ItemStack {
    pub item: Option<Item>,
    pub size: u32
}

/// ItemStack implementation
impl ItemStack {

    pub fn new(item: Item, size: u32) -> Self {
        Self {
            item: Some(item),
            size,
        }
    }

    pub const fn empty() -> Self {
        Self {
            item: None,
            size: 0
        }
    }

    /// Same name same max size? maybe make this more robust.
    ///
    /// maybe check item IDs, types, enchantments, etc.
    pub fn can_merge_with(&self, other: &ItemStack) -> bool {
        let self_properties: &ItemProperties = &self.item.unwrap().properties;
        let other_properties: &ItemProperties = &other.item.unwrap().properties;
        self.item == other.item && self.item.unwrap().identifier == other.item.unwrap().identifier && self_properties.max_stack_size == other_properties.max_stack_size
    }

    /// Try to dump held items onto another item stack
    ///
    /// `self` would be the cursor-held item stack in some instances.
    ///
    /// This method returns `true` if merging onto the item stack was successful,
    /// or `false` if some items were left on `self`.
    pub fn try_merge(&mut self, other_stack: &mut ItemStack) -> bool {

        // are the stacks even similar?
        if !self.can_merge_with(other_stack) {
            return false;
        }

        // how many total
        let combined_stack_size = self.size + other_stack.size;

        // can we can fit everything into the target stack?
        if combined_stack_size <= other_stack.item.unwrap().properties.max_stack_size {

            // dump everything into the other stack
            other_stack.size = combined_stack_size;
            self.size = 0;
            true // success
        }
        else {

            // our combined_stack_size is greater than the max size of other_stack

            // how much many more items can the stack take?
            let available_space = other_stack.item.unwrap().properties.max_stack_size - other_stack.size;

            // how many will we transfer? (the smallest amount, either our own size or available space)
            let transferred_amount = self.size.min(available_space);

            // add to the size as many items as we transferred to the other stack
            other_stack.size += transferred_amount;

            // equally remove the same amount from ourselves
            self.size -= transferred_amount;

            false
        }
    }

    /// Splits this stack in half, returning the split portion
    pub fn split_half(&mut self) -> Option<ItemStack> {

        if self.size <= 1 {
            return None;
        }

        let half: u32 = self.size / 2;
        self.size -= half;

        Some(ItemStack {
            item: self.item.clone(), // same name bc it's literally the same thing
            size: half
        })
    }

    /// Checks if this stack is empty (count = 0)
    #[allow(dead_code)] // this should get used... eventually?
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}