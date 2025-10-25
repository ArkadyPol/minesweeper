use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy::prelude::ReflectComponent;
#[cfg(feature = "debug")]
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Debug, Deref, Component)]
pub struct Neighbors(pub [Option<Entity>; 8]);
// Neighbors2
#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Component)]
#[relationship(relationship_target = Neighbors2)]
pub struct NeighborOf(pub Entity);

#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Component, Deref)]
#[relationship_target(relationship = NeighborOf)]
pub struct Neighbors2(Vec<Entity>);
// Levels
#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Component)]
#[relationship(relationship_target = LevelDown)]
pub struct LevelUp(pub Entity);

#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Component, Deref)]
#[relationship_target(relationship = LevelUp)]
pub struct LevelDown(Entity);

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

#[derive(Component)]
pub struct VirtualCenter;
