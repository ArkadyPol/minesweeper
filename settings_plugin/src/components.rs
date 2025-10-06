use bevy::prelude::Component;

#[derive(Component)]
pub struct SettingsUIRoot;

#[derive(Component)]
pub enum SettingsButtonAction {
    Start,
}

#[derive(Component)]
pub struct TextInput {
    pub value: String,
    pub focused: bool,
    pub cursor_pos: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            value: "".into(),
            focused: false,
            cursor_pos: 0,
        }
    }
}
