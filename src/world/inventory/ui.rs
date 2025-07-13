use super::components::*;
use crate::world::inventory::containers::{ContainerLayout, ContainerManager, ContainerPosition, ContainerType, ContainerUI};
use crate::world::inventory::item_stack::ItemStack;
use crate::world::item::*;
use bevy::prelude::*;

/// Resource to hold the item spritesheet texture atlas
#[derive(Resource)]
pub struct ItemSpritesheet {
    pub texture_atlas: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
}

/// Sets up the initial game state and UI
pub fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut container_manager: ResMut<ContainerManager>,
) {
    // Load the item spritesheet and create texture atlas
    let texture = asset_server.load("textures/item/sprites.png");
    // The spritesheet is 128x144 pixels (8 columns by 9 rows), each sprite 16x16 pixels, tightly packed
    let layout = TextureAtlasLayout::from_grid(
        Vec2::new(16.0, 16.0), // sprite size
        8, // columns
        9, // rows (9 rows total)
        Some(Vec2::ZERO), // no padding between sprites
        Some(Vec2::ZERO), // no offset from edge
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    
    let spritesheet = ItemSpritesheet {
        texture_atlas: texture_atlas_layout,
        texture,
    };
    commands.insert_resource(spritesheet);
    // Add some test items to different containers

    // Add items to hotbar
    if let Some(hotbar) = container_manager.containers.get_mut(&ContainerType::Hotbar) {
        hotbar.set_slot(0, Some(ItemStack::new(items::APPLE, 16)));
        hotbar.set_slot(1, Some(ItemStack::new(items::BOW, 1)));
        hotbar.set_slot(2, Some(ItemStack::new(items::IRON_SWORD, 1)));
    }

    // Add items to player inventory
    if let Some(inventory) = container_manager.containers.get_mut(&ContainerType::PlayerInventory) {
        inventory.set_slot(0, Some(ItemStack::new(items::APPLE, 32)));
        inventory.set_slot(1, Some(ItemStack::new(items::RING, 8)));
        inventory.set_slot(9, Some(ItemStack::new(items::GLASS_BOTTLE, 12)));
    }

    // Add different items to different chests
    if let Some(chest1) = container_manager.containers.get_mut(&ContainerType::Chest(1)) {
        chest1.set_slot(0, Some(ItemStack::new(items::BOW, 1)));
        chest1.set_slot(1, Some(ItemStack::new(items::IRON_SWORD, 1)));
        chest1.set_slot(2, Some(ItemStack::new(items::CHEESE, 3)));
    }

    if let Some(chest2) = container_manager.containers.get_mut(&ContainerType::Chest(2)) {
        chest2.set_slot(0, Some(ItemStack::new(items::CHEESE, 64)));
        chest2.set_slot(1, Some(ItemStack::new(items::APPLE, 64)));
        chest2.set_slot(2, Some(ItemStack::new(items::GLASS_BOTTLE, 32)));
    }

    if let Some(chest3) = container_manager.containers.get_mut(&ContainerType::Chest(3)) {
        chest3.set_slot(0, Some(ItemStack::new(items::IRON_SWORD, 1)));
        chest3.set_slot(1, Some(ItemStack::new(items::RING, 10)));
    }

    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Create UI for the active containers
    create_minecraft_ui(&mut commands, &asset_server, &container_manager);
    create_held_item_ui(&mut commands, &asset_server);
    create_selected_item_ui(&mut commands, &asset_server);
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
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Left side - chest selection panel
            create_chest_selection_panel(parent, asset_server, container_manager);

            // Main UI area
            parent
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

                    // Middle section (for chests and player inventory grouped together)
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexEnd,
                                align_items: AlignItems::Center,
                                flex_grow: 1.0,
                                margin: UiRect::bottom(Val::Px(10.0)), // Small gap above hotbar
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {

                            // Create top containers (chest, etc.) - positioned above player inventory
                            for layout in &container_manager.layouts {
                                if matches!(layout.position, ContainerPosition::Top) {
                                    create_container_ui(parent, asset_server, layout);
                                }
                            }

                            // Create center containers (player inventory) - positioned below chests
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
            z_index: ZIndex::Global(1000),
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
            // Container for item sprite and count
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Item sprite
                    parent.spawn((
                        AtlasImageBundle {
                            style: Style {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                ..default()
                            },
                            image: UiImage::default(),
                            texture_atlas: TextureAtlas::default(),
                            visibility: Visibility::Hidden,
                            ..default()
                        },
                        ItemSprite,
                    ));
                    
                    // Item count text (positioned absolutely in bottom-right)
                    parent.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 12.0,
                                color: Color::WHITE,
                            },
                        ).with_style(Style {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(2.0),
                            right: Val::Px(2.0),
                            ..default()
                        }),
                        ItemCountText,
                    ));
                    
                    // Drag preview text (positioned absolutely in top-left)
                    parent.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 10.0,
                                color: Color::rgb(0.0, 1.0, 0.0),
                            },
                        ).with_style(Style {
                            position_type: PositionType::Absolute,
                            top: Val::Px(2.0),
                            left: Val::Px(2.0),
                            ..default()
                        }),
                        SlotPreviewText,
                    ));
                });
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

/// Creates the UI that displays the selected hotbar item in the bottom-right corner
pub fn create_selected_item_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(SLOT_SIZE + 20.0),
                            height: Val::Px(SLOT_SIZE + 20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        background_color: Color::rgba(0.2, 0.2, 0.2, 0.9).into(),
                        // border_color: Color::rgb(0.6, 0.6, 0.6).into(),
                        border_color: Color::rgb(1.0, 1.0, 0.0).into(),
                        ..default()
                    },
                    SelectedItemDisplay,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    ));
                });
        });
}

/// Creates the left-side chest selection panel
pub fn create_chest_selection_panel(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    container_manager: &ContainerManager,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(80.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.8).into(),
            ..default()
        })
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle::from_section(
                "Chests",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 14.0,
                    color: Color::WHITE,
                },
            ));

            // Chest buttons
            for &chest_id in &container_manager.available_chests {
                let is_active = container_manager.active_chest_id == Some(chest_id);
                let button_color = if is_active {
                    Color::rgb(0.3, 0.6, 0.3) // Green for active
                } else {
                    Color::rgb(0.4, 0.4, 0.4) // Gray for inactive
                };

                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(60.0),
                                height: Val::Px(60.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            background_color: button_color.into(),
                            border_color: Color::rgb(0.6, 0.6, 0.6).into(),
                            ..default()
                        },
                        ChestButton { chest_id },
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            chest_id.to_string(),
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 18.0,
                                color: Color::WHITE,
                            },
                        ));
                    });
            }
        });
}

#[derive(Component)]
pub struct ChestButton {
    pub chest_id: u32,
}