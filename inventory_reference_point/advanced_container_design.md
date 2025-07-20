# Advanced ContainerState Design with Item and ItemStack

This document explores how the current simple `ContainerState` implementation could be enhanced using the more sophisticated `Item` and `ItemStack` system from the reference code.

## Current vs. Advanced Implementation

### Current Simple Implementation
- **Storage**: `Vec<Option<String>>` - Simple string-based items
- **Limitations**: No item properties, no stacking logic, no item metadata
- **Use Case**: Basic proof-of-concept inventory system

### Advanced Implementation with Item/ItemStack
- **Storage**: `Vec<Option<ItemStack>>` - Rich item objects with properties
- **Features**: Stack merging, item properties, durability, sprite coordinates
- **Use Case**: Production-ready game inventory system

## Advanced ContainerState Structure

```rust
extern crate rson_rs as rson;

use rson::de::{from_str, from_reader};
use rson::ser::pretty::to_string;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write, BufReader};
use std::path::Path;

// Import the advanced item system
use crate::world::item::item::{Item, ItemProperties};
use crate::world::inventory::item_stack::ItemStack;

/// Advanced container state that manages ItemStack objects with rich item properties.
/// 
/// This version supports:
/// - Item stacking with configurable stack limits
/// - Item properties (durability, consumable flags, etc.)
/// - Smart merging of compatible item stacks
/// - Stack splitting operations
/// - RSON serialization/deserialization
#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedContainerState {
    /// The name identifier for this container
    container_name: String,
    /// Total number of slots available in this container
    slot_count: usize,
    /// The inventory storage where None = empty slot, Some(ItemStack) = occupied slot
    inventory: Vec<Option<ItemStack>>,
}
```

## Key Differences and Advantages

### 1. Rich Item Data
Instead of storing simple strings, each slot can contain an `ItemStack` with:
- **Item definition**: Static item data (identifier, display name, properties)
- **Stack size**: Current quantity in the stack
- **Properties**: Max stack size, durability, consumable status, equipment flags
- **Visual data**: Sprite coordinates for rendering

### 2. Intelligent Stacking
```rust
impl AdvancedContainerState {
    /// Adds an item to the container, attempting to merge with existing stacks first
    pub fn add_item(&mut self, item: Item, quantity: u32) -> Result<Vec<usize>, String> {
        let mut remaining_quantity = quantity;
        let mut affected_slots = Vec::new();
        
        // First pass: try to merge with existing stacks
        for (index, slot) in self.inventory.iter_mut().enumerate() {
            if remaining_quantity == 0 { break; }
            
            if let Some(existing_stack) = slot {
                if existing_stack.item.is_some() && 
                   existing_stack.item.unwrap() == item {
                    
                    let available_space = item.properties.max_stack_size - existing_stack.size;
                    let merge_amount = remaining_quantity.min(available_space);
                    
                    if merge_amount > 0 {
                        existing_stack.size += merge_amount;
                        remaining_quantity -= merge_amount;
                        affected_slots.push(index);
                    }
                }
            }
        }
        
        // Second pass: create new stacks in empty slots
        for (index, slot) in self.inventory.iter_mut().enumerate() {
            if remaining_quantity == 0 { break; }
            
            if slot.is_none() {
                let stack_size = remaining_quantity.min(item.properties.max_stack_size);
                *slot = Some(ItemStack::new(item, stack_size));
                remaining_quantity -= stack_size;
                affected_slots.push(index);
            }
        }
        
        if remaining_quantity > 0 {
            Err(format!("Could not fit {} items. {} items remaining.", quantity - remaining_quantity, remaining_quantity))
        } else {
            Ok(affected_slots)
        }
    }
}
```

### 3. Advanced Stack Operations
```rust
impl AdvancedContainerState {
    /// Attempts to merge two item stacks at different slot positions
    pub fn merge_stacks(&mut self, from_slot: usize, to_slot: usize) -> Result<bool, String> {
        // Validate slot indices
        if from_slot >= self.slot_count || to_slot >= self.slot_count {
            return Err("Invalid slot index".to_string());
        }
        
        if from_slot == to_slot {
            return Err("Cannot merge stack with itself".to_string());
        }
        
        // Get mutable references to both slots
        let (from_stack, to_stack) = {
            if from_slot < to_slot {
                let (left, right) = self.inventory.split_at_mut(to_slot);
                (&mut left[from_slot], &mut right[0])
            } else {
                let (left, right) = self.inventory.split_at_mut(from_slot);
                (&mut right[0], &mut left[to_slot])
            }
        };
        
        match (from_stack.as_mut(), to_stack.as_mut()) {
            (Some(from), Some(to)) => {
                let fully_merged = from.try_merge(to);
                
                // If the source stack is now empty, remove it
                if from.size == 0 {
                    *from_stack = None;
                }
                
                Ok(fully_merged)
            },
            (Some(_), None) => {
                // Move entire stack to empty slot
                *to_stack = from_stack.take();
                Ok(true)
            },
            (None, _) => Err("Source slot is empty".to_string()),
        }
    }
    
    /// Splits a stack at the specified slot, returning half to the cursor/hand
    pub fn split_stack(&mut self, slot_index: usize) -> Result<Option<ItemStack>, String> {
        if slot_index >= self.slot_count {
            return Err("Invalid slot index".to_string());
        }
        
        match self.inventory[slot_index].as_mut() {
            Some(stack) => Ok(stack.split_half()),
            None => Err("Cannot split empty slot".to_string()),
        }
    }
}
```

### 4. Enhanced Inspection and Queries
```rust
impl AdvancedContainerState {
    /// Returns detailed information about a slot's contents
    pub fn inspect_slot_detailed(&self, slot_index: usize) -> String {
        match self.peek_slot(slot_index) {
            Some(stack) => {
                if let Some(item) = stack.item {
                    format!(
                        "Slot {}: {} x{} ({})\n  Max Stack: {}\n  Durability: {:?}\n  Consumable: {}\n  Sprite: {:?}",
                        slot_index,
                        item.display_name,
                        stack.size,
                        item.identifier,
                        item.properties.max_stack_size,
                        item.properties.durability,
                        item.properties.is_consumable,
                        item.sprite_coords
                    )
                } else {
                    format!("Slot {}: Empty stack (corrupted data)", slot_index)
                }
            },
            None => format!("Slot {}: Empty", slot_index),
        }
    }
    
    /// Counts total quantity of a specific item across all stacks
    pub fn count_item_quantity(&self, item_identifier: &str) -> u32 {
        self.inventory.iter()
            .filter_map(|slot| slot.as_ref())
            .filter(|stack| {
                stack.item.map_or(false, |item| item.identifier == item_identifier)
            })
            .map(|stack| stack.size)
            .sum()
    }
    
    /// Finds all slots containing a specific item
    pub fn find_item_slots(&self, item_identifier: &str) -> Vec<usize> {
        self.inventory.iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.as_ref().and_then(|stack| {
                    stack.item.and_then(|item| {
                        if item.identifier == item_identifier {
                            Some(index)
                        } else {
                            None
                        }
                    })
                })
            })
            .collect()
    }
}
```

### 5. RSON Serialization Example

The RSON output would be much richer:

```rson
{
    container_name: "player_backpack",
    slot_count: 27,
    inventory: [
        Some(ItemStack {
            item: Some(Item {
                identifier: "iron_sword",
                display_name: "Iron Sword",
                properties: ItemProperties {
                    max_stack_size: 1,
                    durability: Some(250),
                    is_consumable: false,
                    offhand_equipable: false,
                },
                sprite_coords: (2, 1),
            }),
            size: 1,
        }),
        Some(ItemStack {
            item: Some(Item {
                identifier: "health_potion",
                display_name: "Health Potion",
                properties: ItemProperties {
                    max_stack_size: 16,
                    durability: None,
                    is_consumable: true,
                    offhand_equipable: false,
                },
                sprite_coords: (0, 3),
            }),
            size: 8,
        }),
        None,
        None,
        // ... remaining slots
    ],
}
```

## Migration Strategy

To upgrade from the current simple implementation to the advanced version:

1. **Phase 1**: Create parallel `AdvancedContainerState` struct
2. **Phase 2**: Add conversion methods between simple and advanced formats
3. **Phase 3**: Update game logic to use advanced features gradually
4. **Phase 4**: Deprecate simple implementation once advanced is stable

## Benefits of Advanced Implementation

1. **Game-Ready Features**: Stack limits, item properties, merging logic
2. **Performance**: Efficient operations on large inventories
3. **Extensibility**: Easy to add new item properties and behaviors
4. **Data Integrity**: Type-safe operations with compile-time guarantees
5. **Rich Serialization**: Complete item state preserved in save files
6. **UI Support**: All data needed for inventory rendering and interaction

This advanced implementation provides a solid foundation for a production game inventory system while maintaining the RSON serialization capabilities from the current simple version.