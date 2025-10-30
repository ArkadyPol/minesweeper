use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy::prelude::ReflectComponent;
#[cfg(feature = "debug")]
use bevy_inspector_egui::prelude::ReflectInspectorOptions;

use crate::components::Coordinates;

#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Component)]
#[relationship(relationship_target = GridChildren)]
pub struct GridChildOf(pub Entity);

#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Component, Deref)]
#[relationship_target(relationship = GridChildOf)]
pub struct GridChildren(Vec<Entity>);

#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Component, Deref)]
pub struct Center(pub u8);

impl Center {
    pub fn get_size(&self) -> IVec2 {
        IVec2::splat(3_i32.pow(**self as u32))
    }
}

#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Component, Deref)]
pub struct GridMap(pub Vec<(Entity, Coordinates)>);
