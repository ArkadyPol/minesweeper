use std::{
    fmt,
    num::{ParseFloatError, ParseIntError},
};

use bevy::prelude::Component;

#[derive(Component)]
pub struct SettingsUIRoot;

#[derive(Component)]
pub enum SettingsButtonAction {
    Start,
}

#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
#[derive(Debug, Clone)]
pub enum InputValue {
    Str(String),
    Float(f32),
    Int(i32),
}

#[derive(Debug)]
pub enum InputError {
    ParseFloatError(ParseFloatError),
    ParseIntError(ParseIntError),
}

impl From<ParseFloatError> for InputError {
    fn from(value: ParseFloatError) -> Self {
        InputError::ParseFloatError(value)
    }
}

impl From<ParseIntError> for InputError {
    fn from(value: ParseIntError) -> Self {
        InputError::ParseIntError(value)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::ParseFloatError(e) => write!(f, "{}", e),
            InputError::ParseIntError(e) => write!(f, "{}", e),
        }
    }
}

impl InputValue {
    pub fn parse_and_mut(&mut self, value: &str) -> Result<(), InputError> {
        match self {
            InputValue::Str(s) => *s = value.into(),
            InputValue::Float(f) => *f = value.parse::<f32>()?,
            InputValue::Int(i) => *i = value.parse::<i32>()?,
        }
        Ok(())
    }

    pub fn as_string(&self) -> String {
        match self {
            InputValue::Str(s) => s.into(),
            InputValue::Float(f) => f.to_string(),
            InputValue::Int(i) => i.to_string(),
        }
    }
}

impl From<f32> for InputValue {
    fn from(value: f32) -> Self {
        InputValue::Float(value)
    }
}

impl From<i32> for InputValue {
    fn from(value: i32) -> Self {
        InputValue::Int(value)
    }
}

impl From<String> for InputValue {
    fn from(value: String) -> Self {
        InputValue::Str(value)
    }
}

impl Into<String> for InputValue {
    fn into(self) -> String {
        self.as_string()
    }
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
