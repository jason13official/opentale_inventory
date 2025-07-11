use opentale_inventory::*;
use crate::world::item::{item::*};

define_items! {
    DIAMOND => "diamond" as "Diamond": ItemProperties::new(),
    IRON_SWORD => "iron_sword" as "Iron Sword": ItemProperties::new().durability(120).max_stack_size(1), // todo durability implies stack size 1
    APPLE => "apple" as "Apple": ItemProperties::new().consumable(true),
    GLASS_BOTTLE => "glass_bottle" as "Glass Bottle": ItemProperties::new().max_stack_size(16).offhand_equipable(true), // todo no need to specify true when calling implies the same thing
    BREAD => "bread" as "Bread": ItemProperties::new().consumable(true),
}