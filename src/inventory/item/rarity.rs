#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn color(&self) -> &'static str {
        match self {
            Rarity::Common => "white", // todo: replace string literals with InventoryColor enum?
            Rarity::Uncommon => "yellow",
            Rarity::Rare => "aqua",
            Rarity::Epic => "light_purple",
            Rarity::Legendary => "light_blue",
        }
    }
}