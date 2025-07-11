use bevy::utils::HashMap;
use crate::world::item::item::*;

#[derive(Debug)]
pub struct ItemRegistry {
    map: HashMap<&'static str, &'static Item>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, identifier: &'static str, display_name: &'static str, props: ItemProperties) -> &'static Item {
        let item = Box::leak(Box::new(Item::new(identifier, display_name, props)));
        self.map.insert(identifier, item);
        item
    }

    pub fn get(&self, name: &str) -> Option<&'static Item> {
        self.map.get(name).copied()
    }
}