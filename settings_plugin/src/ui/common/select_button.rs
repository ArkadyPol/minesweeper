use bevy::{
    color::palettes::css::{BLUE, GRAY},
    ecs::relationship::RelatedSpawner,
    prelude::*,
    ui::Checked,
    ui_widgets::RadioButton,
};

use super::text;

pub fn select_button(parent: &mut RelatedSpawner<'_, ChildOf>, label: &str, selected: bool) {
    let mut cmd = parent.spawn((
        Name::new("Button"),
        Node {
            width: px(120),
            padding: px(4).all(),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        if selected {
            BackgroundColor(Color::from(BLUE))
        } else {
            BackgroundColor(Color::from(GRAY))
        },
        RadioButton,
        children![text(24.0, label)],
    ));

    if selected {
        cmd.insert(Checked);
    }
}
