//! DMX Types and Data Structures
//!
//! This module defines all core types for the DMX lighting control system:
//!
//! ## Fixture Management
//! - [`FixtureTemplate`] - Defines a type of fixture with its channel layouts
//! - [`FixtureMode`] - A specific channel configuration for a fixture
//! - [`ChannelDef`] - Individual channel definition within a mode
//! - [`Fixture`] - An instance of a fixture with current runtime values
//! - [`FixtureTemplateLibrary`] - Collection of all available fixture templates
//!
//! ## Channel Types
//! - [`ChannelType`] - Enum of all possible DMX channel functions
//! - [`Color`] - RGBW color values for fixture output
//!
//! ## Playback
//! - [`Cue`] - A snapshot of DMX values with timing information
//! - [`Executor`] - Playback controller with fader and cue list
//! - [`DMXBufferValue`] - Single channel value for buffer manipulation
//!
//! ## Audio (Future)
//! - [`AudioTrack`] - Audio file playback for shows
//! - [`AudioAction`] - Audio behavior modes
//!
//! ## Groups
//! - [`FixtureGroup`] - Collective control of multiple fixtures

pub use open_dmx::DMX_CHANNELS;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Color values for RGB-type fixtures.
/// Represents the color channels commonly found in LED PARs and moving lights.
/// Each field holds a DMX value (0-255) for that color component.
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Color {
    /// Red channel value (0-255)
    pub r: u8,
    /// Green channel value (0-255)
    pub g: u8,
    /// Blue channel value (0-255)
    pub b: u8,
    /// White channel value (0-255) - used in RGBW fixtures
    pub w: u8,
    /// Amber channel value (0-255) - used in RGBWA fixtures
    pub amber: u8,
    /// UV (Ultraviolet) channel value (0-255) - used in RGBWAU fixtures
    pub uv: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            w: 0,
            amber: 0,
            uv: 0,
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self::from_rgb(r, g, b))
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn has_color(&self) -> bool {
        self.r != 0 || self.g != 0 || self.b != 0 || self.w != 0
    }
}

/// Channel type definitions for fixtures.
/// Represents the different types of DMX channels that a fixture can have.
/// Each variant corresponds to a specific function or color in a lighting fixture.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ChannelType {
    /// Intensity/Dimmer - controls overall brightness (0 = off, 255 = full)
    Intensity,
    /// Red color channel for RGB mixing
    Red,
    /// Green color channel for RGB mixing
    Green,
    /// Blue color channel for RGB mixing
    Blue,
    /// White color channel for RGBW fixtures
    White,
    /// Amber color channel for RGBWA fixtures (adds warm orange)
    Amber,
    /// UV (Ultraviolet) channel for blacklight effects
    UV,
    /// Color wheel - cycles through predefined colors
    ColorWheel,
    /// CTO (Color Temperature Orange) - warms the color temperature
    CTO,
    /// CTB (Color Temperature Blue) - cools the color temperature
    CTB,
    /// Pan - horizontal movement for moving heads (coarse 8-bit)
    Pan,
    /// Pan Fine - fine horizontal adjustment (16-bit, used with Pan)
    PanFine,
    /// Tilt - vertical movement for moving heads (coarse 8-bit)
    Tilt,
    /// Tilt Fine - fine vertical adjustment (16-bit, used with Tilt)
    TiltFine,
    /// Gobo Wheel - rotates through pattern gobos
    GoboWheel,
    /// Gobo Rotation - rotates the current gobo pattern
    GoboRotation,
    /// Secondary Gobo Wheel - additional gobo patterns
    GoboWheel2,
    /// Rotation for secondary gobo wheel
    GoboRotation2,
    /// Shutter - blocks light output for strobe effects
    Shutter,
    /// Strobe - electronic strobe frequency control
    Strobe,
    /// Zoom - adjusts beam angle (narrow to wide)
    Zoom,
    /// Focus - adjusts beam sharpness/distance focus
    Focus,
    /// Prism - rotates a prism effect
    Prism,
    /// Frost - diffuses the beam for wash effect
    Frost,
    /// Control - special functions like lamp on/off, reset
    Control,
    /// Speed - controls movement speed for moving heads
    Speed,
}
impl ChannelType {
    pub fn is(&self, v: Self) -> bool {
        *self == v
    }
}

impl ChannelType {
    pub fn name(&self) -> &'static str {
        match self {
            ChannelType::Intensity => "Intensity",
            ChannelType::Red => "Red",
            ChannelType::Green => "Green",
            ChannelType::Blue => "Blue",
            ChannelType::White => "White",
            ChannelType::Amber => "Amber",
            ChannelType::UV => "UV",
            ChannelType::ColorWheel => "Color Wheel",
            ChannelType::CTO => "CTO (Warm)",
            ChannelType::CTB => "CTB (Cool)",
            ChannelType::Pan => "Pan",
            ChannelType::PanFine => "Pan Fine",
            ChannelType::Tilt => "Tilt",
            ChannelType::TiltFine => "Tilt Fine",
            ChannelType::GoboWheel => "Gobo Wheel",
            ChannelType::GoboRotation => "Gobo Rotation",
            ChannelType::GoboWheel2 => "Gobo Wheel 2",
            ChannelType::GoboRotation2 => "Gobo Rotation 2",
            ChannelType::Shutter => "Shutter",
            ChannelType::Strobe => "Strobe",
            ChannelType::Zoom => "Zoom",
            ChannelType::Focus => "Focus",
            ChannelType::Prism => "Prism",
            ChannelType::Frost => "Frost",
            ChannelType::Control => "Control",
            ChannelType::Speed => "Speed",
        }
    }
}

/// Definition of a single channel in a fixture mode.
/// Describes what type of control this channel provides and its position
/// within the fixture's DMX footprint.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChannelDef {
    /// The type of function this channel controls (e.g., Red, Pan, Intensity)
    pub channel_type: ChannelType,
    /// Offset from the fixture's start channel (0-based).
    /// For a fixture starting at DMX channel 5 with Red at offset 1,
    /// Red is actually on channel 6.
    pub offset: u8,
    /// Human-readable name for this channel (auto-generated from channel_type)
    pub name: String,
}

impl ChannelDef {
    pub fn new(channel_type: ChannelType, offset: u8) -> Self {
        Self {
            name: channel_type.name().to_string(),
            channel_type,
            offset,
        }
    }
}

/// A mode definition for a fixture template (e.g., 8ch, 16ch).
/// Fixture modes define different channel layouts for the same physical fixture.
/// Common modes include "Dimmer" (with intensity channel) and "RGB" (without).
/// Different modes use different numbers of DMX channels.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FixtureMode {
    /// Name of this mode (e.g., "4ch (Dimmer)", "3ch (RGB)", "17ch")
    pub name: String,
    /// List of channel definitions in order
    pub channels: Vec<ChannelDef>,
}

impl FixtureMode {
    pub fn new(name: &str, channels: Vec<ChannelDef>) -> Self {
        Self {
            name: name.to_string(),
            channels,
        }
    }

    pub fn total_channels(&self) -> usize {
        self.channels.len()
    }
}

/// A fixture template defining channel layouts.
/// Represents a "type" of fixture (e.g., "Generic RGB Par") rather than
/// a specific instance. Contains multiple modes with different channel layouts.
/// Templates can be predefined (built-in) or user-defined.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FixtureTemplate {
    /// Unique identifier for this template
    pub id: u32,
    /// Display name (e.g., "Generic RGB Par", "Martin MAC 250")
    pub name: String,
    /// Manufacturer name (e.g., "Generic", "Martin", "Chauvet")
    pub manufacturer: String,
    /// Available modes for this fixture
    pub modes: Vec<FixtureMode>,
    /// Whether this template was created by the user (true) or is built-in (false)
    pub is_user_defined: bool,
}

impl FixtureTemplate {
    pub fn new(id: u32, name: &str, manufacturer: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            manufacturer: manufacturer.to_string(),
            modes: Vec::new(),
            is_user_defined: false,
        }
    }

    pub fn add_mode(&mut self, mode: FixtureMode) {
        self.modes.push(mode);
    }

    pub fn get_mode(&self, index: usize) -> Option<&FixtureMode> {
        self.modes.get(index)
    }

    pub fn default_mode(&self) -> Option<&FixtureMode> {
        self.modes.first()
    }
}

/// Library of fixture templates (predefined + user-defined).
/// Contains all available fixture definitions including built-in templates
/// for common fixtures and any user-created templates.
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct FixtureTemplateLibrary {
    /// All available fixture templates
    pub templates: Vec<FixtureTemplate>,
    /// Next available ID for user-created templates
    pub next_id: u32,
}

impl FixtureTemplateLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            templates: Vec::new(),
            next_id: 1,
        };
        library.load_predefined_templates();
        library
    }

    pub fn add_user_template(&mut self, template: FixtureTemplate) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let mut template = template;
        template.id = id;
        template.is_user_defined = true;
        self.templates.push(template);
        id
    }

    pub fn get_template(&self, id: u32) -> Option<&FixtureTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    pub fn get_template_mut(&mut self, id: u32) -> Option<&mut FixtureTemplate> {
        self.templates.iter_mut().find(|t| t.id == id)
    }

    pub fn predefined_templates(&self) -> Vec<&FixtureTemplate> {
        self.templates
            .iter()
            .filter(|t| !t.is_user_defined)
            .collect()
    }

    pub fn user_templates(&self) -> Vec<&FixtureTemplate> {
        self.templates
            .iter()
            .filter(|t| t.is_user_defined)
            .collect()
    }

    fn load_predefined_templates(&mut self) {
        // Single Channel - Dimmer
        let mut dimmer = FixtureTemplate::new(self.next_id, "Generic Dimmer", "Generic");
        self.next_id += 1;
        dimmer.add_mode(FixtureMode::new(
            "1ch",
            vec![ChannelDef::new(ChannelType::Intensity, 0)],
        ));
        self.templates.push(dimmer);

        // RGB Par
        let mut rgb = FixtureTemplate::new(self.next_id, "Generic RGB Par", "Generic");
        self.next_id += 1;
        rgb.add_mode(FixtureMode::new(
            "4ch (Dimmer)",
            vec![
                ChannelDef::new(ChannelType::Intensity, 0),
                ChannelDef::new(ChannelType::Red, 1),
                ChannelDef::new(ChannelType::Green, 2),
                ChannelDef::new(ChannelType::Blue, 3),
            ],
        ));
        rgb.add_mode(FixtureMode::new(
            "3ch (RGB)",
            vec![
                ChannelDef::new(ChannelType::Red, 0),
                ChannelDef::new(ChannelType::Green, 1),
                ChannelDef::new(ChannelType::Blue, 2),
            ],
        ));
        self.templates.push(rgb);

        // RGBW Par
        let mut rgbw = FixtureTemplate::new(self.next_id, "Generic RGBW Par", "Generic");
        self.next_id += 1;
        rgbw.add_mode(FixtureMode::new(
            "5ch (Dimmer)",
            vec![
                ChannelDef::new(ChannelType::Intensity, 0),
                ChannelDef::new(ChannelType::Red, 1),
                ChannelDef::new(ChannelType::Green, 2),
                ChannelDef::new(ChannelType::Blue, 3),
                ChannelDef::new(ChannelType::White, 4),
            ],
        ));
        rgbw.add_mode(FixtureMode::new(
            "4ch (RGBW)",
            vec![
                ChannelDef::new(ChannelType::Red, 0),
                ChannelDef::new(ChannelType::Green, 1),
                ChannelDef::new(ChannelType::Blue, 2),
                ChannelDef::new(ChannelType::White, 3),
            ],
        ));
        self.templates.push(rgbw);

        // RGBA Par
        let mut rgba = FixtureTemplate::new(self.next_id, "Generic RGBA Par", "Generic");
        self.next_id += 1;
        rgba.add_mode(FixtureMode::new(
            "5ch (Dimmer)",
            vec![
                ChannelDef::new(ChannelType::Intensity, 0),
                ChannelDef::new(ChannelType::Red, 1),
                ChannelDef::new(ChannelType::Green, 2),
                ChannelDef::new(ChannelType::Blue, 3),
                ChannelDef::new(ChannelType::Amber, 4),
            ],
        ));
        rgba.add_mode(FixtureMode::new(
            "4ch (RGBA)",
            vec![
                ChannelDef::new(ChannelType::Red, 0),
                ChannelDef::new(ChannelType::Green, 1),
                ChannelDef::new(ChannelType::Blue, 2),
                ChannelDef::new(ChannelType::Amber, 3),
            ],
        ));
        self.templates.push(rgba);

        // RGBWAU Par (6 channel)
        let mut rgbwau = FixtureTemplate::new(self.next_id, "Generic RGBWAU Par", "Generic");
        self.next_id += 1;
        rgbwau.add_mode(FixtureMode::new(
            "7ch (Dimmer)",
            vec![
                ChannelDef::new(ChannelType::Intensity, 0),
                ChannelDef::new(ChannelType::Red, 1),
                ChannelDef::new(ChannelType::Green, 2),
                ChannelDef::new(ChannelType::Blue, 3),
                ChannelDef::new(ChannelType::White, 4),
                ChannelDef::new(ChannelType::Amber, 5),
                ChannelDef::new(ChannelType::UV, 6),
            ],
        ));
        rgbwau.add_mode(FixtureMode::new(
            "6ch",
            vec![
                ChannelDef::new(ChannelType::Red, 0),
                ChannelDef::new(ChannelType::Green, 1),
                ChannelDef::new(ChannelType::Blue, 2),
                ChannelDef::new(ChannelType::White, 3),
                ChannelDef::new(ChannelType::Amber, 4),
                ChannelDef::new(ChannelType::UV, 5),
            ],
        ));
        self.templates.push(rgbwau);

        // Generic Moving Head
        let mut moving = FixtureTemplate::new(self.next_id, "Generic Moving Head", "Generic");
        self.next_id += 1;
        moving.add_mode(FixtureMode::new(
            "17ch",
            vec![
                ChannelDef::new(ChannelType::Intensity, 0),
                ChannelDef::new(ChannelType::Pan, 1),
                ChannelDef::new(ChannelType::PanFine, 2),
                ChannelDef::new(ChannelType::Tilt, 3),
                ChannelDef::new(ChannelType::TiltFine, 4),
                ChannelDef::new(ChannelType::Speed, 5),
                ChannelDef::new(ChannelType::ColorWheel, 6),
                ChannelDef::new(ChannelType::GoboWheel, 7),
                ChannelDef::new(ChannelType::GoboRotation, 8),
                ChannelDef::new(ChannelType::Shutter, 9),
                ChannelDef::new(ChannelType::Focus, 10),
                ChannelDef::new(ChannelType::Zoom, 11),
                ChannelDef::new(ChannelType::Prism, 12),
                ChannelDef::new(ChannelType::Control, 13),
                ChannelDef::new(ChannelType::Red, 14),
                ChannelDef::new(ChannelType::Green, 15),
                ChannelDef::new(ChannelType::Blue, 16),
            ],
        ));
        moving.add_mode(FixtureMode::new(
            "12ch",
            vec![
                ChannelDef::new(ChannelType::Pan, 0),
                ChannelDef::new(ChannelType::PanFine, 1),
                ChannelDef::new(ChannelType::Tilt, 2),
                ChannelDef::new(ChannelType::TiltFine, 3),
                ChannelDef::new(ChannelType::Speed, 4),
                ChannelDef::new(ChannelType::ColorWheel, 5),
                ChannelDef::new(ChannelType::GoboWheel, 6),
                ChannelDef::new(ChannelType::Shutter, 7),
                ChannelDef::new(ChannelType::Intensity, 8),
                ChannelDef::new(ChannelType::Focus, 9),
                ChannelDef::new(ChannelType::Zoom, 10),
                ChannelDef::new(ChannelType::Control, 11),
            ],
        ));
        moving.add_mode(FixtureMode::new(
            "8ch",
            vec![
                ChannelDef::new(ChannelType::Pan, 0),
                ChannelDef::new(ChannelType::Tilt, 1),
                ChannelDef::new(ChannelType::Speed, 2),
                ChannelDef::new(ChannelType::ColorWheel, 3),
                ChannelDef::new(ChannelType::GoboWheel, 4),
                ChannelDef::new(ChannelType::Shutter, 5),
                ChannelDef::new(ChannelType::Intensity, 6),
                ChannelDef::new(ChannelType::Control, 7),
            ],
        ));
        self.templates.push(moving);

        // Martin MAC 250
        let mut mac250 = FixtureTemplate::new(self.next_id, "Martin MAC 250", "Martin");
        self.next_id += 1;
        mac250.add_mode(FixtureMode::new(
            "22ch",
            vec![
                ChannelDef::new(ChannelType::Pan, 0),
                ChannelDef::new(ChannelType::PanFine, 1),
                ChannelDef::new(ChannelType::Tilt, 2),
                ChannelDef::new(ChannelType::TiltFine, 3),
                ChannelDef::new(ChannelType::Speed, 4),
                ChannelDef::new(ChannelType::ColorWheel, 5),
                ChannelDef::new(ChannelType::GoboWheel, 6),
                ChannelDef::new(ChannelType::GoboRotation, 7),
                ChannelDef::new(ChannelType::GoboWheel2, 8),
                ChannelDef::new(ChannelType::Shutter, 9),
                ChannelDef::new(ChannelType::Intensity, 10),
                ChannelDef::new(ChannelType::Focus, 11),
                ChannelDef::new(ChannelType::Zoom, 12),
                ChannelDef::new(ChannelType::Prism, 13),
                ChannelDef::new(ChannelType::Frost, 14),
                ChannelDef::new(ChannelType::Control, 15),
                ChannelDef::new(ChannelType::Red, 16),
                ChannelDef::new(ChannelType::Green, 17),
                ChannelDef::new(ChannelType::Blue, 18),
                ChannelDef::new(ChannelType::White, 19),
                ChannelDef::new(ChannelType::Amber, 20),
                ChannelDef::new(ChannelType::UV, 21),
            ],
        ));
        self.templates.push(mac250);

        // Chauvet DJ SlimPAR
        let mut slimpar = FixtureTemplate::new(self.next_id, "Chauvet DJ SlimPAR Q12", "Chauvet");
        self.next_id += 1;
        slimpar.add_mode(FixtureMode::new(
            "6ch",
            vec![
                ChannelDef::new(ChannelType::Red, 0),
                ChannelDef::new(ChannelType::Green, 1),
                ChannelDef::new(ChannelType::Blue, 2),
                ChannelDef::new(ChannelType::White, 3),
                ChannelDef::new(ChannelType::Amber, 4),
                ChannelDef::new(ChannelType::Strobe, 5),
            ],
        ));
        slimpar.add_mode(FixtureMode::new(
            "10ch",
            vec![
                ChannelDef::new(ChannelType::Red, 0),
                ChannelDef::new(ChannelType::Green, 1),
                ChannelDef::new(ChannelType::Blue, 2),
                ChannelDef::new(ChannelType::White, 3),
                ChannelDef::new(ChannelType::Amber, 4),
                ChannelDef::new(ChannelType::Strobe, 5),
                ChannelDef::new(ChannelType::ColorWheel, 6),
                ChannelDef::new(ChannelType::Zoom, 7),
                ChannelDef::new(ChannelType::Focus, 8),
                ChannelDef::new(ChannelType::Control, 9),
            ],
        ));
        self.templates.push(slimpar);

        // LED Bar
        let mut ledbar = FixtureTemplate::new(self.next_id, "Generic LED Bar", "Generic");
        self.next_id += 1;
        ledbar.add_mode(FixtureMode::new(
            "4x RGB",
            vec![
                ChannelDef::new(ChannelType::Red, 0),
                ChannelDef::new(ChannelType::Green, 1),
                ChannelDef::new(ChannelType::Blue, 2),
                ChannelDef::new(ChannelType::Intensity, 3),
            ],
        ));
        ledbar.add_mode(FixtureMode::new(
            "8x RGB",
            vec![
                ChannelDef::new(ChannelType::Red, 0),
                ChannelDef::new(ChannelType::Green, 1),
                ChannelDef::new(ChannelType::Blue, 2),
                ChannelDef::new(ChannelType::Intensity, 3),
                ChannelDef::new(ChannelType::Red, 4),
                ChannelDef::new(ChannelType::Green, 5),
                ChannelDef::new(ChannelType::Blue, 6),
                ChannelDef::new(ChannelType::Intensity, 7),
            ],
        ));
        self.templates.push(ledbar);

        // Strobe
        let mut strobe = FixtureTemplate::new(self.next_id, "Generic Strobe", "Generic");
        self.next_id += 1;
        strobe.add_mode(FixtureMode::new(
            "2ch",
            vec![
                ChannelDef::new(ChannelType::Strobe, 0),
                ChannelDef::new(ChannelType::Intensity, 1),
            ],
        ));
        self.templates.push(strobe);

        // Blinder
        let mut blinder = FixtureTemplate::new(self.next_id, "Generic Blinder", "Generic");
        self.next_id += 1;
        blinder.add_mode(FixtureMode::new(
            "1ch",
            vec![ChannelDef::new(ChannelType::Intensity, 0)],
        ));
        self.templates.push(blinder);
    }
}

/// Fixture instance with runtime state.
/// Represents a specific physical fixture in the DMX universe with its current
/// settings (color, position, etc.). References a template for channel layout.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Fixture {
    /// Unique identifier for this fixture instance
    pub id: u32,
    /// User-assigned name for this fixture
    pub name: String,
    /// Starting DMX channel (1-based). Channels for this fixture occupy
    /// start_channel through start_channel + mode_channels - 1.
    pub start_channel: usize,
    /// ID of the template this fixture uses
    pub template_id: u32,
    /// Index of the selected mode within the template
    pub mode_index: usize,
    /// The dimmer value
    pub intensity: u8,
    /// Current RGBW color values
    pub color: Color,
    /// Current pan position (0-255, maps to 0-540° typically)
    pub pan: u16,
    /// Current tilt position (0-255, maps to 0-270° typically)
    pub tilt: u16,
    /// Shutter/Strobe value (0 = open, 1-255 = strobe speed)
    pub shutter: u8,
    /// Current gobo selection
    pub gobo: u8,
    /// Current zoom position
    pub zoom: u8,
    /// Current focus position
    pub focus: u8,
    /// Custom channel values for undefined channel types (channel_offset -> value)
    pub custom_values: HashMap<usize, u8>,
}

impl Fixture {
    pub fn new(
        id: u32,
        name: String,
        start_channel: usize,
        template_id: u32,
        mode_index: usize,
    ) -> Self {
        Self {
            id,
            name,
            start_channel,
            template_id,
            mode_index,
            color: Color::default(),
            pan: 128,
            tilt: 128,
            shutter: Default::default(),
            gobo: Default::default(),
            zoom: 128,
            focus: 128,
            custom_values: HashMap::new(),
            intensity: Default::default(),
        }
    }

    pub fn get_dmx_values(&self, template: &FixtureTemplate) -> Vec<u8> {
        if let Some(mode) = template.get_mode(self.mode_index) {
            let mut values = vec![0u8; mode.total_channels()];

            for channel in &mode.channels {
                let value = match channel.channel_type {
                    ChannelType::Intensity => self.intensity,
                    ChannelType::Red => self.color.r,
                    ChannelType::Green => self.color.g,
                    ChannelType::Blue => self.color.b,
                    ChannelType::White => self.color.w,
                    ChannelType::Amber => self.color.amber,
                    ChannelType::UV => self.color.uv,
                    ChannelType::Pan => (self.pan >> 8) as u8,
                    ChannelType::PanFine => (self.pan & 0xFF) as u8,
                    ChannelType::Tilt => (self.tilt >> 8) as u8,
                    ChannelType::TiltFine => (self.tilt & 0xFF) as u8,
                    ChannelType::Shutter | ChannelType::Strobe => self.shutter,
                    ChannelType::GoboWheel => self.gobo,
                    ChannelType::Zoom => self.zoom,
                    ChannelType::Focus => self.focus,
                    _ => *self
                        .custom_values
                        .get(&(channel.offset as usize))
                        .unwrap_or(&0),
                };
                values[channel.offset as usize] = value;
            }
            values
        } else {
            Vec::new()
        }
    }
    pub fn get_fixture_as_buffer(
        &self,
        template: &FixtureTemplate,
    ) -> Vec<(ChannelType, DMXBufferValue)> {
        if let Some(mode) = template.get_mode(self.mode_index) {
            let mut values = Vec::new();

            for chan_def in &mode.channels {
                let value = match chan_def.channel_type {
                    ChannelType::Intensity => self.intensity,
                    ChannelType::Red => self.color.r,
                    ChannelType::Green => self.color.g,
                    ChannelType::Blue => self.color.b,
                    ChannelType::White => self.color.w,
                    ChannelType::Amber => self.color.amber,
                    ChannelType::UV => self.color.uv,
                    ChannelType::Pan => (self.pan >> 8) as u8,
                    ChannelType::PanFine => (self.pan & 0xFF) as u8,
                    ChannelType::Tilt => (self.tilt >> 8) as u8,
                    ChannelType::TiltFine => (self.tilt & 0xFF) as u8,
                    ChannelType::Shutter | ChannelType::Strobe => self.shutter,
                    ChannelType::GoboWheel => self.gobo,
                    ChannelType::Zoom => self.zoom,
                    ChannelType::Focus => self.focus,
                    _ => *self
                        .custom_values
                        .get(&(chan_def.offset as usize))
                        .unwrap_or(&0),
                };

                let dmx_chan = self.start_channel + chan_def.offset as usize;
                values.push((chan_def.channel_type, DMXBufferValue::new(dmx_chan, value)));
            }

            values
        } else {
            Vec::new()
        }
    }
}

/// Fixture group for collective control.
/// Allows multiple fixtures to be controlled as a single unit.
/// Useful for treating multiple PARs as one "unit" for patching or control.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FixtureGroup {
    /// Unique identifier for this group
    pub id: u32,
    /// User-assigned name for this group
    pub name: String,
    /// List of fixture IDs that belong to this group
    pub fixture_ids: Vec<u32>,
    /// Optional grid position for visual representation
    pub grid_index: Option<usize>,
}

impl FixtureGroup {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            fixture_ids: Vec::new(),
            grid_index: None,
        }
    }
}

/// Audio playback action for show control.
/// Defines how audio tracks behave in relation to the show/sequence.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum AudioAction {
    /// No special audio action - track plays normally
    None,
    /// Follow mode - audio starts when show starts
    Follow,
    /// Continue mode - audio continues across cues
    Continue,
}

/// Audio track for show control.
/// Represents an audio file that can be played during a light show,
/// typically used for music or sound effects.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AudioTrack {
    /// Unique identifier for this track
    pub id: u32,
    /// Display name for the track
    pub name: String,
    /// File path to the audio file
    pub file_path: String,
    /// Fade in duration in seconds
    pub fade_in: f32,
    /// Fade out duration in seconds
    pub fade_out: f32,
    /// Starting position in the track (in seconds)
    pub start_point: f32,
    /// Optional end position for looping/clipped playback
    pub end_point: Option<f32>,
    /// Volume level (0.0 - 1.0)
    pub volume: f32,
    /// Total duration of the track in seconds
    pub duration: f32,
    /// Audio action behavior
    pub action: AudioAction,
}

impl AudioTrack {
    pub fn new(id: u32, name: String, file_path: String) -> Self {
        Self {
            id,
            name,
            file_path,
            fade_in: 0.0,
            fade_out: 0.0,
            start_point: 0.0,
            end_point: None,
            volume: 1.0,
            duration: 0.0,
            action: AudioAction::None,
        }
    }
}

/// Represents a cue containing DMX values and timing information.
/// A cue is a snapshot of all DMX channel values that can be recalled
/// and played back through an executor. Supports fade times for smooth transitions.
#[derive(Clone)]
pub struct Cue {
    /// Unique identifier for the cue
    pub id: u32,
    /// Human-readable name of the cue
    pub name: String,
    /// Fade time in seconds (how long to transition to this cue)
    pub fade_time: f32,
    /// Delay time in seconds before starting the fade
    pub delay: f32,
    /// DMX channel values (512 channels, index 0 = channel 1)
    pub levels: Vec<u8>,
}

impl Cue {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            name: format!("Cue {}", id),
            fade_time: 0.0,
            delay: 0.0,
            levels: vec![0; DMX_CHANNELS],
        }
    }
}

/// Represents a single DMX channel value in the buffer.
/// Used for the temporary buffer that holds values before storing to a cue,
/// or for direct channel manipulation commands.
#[derive(PartialEq, Clone, Debug)]
pub struct DMXBufferValue {
    /// DMX channel number (1-based, 1-512)
    pub chan: usize,
    /// DMX value (0-255)
    pub dmx: u8,
}

impl DMXBufferValue {
    pub fn new(chan: usize, val: u8) -> Self {
        Self { chan, dmx: val }
    }
}
/// Represents an executor that controls playback of cues with a fader.
/// Executors are the playback section of a lighting console - each has:
/// - A list of cues that can be stepped through
/// - A fader for intensity control
/// - GO/BACK buttons for cue advancement
/// - Fade engine for smooth transitions between cues
pub struct Executor {
    /// Index of this executor (0-based)
    pub id: u32,
    /// Currently active cue ID (if any)
    pub current_cue: Option<u32>,
    /// Index of the current cue in the cue_list
    pub current_cue_index: usize,
    /// Whether the executor is currently playing (not currently used)
    pub is_running: bool,
    /// List of cues stored in this executor
    pub cue_list: Vec<Cue>,
    /// Fader position (0.0 to 1.0) - controls output intensity
    pub fader_level: f32,
    /// DMX values from the current cue (cached for mixing)
    pub stored_channels: Vec<u8>,
    /// Target fader level for fade transitions
    pub target_level: f32,
    /// Current output level (used during fade interpolation)
    pub current_output_level: f32,
    /// Timestamp when fade started (for interpolation)
    pub fade_start_time: f64,
    /// Whether a fade is currently in progress
    pub is_fading: bool,
    /// Last fader level (for detecting fader movements)
    pub last_fader_level: f32,
}

impl Executor {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            current_cue: None,
            current_cue_index: Default::default(),
            is_running: Default::default(),
            cue_list: Default::default(),
            fader_level: Default::default(),
            stored_channels: vec![0; DMX_CHANNELS],
            target_level: Default::default(),
            current_output_level: Default::default(),
            fade_start_time: Default::default(),
            is_fading: Default::default(),
            last_fader_level: Default::default(),
        }
    }

    pub fn go(&mut self) {
        if self.cue_list.is_empty() {
            return;
        }
        self.current_cue_index = self.current_cue_index.saturating_add(1) % self.cue_list.len();
        self.current_cue = Some(self.cue_list[self.current_cue_index].id);
        self.stored_channels = self.cue_list[self.current_cue_index].levels.clone();
        self.target_level = self.fader_level;
        self.is_fading = true;
        self.fade_start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
    }

    pub fn go_back(&mut self) {
        if self.cue_list.is_empty() {
            return;
        }
        self.current_cue_index =
            (self.cue_list.len() + self.current_cue_index - 1) % self.cue_list.len();
        self.current_cue = Some(self.cue_list[self.current_cue_index].id);
        self.stored_channels = self.cue_list[self.current_cue_index].levels.clone();
        self.target_level = self.fader_level;
        self.is_fading = true;
        self.fade_start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
    }

    pub fn update_fade(&mut self) {
        if self.last_fader_level == 0.0 && self.fader_level != 0.0 {
            self.target_level = 1.0;
            self.is_fading = true;
            self.fade_start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
        }
        self.last_fader_level = self.fader_level;
        if !self.is_fading || self.cue_list.is_empty() {
            self.current_output_level = self.fader_level;
            return;
        }

        let current_cue = &self.cue_list[self.current_cue_index];
        let fade_time = current_cue.fade_time;
        if fade_time <= 0.0 {
            self.current_output_level = self.fader_level;
            self.is_fading = false;
            return;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let elapsed = now - self.fade_start_time;
        let progress = (elapsed / fade_time as f64).min(1.0) as f32;

        self.current_output_level = progress * self.fader_level;

        if progress >= 1.0 {
            self.is_fading = false;
            self.current_output_level = self.fader_level;
        }
    }
}

// Re-export commonly used types
pub use ChannelType as Ch;

pub fn ch(channel_type: ChannelType, offset: u8) -> ChannelDef {
    ChannelDef::new(channel_type, offset)
}
