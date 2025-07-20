use bevy::asset::Handle;
use bevy::prelude::{Component, Image, Resource};

/// Track the UI elements original size at normal scaling
#[derive(Component)]
pub struct BaseSize {
    pub width: f32,
    pub height: f32,
}

/// A resource for our game app to track the current user-set scaling
#[derive(Resource)]
pub struct UIScale {
    pub scale: u8,
}

/// Used to mark UI elements that should update their scaling along with the UI scaling
#[derive(Component)]
pub struct ScalableUIElement;

/// Hard-coded list of images used for menus. TODO: Should probably replace with a registry/macro implementation
#[derive(Resource)]
pub struct UITextures {
    pub hotbar: Handle<Image>,
    pub chest_menu: Handle<Image>,
}

impl Default for UIScale {
    fn default() -> Self {
        Self {
            scale: 2
        }
    }
}

impl UIScale {

    pub fn cycle_up(&mut self) {
        self.scale = if self.scale >= 4 { 1 } else { self.scale + 1 };
    }

    pub fn cycle_down(&mut self) {
        self.scale = if self.scale <= 1 { 4 } else { self.scale - 1 };
    }
}