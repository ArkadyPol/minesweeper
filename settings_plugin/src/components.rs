use bevy::prelude::*;

use crate::input_value::InputValue;

#[derive(Component)]
pub struct SettingsUIRoot;

#[derive(Component)]
pub enum SettingsButtonAction {
    Start,
}

#[cfg(feature = "debug")]
use bevy::prelude::ReflectComponent;
#[cfg(feature = "debug")]
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Debug, Clone, Component)]
pub struct TextInput {
    pub value: InputValue,
    pub focused: bool,
    pub cursor_pos: usize,
    pub is_cursor_inserted: bool,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            value: InputValue::Str("".into()),
            focused: false,
            cursor_pos: 0,
            is_cursor_inserted: false,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct CursorTimer(pub Timer);

impl Default for CursorTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Repeating))
    }
}
