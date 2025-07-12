use bevy::prelude::*;
use crate::world::inventory::containers::{ContainerLayout, ContainerManager, ContainerPosition, ContainerType, ContainerUI};
use crate::world::inventory::inventory::SlotContainer;
use crate::world::inventory::item_stack::ItemStack;
use crate::world::item::*;
use super::components::*;

/// Sets up the initial game state and UI
pub fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut container_manager: ResMut<ContainerManager>,
) {
    // Add some test items to different containers

    // Add items to hotbar
    if let Some(hotbar) = container_manager.containers.get_mut(&ContainerType::Hotbar) {
        hotbar.set_slot(0, Some(ItemStack::new(items::APPLE, 16)));
        hotbar.set_slot(1, Some(ItemStack::new(items::DIAMOND, 2)));
        hotbar.set_slot(2, Some(ItemStack::new(items::IRON_SWORD, 1)));
    }

    // Add items to player inventory
    if let Some(inventory) = container_manager.containers.get_mut(&ContainerType::PlayerInventory) {
        inventory.set_slot(0, Some(ItemStack::new(items::APPLE, 32)));
        inventory.set_slot(1, Some(ItemStack::new(items::BREAD, 8)));
        inventory.set_slot(9, Some(ItemStack::new(items::GLASS_BOTTLE, 12)));
    }

    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Create UI for the active containers
    create_minecraft_ui(&mut commands, &asset_server, &container_manager);
    create_held_item_ui(&mut commands, &asset_server);
    create_hud(&mut commands, &asset_server);
}

pub fn create_minecraft_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    container_manager: &ContainerManager,
) {
    // Create a root container for all UI elements
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Top section (for chests and other containers)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Create top containers (chest, etc.)
                    for layout in &container_manager.layouts {
                        if matches!(layout.position, ContainerPosition::Top) {
                            create_container_ui(parent, asset_server, layout);
                        }
                    }
                });

            // Middle section (for player inventory)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Create center containers (player inventory)
                    for layout in &container_manager.layouts {
                        if matches!(layout.position, ContainerPosition::Center) {
                            create_container_ui(parent, asset_server, layout);
                        }
                    }
                });

            // Bottom section (for hotbar)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Create bottom containers (hotbar)
                    for layout in &container_manager.layouts {
                        if matches!(layout.position, ContainerPosition::Bottom) {
                            create_container_ui(parent, asset_server, layout);
                        }
                    }
                });
        });
}

pub fn create_container_ui(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    layout: &ContainerLayout
) {
    let container_width = (layout.columns as f32) * (SLOT_SIZE + SLOT_MARGIN * 2.0);
    let container_height = (layout.rows as f32) * (SLOT_SIZE + SLOT_MARGIN * 2.0);

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            },
            ContainerUI { container_type: layout.container_type.clone() },
        ))
        .with_children(|parent| {
            // Title (only if not empty)
            if !layout.title.is_empty() {
                parent.spawn(TextBundle::from_section(
                    &layout.title,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                ));
            }

            // Container background
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(container_width + 20.0),
                        height: Val::Px(container_height + 20.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.2, 0.2, 0.2, 0.9).into(),
                    border_color: Color::rgb(0.6, 0.6, 0.6).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Create rows
                    for row in 0..layout.rows {
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|parent| {
                                // Create slots in this row
                                for col in 0..layout.columns {
                                    let slot_index = row * layout.columns + col;
                                    if slot_index < layout.slot_count {
                                        create_inventory_slot(parent, slot_index, asset_server, &layout.container_type);
                                    }
                                }
                            });
                    }
                });
        });
}

/// Create HUD with container switching buttons
pub fn create_hud(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Top bar with instructions
            parent
                .spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "E: Open/Close Inventory | C: Open/Close Chest | Esc: Close All",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    ));
                });
        });
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
                        width: Val::Px(640.0),
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
                        create_inventory_slot(parent, i, asset_server, &ContainerType::PlayerInventory);
                    }
                });
        });
}

/// Creates a single inventory slot
pub fn create_inventory_slot(
    parent: &mut ChildBuilder,
    index: usize,
    asset_server: &Res<AssetServer>,
    container_type: &ContainerType
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(SLOT_SIZE),
                    height: Val::Px(SLOT_SIZE),
                    margin: UiRect::all(Val::Px(SLOT_MARGIN)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: Color::rgb(0.4, 0.4, 0.4).into(),
                border_color: Color::rgb(0.6, 0.6, 0.6).into(),
                ..default()
            },
            InventorySlot {
                index,
                container_type: container_type.clone()
            },
        ))
        .with_children(|parent| {
            // Text to display item name and count
            parent.spawn(TextBundle::from_section(
                "",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
            HeldItemDisplay,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
            ));
        });
}