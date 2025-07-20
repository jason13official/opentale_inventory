use opentale_inventory::data::inventory::container::state::ContainerState;

#[test]
fn test_new_container() {
    let container = ContainerState::new("Test Container".to_string(), 5);
    assert_eq!(container.empty_slots(), 5);
    assert_eq!(container.occupied_slots(), 0);
}

#[test]
fn test_add_item() {
    let mut container = ContainerState::new("Test Container".to_string(), 3);
    
    let result = container.add_item("Sword".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // First slot
    assert_eq!(container.empty_slots(), 2);
    assert_eq!(container.occupied_slots(), 1);
}

#[test]
fn test_add_item_to_full_container() {
    let mut container = ContainerState::new("Full Container".to_string(), 2);
    
    container.add_item("Item1".to_string()).unwrap();
    container.add_item("Item2".to_string()).unwrap();
    
    let result = container.add_item("Item3".to_string());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Container is full");
}

#[test]
fn test_validate_success() {
    let container = ContainerState::new("Valid Container".to_string(), 5);
    assert!(container.validate().is_ok());
}

#[test]
fn test_validate_empty_name() {
    let mut container = ContainerState::new("Valid".to_string(), 5);
    // This test would require accessing private fields, so we'll test through constructor
    let empty_name_container = ContainerState::new("".to_string(), 5);
    assert!(empty_name_container.validate().is_err());
}

#[test]
fn test_validate_zero_slots() {
    let zero_slot_container = ContainerState::new("Test".to_string(), 0);
    let result = zero_slot_container.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least one slot"));
}

#[test]
fn test_empty_and_occupied_slots() {
    let mut container = ContainerState::new("Test".to_string(), 10);
    
    // Initially all empty
    assert_eq!(container.empty_slots(), 10);
    assert_eq!(container.occupied_slots(), 0);
    
    // Add some items
    container.add_item("Item1".to_string()).unwrap();
    container.add_item("Item2".to_string()).unwrap();
    
    assert_eq!(container.empty_slots(), 8);
    assert_eq!(container.occupied_slots(), 2);
}