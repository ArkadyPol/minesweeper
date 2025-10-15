use bevy::prelude::*;

pub fn text(font_size: f32, value: impl Into<String>) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(Color::WHITE),
    )
}
