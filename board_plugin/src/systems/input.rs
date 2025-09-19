use crate::Board;
use bevy::{input::mouse::MouseButtonInput, log, prelude::*, window::PrimaryWindow};

pub fn input_handling(
    window: Query<&Window, With<PrimaryWindow>>,
    board: Res<Board>,
    mut button_evr: EventReader<MouseButtonInput>,
) {
    let window = window.single().unwrap();

    for event in button_evr.read() {
        if event.state.is_pressed() {
            let position = window.cursor_position();
            if let Some(pos) = position {
                log::trace!("Mouse button pressed: {:?} at {}", event.button, pos);
                let tile_coordinates = board.mouse_position(window, pos);
                if let Some(coordinates) = tile_coordinates {
                    match event.button {
                        MouseButton::Left => {
                            log::info!("Trying to uncover tile on {}", coordinates);
                            // TODO: generate an event
                        }
                        MouseButton::Right => {
                            log::info!("Trying to mark tile on {}", coordinates);
                            // TODO: generate an event
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}
