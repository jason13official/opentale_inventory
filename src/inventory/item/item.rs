use crate::inventory::item::food_properties::FoodProperties;
use crate::inventory::item::rarity::Rarity;

#[derive(Debug, Clone)]
pub struct ItemProperties {
    /// Unsigned as stack size cannot be negative.
    /// `u32` to match Minecraft using `int`, but
    /// we range of 0 to 4,294,967,295 in rust instead of -2,147,483,648 to 2,147,483,647 in Java
    max_stack_size: u32,

    /// Unsigned as damage cannot be negative. Can be 0 in some cases: Items that don't take damage,
    /// and items that have a durability and do take damage.
    max_damage: u32,

    /// Useful for buckets, bottles, etc.; cases where using the item in a recipe should return the
    /// base item or another item back.
    crafting_remainder: Option<Item>,

    /// Useful for determining chance to spawn as loot, and used for slot coloring
    rarity: Rarity,

    /// Usually `None` unless the item is explicitly a food item. FoodProperties determines the nutrition
    /// and saturation modifiers of food as well as attached effects/attribute modifiers.
    food_properties: Option<FoodProperties>,

    fire_resistant: bool,
}

impl ItemProperties {

    /// Used via `ItemProperties::new()`, creates default properties for generic items:
    /// - stacks to 64
    /// - no damage from using
    /// - no remainder after crafting (i.e. item is not a bucket)
    /// - the item is common, not uncommon/rare/epic/etc.
    /// - no food properties (the item is not edible)
    pub const fn new() -> Self {
        Self {
            max_stack_size: 64,
            max_damage: 0,
            crafting_remainder: None,
            rarity: Rarity::Common,
            food_properties: None,
            fire_resistant: false,
        }
    }

    /// FoodProperties instances are constructed using a dynamic function, and so cannot be properly
    /// deconstructed as a constant value. We explicitly omit "const", as FoodProperties cannot be
    /// known at compile time, only runtime.
    pub fn food(mut self, food_properties: FoodProperties) -> Self {
        self.food_properties = Some(food_properties);
        self
    }

    pub const fn max_stack_size(mut self, size: u32) -> Self {
        if self.max_damage > 0 { panic!("Unable to have damage AND stack."); }
        self.max_stack_size = size;
        self
    }

    pub const fn max_damage(mut self, amount: u32) -> Self {
        self.max_damage = amount;
        self.max_stack_size = 1;
        self
    }

    pub const fn crafting_remainder(mut self, remainder: Item) -> Self {
        self.crafting_remainder = Some(remainder);
        self
    }

    pub const fn rarity(mut self, rarity: Rarity) -> Self {
        self.rarity = rarity;
        self
    }

    pub const fn fire_resistant(mut self) -> Self {
        self.fire_resistant = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    rarity: Rarity,
    max_stack_size: u32,
    max_damage: u32,
    fire_resistant: bool,
    crafting_remainder: Option<Item>,
    food_properties: Option<FoodProperties>
}

/// Implementation mostly consists of getter methods; Item instances should generally remain
/// immutable.
impl Item {

    pub const fn new(properties: ItemProperties) -> Self {
        Self {
            rarity: properties.rarity,
            max_stack_size: properties.max_stack_size,
            max_damage: properties.max_damage,
            fire_resistant: properties.fire_resistant,
            crafting_remainder: properties.crafting_remainder,
            food_properties: properties.food_properties,
        }
    }

    pub const fn rarity(self) -> Rarity {
        self.rarity
    }

    pub const fn max_stack_size(self) -> u32 {
        self.max_stack_size
    }

    pub const fn max_damage(self) -> u32 {
        self.max_damage
    }

    pub const fn fire_resistant(self) -> bool {
        self.fire_resistant
    }

    pub const fn crafting_remainder(self) -> Option<Item> {
        self.crafting_remainder
    }

    // ownership isn't cheap, only give a reference
    pub fn food_properties(&self) -> Option<&FoodProperties> {
        self.food_properties.as_ref()
    }
}