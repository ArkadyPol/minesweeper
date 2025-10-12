use bevy::prelude::*;

use super::text;

pub fn label(font: Handle<Font>, value: impl Into<String> + Clone) -> impl Bundle {
    (
        Name::new(value.clone().into()),
        Label,
        text(font.clone(), value),
    )
}
