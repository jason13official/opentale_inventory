use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{Camera2dBundle, Commands};
use crate::data::inventory::container::state::ContainerState;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        println!("Built an instance of InventoryPlugin.");

        app.add_systems(Startup, setup_game);
    }
}

/// Here, we set up the initial state of our game. This includes initializing our textures/spritesheets,
/// creating containers, spawning the camera, and creating the first UI elements to display on-screen.
pub fn setup_game(
    mut commands: Commands,
) {
    println!("Initializing game state from InventoryPlugin.");

    commands.spawn(Camera2dBundle::default());

    let mut container: ContainerState = ContainerState::new("chest_uuid".to_string(), 4);
    container.add_item("sword".to_string()).unwrap();
    container.add_item("potion".to_string()).unwrap();

    // save the contents of the inventory to a file
    container.save_to_file("chest_uuid.rson").unwrap();

    println!("read container as an RSON string, and print contents");
    // read container as an RSON string, and print contents
    let rson_string = container.to_rson_string().unwrap();
    let container_copy = ContainerState::from_rson_string(&rson_string).unwrap();
    for n in 0..container_copy.size() {
        let s = container_copy.peek_slot(n);
        if s != None {
            let details = s.unwrap();
            println!("{}", details)
        }
    }

    println!("read container from an RSON file, and print contents");
    // read container from an RSON file, and print contents
    let loaded_container = ContainerState::load_from_file("chest_uuid.rson").unwrap();
    for n in 0..loaded_container.size() {
        let s = loaded_container.peek_slot(n);
        if s != None {
            let details = s.unwrap();
            println!("{}", details)
        }
    }
}