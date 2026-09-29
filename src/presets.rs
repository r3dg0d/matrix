//! Named visual presets.

use crate::charset::CharsetId;
use crate::palette::PaletteId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PresetId {
    Hacker,
    Screensaver,
    Dense,
    Sparse,
    Slow,
    Hyperspeed,
    Nixos,
}

impl PresetId {
    pub fn all() -> &'static [PresetId] {
        &[
            PresetId::Hacker,
            PresetId::Screensaver,
            PresetId::Dense,
            PresetId::Sparse,
            PresetId::Slow,
            PresetId::Hyperspeed,
            PresetId::Nixos,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PresetId::Hacker => "hacker",
            PresetId::Screensaver => "screensaver",
            PresetId::Dense => "dense",
            PresetId::Sparse => "sparse",
            PresetId::Slow => "slow",
            PresetId::Hyperspeed => "hyperspeed",
            PresetId::Nixos => "nixos",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "hacker" | "default" => Some(PresetId::Hacker),
            "screensaver" | "idle" => Some(PresetId::Screensaver),
            "dense" | "heavy" => Some(PresetId::Dense),
            "sparse" | "light" => Some(PresetId::Sparse),
            "slow" | "chill" => Some(PresetId::Slow),
            "hyperspeed" | "fast" | "warp" => Some(PresetId::Hyperspeed),
            "nixos" | "nix" => Some(PresetId::Nixos),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        let all = Self::all();
        let i = all.iter().position(|p| *p == self).unwrap_or(0);
        all[(i + 1) % all.len()]
    }

    pub fn description(self) -> &'static str {
        match self {
            PresetId::Hacker => "Balanced classic green rain",
            PresetId::Screensaver => "Calm, sparse, lower FPS",
            PresetId::Dense => "High column density",
            PresetId::Sparse => "Few columns, longer trails",
            PresetId::Slow => "Leisurely fall speed",
            PresetId::Hyperspeed => "Max speed, high FPS",
            PresetId::Nixos => "Nix charset + cyberblue",
        }
    }

    pub fn apply(self) -> PresetSettings {
        match self {
            PresetId::Hacker => PresetSettings {
                speed: 1.0,
                density: 0.75,
                fps: 30,
                charset: CharsetId::Classic,
                palette: PaletteId::Matrix,
                trail: 0.55,
            },
            PresetId::Screensaver => PresetSettings {
                speed: 0.55,
                density: 0.35,
                fps: 20,
                charset: CharsetId::Classic,
                palette: PaletteId::Matrix,
                trail: 0.7,
            },
            PresetId::Dense => PresetSettings {
                speed: 1.1,
                density: 1.0,
                fps: 30,
                charset: CharsetId::Classic,
                palette: PaletteId::Matrix,
                trail: 0.45,
            },
            PresetId::Sparse => PresetSettings {
                speed: 0.9,
                density: 0.25,
                fps: 30,
                charset: CharsetId::Katakana,
                palette: PaletteId::Matrix,
                trail: 0.85,
            },
            PresetId::Slow => PresetSettings {
                speed: 0.35,
                density: 0.6,
                fps: 24,
                charset: CharsetId::Classic,
                palette: PaletteId::Amber,
                trail: 0.75,
            },
            PresetId::Hyperspeed => PresetSettings {
                speed: 2.5,
                density: 0.85,
                fps: 60,
                charset: CharsetId::Binary,
                palette: PaletteId::Cyberblue,
                trail: 0.35,
            },
            PresetId::Nixos => PresetSettings {
                speed: 1.0,
                density: 0.7,
                fps: 30,
                charset: CharsetId::Nix,
                palette: PaletteId::Cyberblue,
                trail: 0.55,
            },
        }
    }
}

impl std::fmt::Display for PresetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct PresetSettings {
    pub speed: f32,
    pub density: f32,
    pub fps: u32,
    pub charset: CharsetId,
    pub palette: PaletteId,
    /// Relative trail length (0..1).
    pub trail: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_presets_parse() {
        for p in PresetId::all() {
            assert_eq!(PresetId::parse(p.as_str()), Some(*p));
            let s = p.apply();
            assert!(s.speed > 0.0);
            assert!(s.density > 0.0 && s.density <= 1.0);
            assert!(s.fps > 0);
        }
    }
}
