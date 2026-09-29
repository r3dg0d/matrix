//! XDG config loading (`~/.config/matrix/config.toml`).

use crate::charset::CharsetId;
use crate::palette::PaletteId;
use crate::presets::PresetId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FileConfig {
    pub speed: Option<f32>,
    pub density: Option<f32>,
    pub fps: Option<u32>,
    pub color: Option<String>,
    pub charset: Option<String>,
    pub chars: Option<String>,
    pub preset: Option<String>,
    pub truecolor: Option<bool>,
    pub trail: Option<f32>,
}

impl FileConfig {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".").join(".config"))
            .join("matrix")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn example_path() -> PathBuf {
        Self::config_dir().join("config.example.toml")
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)?;
        let cfg: FileConfig = toml::from_str(&text)?;
        Ok(cfg)
    }

    pub fn example_toml() -> &'static str {
        r#"# matrix digital rain — example config
# Copy to ~/.config/matrix/config.toml and tweak.

# Fall speed multiplier (0.1 .. 5.0)
speed = 1.0

# Column density (0.05 .. 1.0) — fraction of terminal width used as active columns
density = 0.75

# Frame rate cap
fps = 30

# Palette: matrix | amber | cyberblue | red | purple | mono
color = "matrix"

# Charset: classic | ascii | binary | hex | katakana | nix
charset = "classic"

# Or provide a custom glyph string (overrides charset)
# chars = "01NEO"

# Named preset: hacker | screensaver | dense | sparse | slow | hyperspeed | nixos
# preset = "hacker"

# Force truecolor (auto-detect by default)
# truecolor = true

# Trail length factor (0.1 .. 1.0)
trail = 0.55
"#
    }

    pub fn ensure_example() -> anyhow::Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let example = Self::example_path();
        if !example.exists() {
            std::fs::write(&example, Self::example_toml())?;
        }
        // Also seed config.toml if missing so first run has something to edit.
        let cfg = Self::config_path();
        if !cfg.exists() {
            std::fs::write(&cfg, Self::example_toml())?;
        }
        Ok(())
    }

    pub fn resolved_palette(&self) -> Option<PaletteId> {
        self.color.as_deref().and_then(PaletteId::parse)
    }

    pub fn resolved_charset(&self) -> Option<CharsetId> {
        self.charset.as_deref().and_then(CharsetId::parse)
    }

    pub fn resolved_preset(&self) -> Option<PresetId> {
        self.preset.as_deref().and_then(PresetId::parse)
    }
}

/// Runtime settings after merging CLI > config > preset > defaults.
#[derive(Debug, Clone)]
pub struct Settings {
    pub speed: f32,
    pub density: f32,
    pub fps: u32,
    pub palette: PaletteId,
    pub charset: CharsetId,
    pub custom_chars: Option<String>,
    pub preset: Option<PresetId>,
    pub truecolor: bool,
    pub trail: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            speed: 1.0,
            density: 0.75,
            fps: 30,
            palette: PaletteId::Matrix,
            charset: CharsetId::Classic,
            custom_chars: None,
            preset: None,
            truecolor: detect_truecolor(),
            trail: 0.55,
        }
    }
}

impl Settings {
    pub fn clamp(&mut self) {
        self.speed = self.speed.clamp(0.05, 8.0);
        self.density = self.density.clamp(0.05, 1.0);
        self.fps = self.fps.clamp(5, 120);
        self.trail = self.trail.clamp(0.1, 1.0);
    }
}

pub fn detect_truecolor() -> bool {
    if let Ok(c) = std::env::var("COLORTERM") {
        let c = c.to_ascii_lowercase();
        if c.contains("truecolor") || c.contains("24bit") {
            return true;
        }
    }
    if let Ok(t) = std::env::var("TERM") {
        let t = t.to_ascii_lowercase();
        if t.contains("truecolor") || t.contains("24bit") || t.contains("direct") {
            return true;
        }
    }
    // Most modern terminals; crossterm will still accept AnsiValue fallback if we force false.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example_toml() {
        let cfg: FileConfig = toml::from_str(FileConfig::example_toml()).unwrap();
        assert_eq!(cfg.speed, Some(1.0));
        assert_eq!(cfg.color.as_deref(), Some("matrix"));
        assert_eq!(cfg.resolved_palette(), Some(PaletteId::Matrix));
        assert_eq!(cfg.resolved_charset(), Some(CharsetId::Classic));
    }

    #[test]
    fn settings_clamp() {
        let mut s = Settings {
            speed: 100.0,
            density: 2.0,
            fps: 1000,
            trail: 5.0,
            ..Default::default()
        };
        s.clamp();
        assert!(s.speed <= 8.0);
        assert!(s.density <= 1.0);
        assert!(s.fps <= 120);
        assert!(s.trail <= 1.0);
    }
}
