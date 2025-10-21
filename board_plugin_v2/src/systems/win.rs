use bevy::{color::palettes::css::GREEN, prelude::*};

use crate::{
    components::{Bomb, TileCover, Uncover},
    events::{BoardCompletedEvent, GameEndEvent},
};

pub fn uncover_bombs_on_win(
    _ev: On<BoardCompletedEvent>,
    cover_query: Query<Entity, (With<TileCover>, Without<Uncover>)>,
    bombs: Query<&mut Sprite, With<Bomb>>,
    mut commands: Commands,
) {
    for cover in cover_query {
        commands.entity(cover).despawn();
    }

    for mut sprite in bombs {
        sprite.color = Color::from(GREEN);
    }

    commands.trigger(GameEndEvent {
        message: "You win!".into(),
    });
}
