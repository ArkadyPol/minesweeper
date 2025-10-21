use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Debug, Clone, Component)]
pub struct CountdownTimer {
    pub timer: Timer,
    pub remaining: u8,
}
