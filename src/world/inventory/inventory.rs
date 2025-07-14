use crate::world::inventory::item_stack::ItemStack;
use bevy::prelude::Resource;

#[derive(Debug, Clone)]
pub enum InventoryError {
    IndexOutOfBounds { index: usize, max_size: usize },
}

#[derive(Debug, Copy, Clone)]
pub struct Slot {
    pub stack: Option<ItemStack>
}

impl Slot {

    pub fn new(stack: ItemStack) -> Self {
        Self {
            stack: Some(stack)
        }
    }

    /// Returns an empty slot (holding None)
    pub fn empty() -> Slot {
        Slot {
            stack: None
        }
    }

    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.stack.is_none()
    }

    /// yoink!
    pub fn take(&mut self) -> Option<ItemStack> {
        self.stack.take()
    }

    #[allow(unused)]
    pub fn insert(&mut self, stack: ItemStack) -> Option<ItemStack> {
        match &mut self.stack {
            Some(existing) if existing.item == stack.item => {
                let max = existing.item.unwrap().properties.max_stack_size;
                let available = max.saturating_sub(existing.size);
                let to_add = available.min(stack.size);
                existing.size += to_add;

                if to_add == stack.size {
                    None
                }
                else {
                    Some(ItemStack {
                        item: stack.item,
                        size: stack.size - to_add,
                    })
                }
            }
            None => {
                self.stack = Some(stack);
                None
            }
            Some(_) => {
                Some(stack)
            }
        }
    }
}

#[derive(Resource)]
pub struct SlotContainer {
    #[allow(dead_code)] slot_count: usize, // todo maybe remove? maybe use?
    // slots: [Option<ItemStack>; components::SLOT_COUNT],
    slots: Vec<Slot>, // slot containers may have any variable amount of slots, which each hold an optional item stack
}

impl Default for SlotContainer {

    fn default() -> SlotContainer {
        SlotContainer {
            slot_count: 1,
            // slots: Vec::from([Some(ItemStack::new(DIAMOND, 1)); 1]), // slot containers
            slots: Vec::from([Slot::new(ItemStack::empty()); 1]), // slot containers
        }
    }
}

impl SlotContainer {

    pub fn new(slot_count: usize) -> Self {
        Self {
            slot_count,
            // slots: Vec::from([Some(ItemStack::empty()); 1])
            slots: (0..slot_count).map(|_| Slot::empty()).collect()
        }
    }

    /// Gets a reference to the item in a specific slot
    /// "look but don't touch!"
    pub fn get_slot(&self, index: usize) -> Option<&ItemStack> {

        // double-optional
        // does this slot exist?
        // is there an item in this slot?

        self.slots.get(index)?.stack.as_ref()
    }

    /// Gets a mutable reference to the item in a specific slot
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut ItemStack> {

        // double-optional
        // same thing as above basically
        // with great power, comes great responsibility

        self.slots.get_mut(index)?.stack.as_mut()
    }

    /// Sets the contents of a specific slot
    /// Overrides the original content
    pub fn set_slot(&mut self, index: usize, stack: Option<ItemStack>) -> Result<(), InventoryError> {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = Slot { stack };
            Ok(())
        } else {
            Err(InventoryError::IndexOutOfBounds { 
                index, 
                max_size: self.slots.len() 
            })
        }
    }

    /// Sets the contents of a specific slot, panicking on out-of-bounds access
    /// Only use this when you're certain the index is valid
    pub fn set_slot_unchecked(&mut self, index: usize, stack: Option<ItemStack>) {
        self.slots[index] = Slot { stack };
    }

    /// Removes and returns the item from a specific slot
    pub fn take_slot(&mut self, index: usize) -> Option<ItemStack> {
        // .take() replaces the value with None, and gives use the old value
        self.slots.get_mut(index)?.take()
    }

    /// Gets the number of slots in this container
    pub fn len(&self) -> usize {
        self.slots.len()
    }
}