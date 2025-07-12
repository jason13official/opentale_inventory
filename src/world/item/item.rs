use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ItemProperties {
    pub max_stack_size: u32,
    pub durability: Option<u128>,
    pub is_consumable: bool,
    pub offhand_equipable: bool
}

impl ItemProperties {

    pub const fn new() -> Self {
        Self {
            max_stack_size: 64,
            durability: None,
            is_consumable: false,
            offhand_equipable: false
        }
    }

    pub const fn max_stack_size(mut self, size: u32) -> Self {
        self.max_stack_size = size;
        self
    }

    pub const fn durability(mut self, durability: u128) -> Self {
        self.durability = Some(durability);
        self
    }

    pub const fn consumable(mut self, consumable: bool) -> Self {
        self.is_consumable = consumable;
        self
    }

    pub const fn offhand_equipable(mut self, offhand_equipable: bool) -> Self {
        self.offhand_equipable = offhand_equipable;
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Item {
    /// The Item's internal ID used for lookup
    pub identifier: &'static str,

    /// The Item's human-readable name used for display
    pub display_name: &'static str,

    /// The Item's underlying properties, such as maximum stack size, initial durability, etc.
    pub properties: ItemProperties
}

impl Item {
    pub fn new(identifier: &'static str, display_name: &'static str, properties: ItemProperties) -> Self {
        Self {
            identifier,
            display_name,
            properties
        }
    }
}