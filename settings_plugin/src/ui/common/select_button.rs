use bevy::{
    color::palettes::css::{BLUE, GRAY},
    ecs::relationship::RelatedSpawner,
    prelude::*,
    ui::Checked,
    ui_widgets::RadioButton,
};

use crate::components::Controlled;

use super::text;

pub fn select_button(
    parent: &mut RelatedSpawner<'_, ChildOf>,
    caption: &str,
    selected: bool,
    controls: impl Bundle,
) {
    let mut cmd = parent.spawn((
        Name::new("Button"),
        Node {
            width: px(120),
            padding: px(4).all(),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::from(if selected { BLUE } else { GRAY })),
        RadioButton,
        children![text(24.0, caption)],
        related!(Controlled[controls]),
    ));

    if selected {
        cmd.insert(Checked);
    }
}
