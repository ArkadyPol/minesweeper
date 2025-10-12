use bevy::prelude::*;

pub fn text(font: Handle<Font>, value: impl Into<String>) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font: font.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
    )
}
