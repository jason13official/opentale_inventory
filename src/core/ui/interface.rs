use bevy::asset::{AssetServer, Assets};
use bevy::prelude::{Commands, Res, ResMut};
use bevy::sprite::TextureAtlasLayout;

pub fn inventory_init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    // todo add container manager as resource for plugin and init startup state
) {

}