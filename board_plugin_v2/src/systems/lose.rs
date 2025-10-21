use bevy::{
    color::palettes::css::{GREEN, ORANGE, RED, YELLOW},
    prelude::*,
};

use crate::{
    components::{Bomb, Flag, TileCover},
    events::{BombExplosionEvent, GameEndEvent},
};

pub fn uncover_tiles_on_lose(
    event: On<BombExplosionEvent>,
    tiles: Query<(Entity, &mut Sprite, Has<Bomb>)>,
    children: Query<&Children>,
    covers: Query<Entity, With<TileCover>>,
    flags: Query<Entity, With<Flag>>,
    mut commands: Commands,
) {
    for (entity, mut sprite, has_bomb) in tiles {
        if entity == event.0 {
            sprite.color = Color::from(RED);
            continue;
        }

        let mut has_flag = false;

        for child_entity in children.iter_descendants(entity) {
            if flags.get(child_entity).is_ok() {
                has_flag = true;
                break;
            }
        }

        sprite.color = match (has_bomb, has_flag) {
            (true, true) => Color::from(GREEN),
            (true, false) => Color::from(YELLOW),
            (false, true) => Color::from(ORANGE),
            (false, false) => sprite.color,
        };
    }

    for cover in covers {
        commands.entity(cover).despawn();
    }

    commands.trigger(GameEndEvent {
        message: "You lose!".into(),
    });
}
