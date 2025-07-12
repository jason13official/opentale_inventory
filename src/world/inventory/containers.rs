use crate::world::inventory::inventory::SlotContainer;
use bevy::prelude::*;

// Define container types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContainerType {
    PlayerInventory,
    Hotbar,
    Chest,
    // Add more as needed: Furnace, CraftingTable, etc.
}

// UI Display modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIMode {
    HotbarOnly,           // Only hotbar visible (default game state)
    InventoryOpen,        // Hotbar + Player inventory visible
    ChestOpen,           // Hotbar + Player inventory + Chest visible
}

// Container layout configuration
#[derive(Debug, Clone)]
pub struct ContainerLayout {
    pub container_type: ContainerType,
    pub slot_count: usize,
    pub rows: usize,
    pub columns: usize,
    pub title: String,
    pub position: ContainerPosition,
}

#[derive(Debug, Clone)]
pub enum ContainerPosition {
    Bottom,     // For hotbar
    Center,     // For player inventory
    Top,        // For chest/other containers
}

impl ContainerLayout {
    pub fn player_inventory() -> Self {
        Self {
            container_type: ContainerType::PlayerInventory,
            slot_count: 27,
            rows: 3,
            columns: 9,
            title: "Inventory".to_string(),
            position: ContainerPosition::Center,
        }
    }

    pub fn hotbar() -> Self {
        Self {
            container_type: ContainerType::Hotbar,
            slot_count: 9,
            rows: 1,
            columns: 9,
            title: "".to_string(), // No title for hotbar
            position: ContainerPosition::Bottom,
        }
    }

    pub fn chest() -> Self {
        Self {
            container_type: ContainerType::Chest,
            slot_count: 27,
            rows: 3,
            columns: 9,
            title: "Chest".to_string(),
            position: ContainerPosition::Top,
        }
    }
}

// Resource to manage all containers
#[derive(Resource)]
pub struct ContainerManager {
    pub containers: std::collections::HashMap<ContainerType, SlotContainer>,
    pub ui_mode: UIMode,
    pub layouts: Vec<ContainerLayout>, // All active layouts to display
}

impl Default for ContainerManager {
    fn default() -> Self {
        let mut containers = std::collections::HashMap::new();

        // Create the default containers
        containers.insert(ContainerType::PlayerInventory, SlotContainer::new(27));
        containers.insert(ContainerType::Hotbar, SlotContainer::new(9));

        Self {
            containers,
            ui_mode: UIMode::HotbarOnly,
            layouts: vec![ContainerLayout::hotbar()], // Start with just hotbar
        }
    }
}

impl ContainerManager {
    pub fn open_inventory(&mut self) {
        self.ui_mode = UIMode::InventoryOpen;
        self.layouts = vec![
            ContainerLayout::player_inventory(),
            ContainerLayout::hotbar(),
        ];
    }

    pub fn close_inventory(&mut self) {
        self.ui_mode = UIMode::HotbarOnly;
        self.layouts = vec![ContainerLayout::hotbar()];
    }

    pub fn open_chest(&mut self) {
        // Create a new chest if it doesn't exist
        if !self.containers.contains_key(&ContainerType::Chest) {
            self.containers.insert(ContainerType::Chest, SlotContainer::new(27));
        }

        self.ui_mode = UIMode::ChestOpen;
        self.layouts = vec![
            ContainerLayout::chest(),
            ContainerLayout::player_inventory(),
            ContainerLayout::hotbar(),
        ];
    }

    pub fn close_chest(&mut self) {
        // Always go back to hotbar when closing chests (like Minecraft)
        self.ui_mode = UIMode::HotbarOnly;
        self.layouts = vec![ContainerLayout::hotbar()];
    }

    pub fn get_container(&self, container_type: &ContainerType) -> Option<&SlotContainer> {
        self.containers.get(container_type)
    }

    pub fn get_container_mut(&mut self, container_type: &ContainerType) -> Option<&mut SlotContainer> {
        self.containers.get_mut(container_type)
    }
}

// Component to mark UI elements as belonging to a specific container
#[derive(Component)]
pub struct ContainerUI {
    #[allow(dead_code)] pub container_type: ContainerType, // todo maybe remove? could be useful? who knows
}

// Events for container switching
#[derive(Event)]
pub struct OpenInventoryEvent;

#[derive(Event)]
pub struct CloseInventoryEvent;

#[derive(Event)]
pub struct OpenChestEvent;

#[derive(Event)]
pub struct CloseChestEvent;