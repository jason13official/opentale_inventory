pub mod container;
pub mod visual;
pub mod slot_utils;
pub mod drag_visuals;
pub mod display_systems;
pub mod slot_rendering;

// Re-export public functions to maintain API compatibility
pub use slot_rendering::update_slot_visuals;
pub use display_systems::{update_held_item_display, update_slot_hover_effects, update_selected_item_display};