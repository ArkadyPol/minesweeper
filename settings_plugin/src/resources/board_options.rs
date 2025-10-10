use bevy::prelude::{Resource, Vec3};
use serde::{Deserialize, Serialize};

/// Tile size options
#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileSize {
    /// Fixed tile size
    Fixed(f32),
    /// Window adaptative tile size
    Adaptive { min: f32, max: f32 },
}

impl Default for TileSize {
    fn default() -> Self {
        Self::Adaptive {
            min: 10.0,
            max: 50.0,
        }
    }
}

/// Board position customization options
#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoardPosition {
    /// Centered board
    Centered { offset: Vec3 },
    /// Custom position
    Custom(Vec3),
}

impl Default for BoardPosition {
    fn default() -> Self {
        Self::Centered {
            offset: Default::default(),
        }
    }
}

/// Board generation options. Must be used as a resource
// We use serde to allow saving option presets and loading them at runtime
#[cfg(feature = "debug")]
use bevy::prelude::ReflectResource;
#[cfg(feature = "debug")]
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
#[cfg_attr(
    feature = "debug",
    derive(bevy_inspector_egui::InspectorOptions, bevy::reflect::Reflect),
    reflect(Resource, InspectorOptions)
)]
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct BoardOptions {
    /// Tile map size
    pub map_size: (u16, u16),
    /// bomb count
    pub bomb_count: u16,
    /// Board world position
    pub position: BoardPosition,
    /// Tile world size
    pub tile_size: TileSize,
    /// Padding between tiles
    pub tile_padding: f32,
    /// Does the board generate a safe place to start
    pub safe_start: bool,
}

impl Default for BoardOptions {
    fn default() -> Self {
        Self {
            map_size: (15, 15),
            bomb_count: 30,
            position: Default::default(),
            tile_size: Default::default(),
            tile_padding: 0.,
            safe_start: false,
        }
    }
}

impl BoardOptions {
    pub fn set_width(&mut self, width: u16) -> Result<(), String> {
        let area = Self::get_area(width, self.map_size.1)?;

        if area < 4 {
            return Err("Width too small!".into());
        }

        if area > 3600 {
            return Err("Width too large!".into());
        }

        if area <= self.bomb_count {
            return Err("The bombs don't fit area!".into());
        }

        self.map_size.0 = width;
        Ok(())
    }

    pub fn set_height(&mut self, height: u16) -> Result<(), String> {
        let area = Self::get_area(self.map_size.0, height)?;

        if area < 4 {
            return Err("Height too small!".into());
        }

        if area > 3600 {
            return Err("Height too large!".into());
        }

        if area <= self.bomb_count {
            return Err("The bombs don't fit area!".into());
        }

        self.map_size.1 = height;
        Ok(())
    }

    pub fn set_bomb_count(&mut self, bombs: u16) -> Result<(), String> {
        let area = Self::get_area(self.map_size.0, self.map_size.1)?;

        if bombs >= area {
            return Err("Too many bombs!".into());
        }

        self.bomb_count = bombs;
        Ok(())
    }

    pub fn get_area(width: u16, height: u16) -> Result<u16, String> {
        width.checked_mul(height).ok_or("Area is too big!".into())
    }
}
