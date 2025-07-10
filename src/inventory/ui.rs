use bevy::prelude::*;
use super::components::*;

/// Sets up the initial game state and UI
pub fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut inventory: ResMut<Inventory>,
) {
    // add some test items to the inventory
    inventory.set_slot(0, Some(ItemStack::new("Apple", 16, 64))); // TODO: get away from instancing item stacks with raw strings
    inventory.set_slot(1, Some(ItemStack::new("Sword", 1, 1)));
    inventory.set_slot(2, Some(ItemStack::new("Apple", 32, 64)));

    // spawn the camera so we can actually see things
    commands.spawn(Camera2dBundle::default());

    // create our UI elements
    create_inventory_ui(&mut commands, &asset_server);
    create_held_item_ui(&mut commands, &asset_server);
}

/// Creates the main inventory UI
pub fn create_inventory_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Container for inventory slots
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(400.0),
                        height: Val::Px(80.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceEvenly, // spread them out evenly (hopefully)
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                    border_color: Color::rgb(0.6, 0.6, 0.6).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Create individual slots
                    for i in 0..SLOT_COUNT {
                        create_inventory_slot(parent, i, asset_server);
                    }
                });
        });
}

/// Creates a single inventory slot
pub fn create_inventory_slot(parent: &mut ChildBuilder, index: usize, asset_server: &Res<AssetServer>) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(SLOT_SIZE),
                    height: Val::Px(SLOT_SIZE),
                    margin: UiRect::all(Val::Px(SLOT_MARGIN)), // space between slots (social distancing)
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: Color::rgb(0.4, 0.4, 0.4).into(),
                border_color: Color::rgb(0.6, 0.6, 0.6).into(),
                ..default()
            },
            InventorySlot { index }, // tag this button as an InventorySlot instance
        ))
        .with_children(|parent| {
            // Text to display item name and count
            parent.spawn(TextBundle::from_section(
                "", // gets filled in later
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"), // please exist, dear font file
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ));
        });
}

/// Creates the UI that follows the cursor when holding an item
pub fn create_held_item_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            HeldItemDisplay, // tag this UI element as a HeldItemDisplay instance
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"), // it's definitely there, right...?
                    font_size: 20.0,
                    // color: Color::rgb(1.0, 1.0, 0.0),
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
            ));
        });
}