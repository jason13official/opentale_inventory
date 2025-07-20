use crate::world::item::item::*;
use crate::define_items;

define_items! {
    BOW => "bow" as "Bow" @ (0, 2): ItemProperties::new().durability(120).max_stack_size(1),
    IRON_SWORD => "iron_sword" as "Iron Sword" @ (0, 0): ItemProperties::new().durability(120).max_stack_size(1), // todo durability implies stack size 1
    APPLE => "apple" as "Apple" @ (0, 8): ItemProperties::new().consumable(true),
    GLASS_BOTTLE => "glass_bottle" as "Glass Bottle" @ (7, 3): ItemProperties::new().max_stack_size(16).offhand_equipable(true), // todo no need to specify true when calling implies the same thing
    CHEESE => "cheese" as "Cheese" @ (1, 8): ItemProperties::new().consumable(true),
    RING => "ring" as "Ring" @ (0, 5): ItemProperties::new(),
}