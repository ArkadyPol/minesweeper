use bevy::prelude::Component;

#[derive(Component)]
pub struct SettingsUIRoot;

#[derive(Component)]
pub enum SettingsButtonAction {
    Start,
}
