extern crate rson_rs as rson;

use rson::de::{from_str, from_reader};
use rson::ser::pretty::to_string;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write, BufReader};
use std::path::Path;

/// A container state that manages an inventory system with named slots.
/// 
/// `ContainerState` represents a fixed-size inventory container that can hold
/// items in specific slots. Each slot can either be empty (`None`) or contain
/// an item represented as a `String`.

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerState {
    /// The name identifier for this container
    container_name: String,
    /// Total number of slots available in this container
    slot_count: usize,
    /// The inventory storage where None = empty slot, Some(item) = occupied slot
    inventory: Vec<Option<String>>,
}

impl ContainerState {
    /// Creates a new empty container with the specified name and slot count.
    /// 
    /// All slots are initialized as empty (`None`).
    pub fn new(name: String, slot_count: usize) -> Self {
        ContainerState {
            container_name: name,
            slot_count,
            inventory: vec![None; slot_count],
        }
    }

    /// Adds an item to the first available empty slot.
    /// 
    /// Searches through the inventory from index 0 and places the item
    /// in the first slot that is `None`.
    pub fn add_item(&mut self, item: String) -> Result<usize, String> {
        for (index, slot) in self.inventory.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(item);
                return Ok(index);
            }
        }
        Err("Container is full".to_string())
    }

    /// Gets a reference to an item in a specific slot without removing it.
    /// 
    /// This method provides read-only access to the item at the specified slot.
    pub fn peek_slot(&self, slot_index: usize) -> Option<&String> {
        if slot_index < self.inventory.len() {
            self.inventory.get(slot_index)?.as_ref()
        } else {
            panic!("Attempted to peek an item in slot index {} from a container with a max index of {}", slot_index, self.slot_count - 1)
        }
    }

    /// Creates a human-readable description of what's in a specific slot.
    pub fn inspect_slot(&self, slot: usize) -> String {
        match self.peek_slot(slot) {
            Some(item_ref) => format!("Slot {} contains: {}", slot, item_ref),
            None => format!("Slot {} is empty", slot),
        }
    }

    /// Removes and returns an item from a specific slot.
    /// 
    /// This method takes ownership of the item at the specified slot,
    /// leaving the slot empty (`None`).
    pub fn retrieve_slot_stack(&mut self, slot_index: usize) -> Option<String> {
        if slot_index < self.inventory.len() {
            self.inventory[slot_index].take()
        } else {
            panic!("Attempted to retrieve an item in slot index {} from a container with a max index of {}", slot_index, self.slot_count - 1)
        }
    }

    /// Sets the content of a specific slot to the provided item.
    /// 
    /// This method directly sets the slot's content, replacing whatever
    /// was there before (if anything).
    pub fn set_item(&mut self, slot_index: usize, item: Option<String>) {
        if slot_index < self.inventory.len() {
            self.inventory[slot_index] = item;
        } else {
            panic!("Attempted to set an item into slot index {} of a container with a max index of {}", slot_index, self.slot_count - 1)
        }
    }

    /// Removes an item from a specific slot, making it empty.
    /// 
    /// This is a convenience method that calls `set_item` with `None`.
    pub fn remove_item(&mut self, slot_index: usize) {
        if slot_index < self.inventory.len() {
            self.set_item(slot_index, None)
        } else {
            panic!("Attempted to remove an item from slot index {} in a container with a max index of {}", slot_index, self.slot_count - 1)
        }
    }

    /// Returns the number of empty slots in the container.
    /// 
    /// Counts all slots that contain `None`.
    pub fn empty_slots(&self) -> usize {
        self.inventory.iter().filter(|slot| slot.is_none()).count()
    }

    /// Returns the number of occupied slots in the container.
    /// 
    /// Counts all slots that contain `Some(item)`.
    pub fn occupied_slots(&self) -> usize {
        self.inventory.iter().filter(|slot| slot.is_some()).count()
    }

    /// Validates the internal consistency of the container state.
    /// 
    /// Checks that:
    /// - Inventory size matches the declared slot count
    /// - Container name is not empty or whitespace-only
    /// - Container has at least one slot
    pub fn validate(&self) -> Result<(), String> {
        if self.inventory.len() != self.slot_count {
            return Err(format!(
                "Inventory size ({}) doesn't match slot_count ({})",
                self.inventory.len(),
                self.slot_count
            ));
        }

        if self.container_name.trim().is_empty() {
            return Err("Container name cannot be empty".to_string());
        }

        if self.slot_count == 0 {
            return Err("Container must have at least one slot".to_string());
        }

        Ok(())
    }

    /// Moves an item from one slot to another within the container.
    /// 
    /// This operation will fail if:
    /// - The source slot is empty
    /// - The destination slot is already occupied
    /// - Either slot index is out of bounds
    pub fn move_item(&mut self, from_slot: usize, to_slot: usize) -> Result<(), String> {
        // Check if destination slot is empty
        let has_space: bool = self.peek_slot(to_slot) == None;

        // Attempt to retrieve item from source slot and move it to destination
        if let Some(item) = self.retrieve_slot_stack(from_slot) {
            if has_space { 
                self.set_item(to_slot, Some(item));
                Ok(())
            } else {
                // Put the item back in the original slot if destination is occupied
                self.set_item(from_slot, Some(item));
                Err("Destination slot is already occupied".to_string())
            }
        } else {
            Err("Source slot is empty - no item to move".to_string())
        }
    }

    /// Returns the total number of items currently in the container.
    /// 
    /// This is equivalent to `occupied_slots()` but with a more descriptive name
    /// for contexts where you're counting items rather than slots.
    pub fn count_items(&self) -> usize {
        self.inventory.iter()
            .filter(|slot| slot.is_some())
            .count()
    }

    /// Gets the name of this container.
    pub fn name(&self) -> &str {
        &self.container_name
    }

    /// Gets the total slot capacity of this container.
    pub fn size(&self) -> usize {
        self.slot_count
    }

    /// Checks if the container is completely full.
    pub fn is_full(&self) -> bool {
        self.empty_slots() == 0
    }

    /// Checks if the container is completely empty.
    pub fn is_empty(&self) -> bool {
        self.occupied_slots() == 0
    }

    /// Saves the container state to an RSON file.
    /// 
    /// Serializes the entire container state (including name, slot count, and all items)
    /// to the RSON format and writes it to the specified file path.
    /// 
    /// # Arguments
    /// 
    /// * `file_path` - The path where the RSON file should be saved
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the save was successful
    /// * `Err(String)` - Error message if serialization or file writing failed
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let mut container = ContainerState::new("player_inventory".to_string(), 10);
    /// container.add_item("sword".to_string()).unwrap();
    /// container.add_item("potion".to_string()).unwrap();
    /// 
    /// match container.save_to_file("inventory.rson") {
    ///     Ok(()) => println!("Container saved successfully"),
    ///     Err(e) => println!("Failed to save: {}", e),
    /// }
    /// ```
    /// 
    /// # RSON Format Example
    /// 
    /// The saved file will look like:
    /// ```rson
    /// {
    ///     container_name: "player_inventory",
    ///     slot_count: 10,
    ///     inventory: [
    ///         Some("sword"),
    ///         Some("potion"), 
    ///         None,
    ///         None,
    ///         // ... remaining slots
    ///     ],
    /// }
    /// ```
    pub fn save_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), String> {
        // Serialize to RSON string
        let rson_string = to_string(self)
            .map_err(|e| format!("Failed to serialize container to RSON: {}", e))?;
        
        // Create/open file for writing
        let mut file = File::create(file_path.as_ref())
            .map_err(|e| format!("Failed to create file '{}': {}", file_path.as_ref().display(), e))?;
        
        // Write RSON data to file
        file.write_all(rson_string.as_bytes())
            .map_err(|e| format!("Failed to write data to file: {}", e))?;
        
        Ok(())
    }

    /// Loads a container state from an RSON file.
    /// 
    /// Deserializes a container state from an RSON file, reconstructing the
    /// complete container including name, slot count, and all items.
    /// 
    /// # Arguments
    /// 
    /// * `file_path` - The path to the RSON file to load
    /// 
    /// # Returns
    /// 
    /// * `Ok(ContainerState)` - The loaded container state
    /// * `Err(String)` - Error message if file reading or deserialization failed
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// match ContainerState::load_from_file("inventory.rson") {
    ///     Ok(container) => {
    ///         println!("Loaded container '{}' with {} items", 
    ///                  container.name(), container.count_items());
    ///     },
    ///     Err(e) => println!("Failed to load: {}", e),
    /// }
    /// ```
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<ContainerState, String> {
        // Open file for reading
        let file = File::open(file_path.as_ref())
            .map_err(|e| format!("Failed to open file '{}': {}", file_path.as_ref().display(), e))?;
        
        let reader = BufReader::new(file);
        
        // Deserialize from RSON
        let container: ContainerState = from_reader(reader)
            .map_err(|e| format!("Failed to deserialize RSON data: {}", e))?;
        
        // Validate the loaded container
        container.validate()
            .map_err(|e| format!("Loaded container failed validation: {}", e))?;
        
        Ok(container)
    }

    /// Saves the container state to an RSON string.
    /// 
    /// Serializes the container to RSON format and returns it as a string,
    /// without writing to a file.
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The RSON representation of the container
    /// * `Err(String)` - Error message if serialization failed
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let mut container = ContainerState::new("temp_storage".to_string(), 3);
    /// container.add_item("apple".to_string()).unwrap();
    /// 
    /// let rson_data = container.to_rson_string().unwrap();
    /// println!("Container as RSON:\n{}", rson_data);
    /// ```
    pub fn to_rson_string(&self) -> Result<String, String> {
        to_string(self)
            .map_err(|e| format!("Failed to serialize container to RSON: {}", e))
    }

    /// Loads a container state from an RSON string.
    /// 
    /// Deserializes a container state from an RSON-formatted string.
    /// 
    /// # Arguments
    /// 
    /// * `rson_data` - The RSON string to deserialize
    /// 
    /// # Returns
    /// 
    /// * `Ok(ContainerState)` - The loaded container state
    /// * `Err(String)` - Error message if deserialization failed
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let rson_data = r#"{
    ///     container_name: "test_container",
    ///     slot_count: 5,
    ///     inventory: [Some("item1"), None, Some("item2"), None, None],
    /// }"#;
    /// 
    /// match ContainerState::from_rson_string(rson_data) {
    ///     Ok(container) => println!("Loaded container with {} items", container.count_items()),
    ///     Err(e) => println!("Failed to parse: {}", e),
    /// }
    /// ```
    pub fn from_rson_string(rson_data: &str) -> Result<ContainerState, String> {
        let container: ContainerState = from_str(rson_data)
            .map_err(|e| format!("Failed to deserialize RSON data: {}", e))?;
        
        // Validate the loaded container
        container.validate()
            .map_err(|e| format!("Loaded container failed validation: {}", e))?;
        
        Ok(container)
    }
}