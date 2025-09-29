use bevy::prelude::Component;

#[derive(Component)]
pub struct MenuUIRoot;

#[derive(Component)]
pub enum MenuButtonAction {
    StartGame,
    Quit,
}
