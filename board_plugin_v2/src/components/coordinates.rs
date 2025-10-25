use bevy::prelude::*;
use std::fmt::{self, Display, Formatter};
use std::ops::Add;

#[cfg(feature = "debug")]
use bevy::prelude::ReflectComponent;
#[cfg(feature = "debug")]
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Component, InspectorOptions)
)]
#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component)]
pub struct Coordinates {
    pub x: u16,
    pub y: u16,
}

impl Add<(i8, i8)> for Coordinates {
    type Output = Self;

    fn add(self, (x, y): (i8, i8)) -> Self::Output {
        let x = ((self.x as i16) + x as i16) as u16;
        let y = ((self.y as i16) + y as i16) as u16;
        Self { x, y }
    }
}

impl Add<IVec2> for Coordinates {
    type Output = Self;

    fn add(self, IVec2 { x, y }: IVec2) -> Self::Output {
        let x = ((self.x as i16) + x as i16) as u16;
        let y = ((self.y as i16) + y as i16) as u16;
        Self { x, y }
    }
}

impl Into<IVec2> for Coordinates {
    fn into(self) -> IVec2 {
        IVec2::new(self.x as i32, self.y as i32)
    }
}

impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
