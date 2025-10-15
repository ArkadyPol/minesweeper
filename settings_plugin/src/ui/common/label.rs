use bevy::prelude::*;

use super::text;

pub fn label(value: impl Into<String> + Clone) -> impl Bundle {
    (Name::new(value.clone().into()), Label, text(24.0, value))
}
