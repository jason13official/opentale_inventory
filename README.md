# OpenTale Inventory System

A Minecraft-inspired inventory management system built with Bevy 0.13.0 in Rust.

## Overview

**Key Features:**
- Type-safe item definitions using declarative macros
- Multiple container types (player inventory, hotbar, chests)
- Drag-and-drop item manipulation with visual feedback
- Modular ECS architecture

**Tech Stack:** Rust, Bevy 0.13.0, bevy-inspector-egui

## Architecture

```
src/
├── app.rs              # Application setup and system registration
├── systems/            # Game logic systems
│   ├── inventory/      # Drag and input handling
│   └── ui/            # Visual rendering and display
├── utils/              # Utility functions (item operations, slot finding)
└── world/              # Core data structures
    ├── inventory/      # Containers, components, item stacks
    └── item/          # Item definitions and properties
```

## Core Systems

### Items

Items have four properties: `identifier` (internal ID), `display_name`, `properties`, and `sprite_coords` (spritesheet position).

**Item Properties:**
- `max_stack_size: u32` (default: 64)
- `durability: Option<u128>` (None = no durability)
- `is_consumable: bool` (default: false)
- `offhand_equipable: bool` (default: false)

**Definition using macro:**
```rust
define_items! {
    IRON_SWORD => "iron_sword" as "Iron Sword" @ (0, 0): 
        ItemProperties::new().durability(120).max_stack_size(1),
    APPLE => "apple" as "Apple" @ (0, 8): 
        ItemProperties::new().consumable(true),
}
```

### Containers

**Container Types:**
- `PlayerInventory` (27 slots)
- `Hotbar` (9 slots)
- `Chest(u32)` (27 slots, with unique ID)

**UI Modes:**
- `HotbarOnly` (default)
- `InventoryOpen` (hotbar + player inventory)
- `ChestOpen(u32)` (all containers + specific chest)

The `ContainerManager` resource handles UI mode switching, layout positioning, and dynamic chest creation.

### Item Stacks

**Key Methods:**
- `new(item, size)` - Create stack (respects max_stack_size)
- `can_merge_with(other)` - Check compatibility
- `try_merge(other)` - Attempt merge, returns success
- `split_half()` - Split stack in half

**Merge Requirements:** Same item type, same max stack size, combined size ≤ maximum.

### Drag & Drop

**Two modes:**
- **Left-click drag:** Even distribution across multiple slots
- **Right-click drag:** Single item per slot

**Visual feedback:** Real-time highlighting of valid drop targets during drag operations.

## API Reference

### Key Components

- `InventorySlot` - Marks UI elements as inventory slots with index and container type
- `HeldItem` - Resource tracking cursor-held item stack
- `DragState` - Resource managing drag operations and target slots

### Events

- `OpenInventoryEvent`, `CloseInventoryEvent`
- `OpenChestEvent{chest_id}`, `CloseChestEvent`, `SwitchChestEvent{chest_id}`

## Usage Examples

### Adding Items
Add to `src/world/item/items.rs` and update spritesheet:
```rust
define_items! {
    BOW => "bow" as "Bow" @ (0, 2):
        ItemProperties::new().durability(120).max_stack_size(1),
}
```

### Custom Containers
1. Extend `ContainerType` enum
2. Add `ContainerLayout` method
3. Register in `ContainerManager`

## Getting Started

**Prerequisites:** Rust 1.70+

**Run:** `cargo run`

**Controls:**
- **E** - Toggle inventory
- **C** - Open/close chest
- **1-9** - Select hotbar slots
- **Left-click** - Pick up/place items
- **Right-click** - Pick up/place single items
- **Left/Right-click + Drag** - Distribute items across slots
- **F11** - Toggle fullscreen
- **Escape** - Exit (when only hotbar visible)

**File Structure:**
- `src/main.rs` - Entry point
- `src/app.rs` - System setup
- `src/world/item/items.rs` - Item definitions
- `src/world/inventory/containers.rs` - Container management
- `src/systems/` - Game logic