use bevy::prelude::*;
use crate::world::inventory::inventory::SlotContainer;

// Define container types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContainerType {
    PlayerInventory,
    Hotbar,
    Chest,
    // Add more as needed: Furnace, CraftingTable, etc.
}

// Container layout configuration
#[derive(Debug, Clone)]
pub struct ContainerLayout {
    pub container_type: ContainerType,
    pub slot_count: usize,
    pub rows: usize,
    pub columns: usize,
    pub title: String,
}

impl ContainerLayout {
    pub fn player_inventory() -> Self {
        Self {
            container_type: ContainerType::PlayerInventory,
            slot_count: 27,
            rows: 3,
            columns: 9,
            title: "Inventory".to_string(),
        }
    }

    pub fn hotbar() -> Self {
        Self {
            container_type: ContainerType::Hotbar,
            slot_count: 9,
            rows: 1,
            columns: 9,
            title: "Hotbar".to_string(),
        }
    }

    pub fn chest() -> Self {
        Self {
            container_type: ContainerType::Chest,
            slot_count: 27,
            rows: 3,
            columns: 9,
            title: "Chest".to_string(),
        }
    }
}

// Resource to manage all containers
#[derive(Resource)]
pub struct ContainerManager {
    pub containers: std::collections::HashMap<ContainerType, SlotContainer>,
    pub active_container: ContainerType,
    pub layout: ContainerLayout,
}

impl Default for ContainerManager {
    fn default() -> Self {
        let mut containers = std::collections::HashMap::new();

        // Create the default containers
        containers.insert(ContainerType::PlayerInventory, SlotContainer::new(27));
        containers.insert(ContainerType::Hotbar, SlotContainer::new(9));

        Self {
            containers,
            active_container: ContainerType::Hotbar,
            layout: ContainerLayout::hotbar(),
        }
    }
}

impl ContainerManager {
    pub fn switch_to_container(&mut self, container_type: ContainerType) {
        self.active_container = container_type.clone();
        self.layout = match container_type {
            ContainerType::PlayerInventory => ContainerLayout::player_inventory(),
            ContainerType::Hotbar => ContainerLayout::hotbar(),
            ContainerType::Chest => ContainerLayout::chest(),
        };
    }

    pub fn get_active_container(&self) -> &SlotContainer {
        self.containers.get(&self.active_container).unwrap()
    }

    pub fn get_active_container_mut(&mut self) -> &mut SlotContainer {
        self.containers.get_mut(&self.active_container).unwrap()
    }

    pub fn open_chest(&mut self) {
        // Create a new chest if it doesn't exist
        if !self.containers.contains_key(&ContainerType::Chest) {
            self.containers.insert(ContainerType::Chest, SlotContainer::new(27));
        }
        self.switch_to_container(ContainerType::Chest);
    }

    pub fn close_chest(&mut self) {
        // Switch back to player inventory when closing chest
        self.switch_to_container(ContainerType::PlayerInventory);
    }
}

// Component to mark UI elements as belonging to a specific container
#[derive(Component)]
pub struct ContainerUI {
    pub container_type: ContainerType,
}

// Events for container switching
#[derive(Event)]
pub struct SwitchContainerEvent {
    pub container_type: ContainerType,
}

#[derive(Event)]
pub struct OpenChestEvent;

#[derive(Event)]
pub struct CloseChestEvent;