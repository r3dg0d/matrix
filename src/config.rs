//! XDG config loading (`~/.config/matrix/config.toml`).

use crate::charset::CharsetId;
use crate::palette::PaletteId;
use crate::presets::PresetId;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
        Self::load_from(Self::config_path())
    }

    /// Load `path`. A missing file is the first-run case and uses defaults.
    /// A file that exists but cannot be read or parsed is an error.
    pub fn load_from(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let cfg: FileConfig =
            toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;
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

    /// `Ok(None)` when the file omits `color`. An unknown name is an error,
    /// same as `--color`.
    pub fn resolved_palette(&self) -> anyhow::Result<Option<PaletteId>> {
        match self.color.as_deref() {
            None => Ok(None),
            Some(name) => PaletteId::parse(name)
                .map(Some)
                .ok_or_else(|| anyhow::anyhow!("unknown palette '{name}' (try --list-palettes)")),
        }
    }

    /// `Ok(None)` when the file omits `charset`. An unknown name is an error,
    /// same as `--charset`.
    pub fn resolved_charset(&self) -> anyhow::Result<Option<CharsetId>> {
        match self.charset.as_deref() {
            None => Ok(None),
            Some(name) => CharsetId::parse(name)
                .map(Some)
                .ok_or_else(|| anyhow::anyhow!("unknown charset '{name}' (try --list-charsets)")),
        }
    }

    pub fn resolved_preset(&self) -> Option<PresetId> {
        self.preset.as_deref().and_then(PresetId::parse)
    }
}

/// Flags from the command line. `None` means that flag was not passed.
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub speed: Option<f32>,
    pub density: Option<f32>,
    pub fps: Option<u32>,
    pub color: Option<String>,
    pub preset: Option<String>,
    pub charset: Option<String>,
    pub chars: Option<String>,
    pub trail: Option<f32>,
    pub truecolor: Option<bool>,
}

/// Merge defaults, then the config file, then the CLI.
///
/// Within each layer a preset is applied first and individual knobs overlay it.
/// A CLI preset therefore replaces the whole bundle, including knobs written
/// into `config.toml` (the example seeds every knob). Pass the flag again to
/// override one field of that preset.
pub fn resolve_settings(file_cfg: &FileConfig, cli: &CliOverrides) -> anyhow::Result<Settings> {
    let mut settings = Settings::default();

    if let Some(p) = file_cfg.resolved_preset() {
        apply_preset_settings(&mut settings, p);
    }
    apply_file_knobs(&mut settings, file_cfg)?;

    if let Some(ref name) = cli.preset {
        let p = PresetId::parse(name)
            .ok_or_else(|| anyhow::anyhow!("unknown preset '{name}' (try --list-presets)"))?;
        apply_preset_settings(&mut settings, p);
    }
    apply_cli_knobs(&mut settings, cli)?;

    settings.clamp();
    Ok(settings)
}

fn apply_preset_settings(settings: &mut Settings, preset: PresetId) {
    let p = preset.apply();
    settings.preset = Some(preset);
    settings.speed = p.speed;
    settings.density = p.density;
    settings.fps = p.fps;
    settings.charset = p.charset;
    settings.palette = p.palette;
    settings.trail = p.trail;
    settings.custom_chars = None;
}

fn apply_file_knobs(settings: &mut Settings, file_cfg: &FileConfig) -> anyhow::Result<()> {
    if let Some(v) = file_cfg.speed {
        settings.speed = v;
    }
    if let Some(v) = file_cfg.density {
        settings.density = v;
    }
    if let Some(v) = file_cfg.fps {
        settings.fps = v;
    }
    if let Some(v) = file_cfg.trail {
        settings.trail = v;
    }
    if let Some(p) = file_cfg.resolved_palette()? {
        settings.palette = p;
    }
    if let Some(c) = file_cfg.resolved_charset()? {
        settings.charset = c;
    }
    if let Some(ref ch) = file_cfg.chars {
        settings.custom_chars = Some(ch.clone());
        settings.charset = CharsetId::Custom;
    }
    if let Some(v) = file_cfg.truecolor {
        settings.truecolor = v;
    }
    Ok(())
}

fn apply_cli_knobs(settings: &mut Settings, cli: &CliOverrides) -> anyhow::Result<()> {
    if let Some(v) = cli.speed {
        settings.speed = v;
    }
    if let Some(v) = cli.density {
        settings.density = v;
    }
    if let Some(v) = cli.fps {
        settings.fps = v;
    }
    if let Some(v) = cli.trail {
        settings.trail = v;
    }
    if let Some(ref name) = cli.color {
        settings.palette = PaletteId::parse(name)
            .ok_or_else(|| anyhow::anyhow!("unknown palette '{name}' (try --list-palettes)"))?;
    }
    if let Some(ref name) = cli.charset {
        settings.charset = CharsetId::parse(name)
            .ok_or_else(|| anyhow::anyhow!("unknown charset '{name}' (try --list-charsets)"))?;
        settings.custom_chars = None;
    }
    if let Some(ref ch) = cli.chars {
        settings.custom_chars = Some(ch.clone());
        settings.charset = CharsetId::Custom;
    }
    if let Some(v) = cli.truecolor {
        settings.truecolor = v;
    }
    Ok(())
}

/// Runtime settings after merging CLI over config over preset over defaults.
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
        assert_eq!(cfg.resolved_palette().unwrap(), Some(PaletteId::Matrix));
        assert_eq!(cfg.resolved_charset().unwrap(), Some(CharsetId::Classic));
    }

    #[test]
    fn cli_preset_overrides_seeded_config_knobs() {
        // First run writes example knobs for every field. Those must not undo --preset.
        let file: FileConfig = toml::from_str(FileConfig::example_toml()).unwrap();
        let cli = CliOverrides {
            preset: Some("slow".into()),
            ..CliOverrides::default()
        };
        let s = resolve_settings(&file, &cli).unwrap();
        let slow = PresetId::Slow.apply();
        assert_eq!(s.preset, Some(PresetId::Slow));
        assert_eq!(s.speed, slow.speed);
        assert_eq!(s.density, slow.density);
        assert_eq!(s.fps, slow.fps);
        assert_eq!(s.palette, slow.palette);
        assert_eq!(s.charset, slow.charset);
        assert_eq!(s.trail, slow.trail);
        assert!(s.custom_chars.is_none());
    }

    #[test]
    fn cli_knob_still_overrides_cli_preset() {
        let cli = CliOverrides {
            preset: Some("slow".into()),
            speed: Some(2.0),
            ..CliOverrides::default()
        };
        let s = resolve_settings(&FileConfig::default(), &cli).unwrap();
        assert_eq!(s.speed, 2.0);
        assert_eq!(s.palette, PaletteId::Amber);
        assert_eq!(s.fps, PresetId::Slow.apply().fps);
    }

    #[test]
    fn file_knob_overrides_file_preset_only() {
        let file = FileConfig {
            preset: Some("slow".into()),
            speed: Some(2.0),
            ..FileConfig::default()
        };
        let bare = resolve_settings(&file, &CliOverrides::default()).unwrap();
        assert_eq!(bare.speed, 2.0);
        assert_eq!(bare.palette, PaletteId::Amber);

        let cli = CliOverrides {
            preset: Some("nixos".into()),
            ..CliOverrides::default()
        };
        let s = resolve_settings(&file, &cli).unwrap();
        let nix = PresetId::Nixos.apply();
        assert_eq!(s.speed, nix.speed);
        assert_eq!(s.palette, nix.palette);
        assert_eq!(s.charset, nix.charset);
    }

    #[test]
    fn cli_preset_drops_file_custom_chars() {
        let file = FileConfig {
            chars: Some("ABC".into()),
            ..FileConfig::default()
        };
        let kept = resolve_settings(&file, &CliOverrides::default()).unwrap();
        assert_eq!(kept.custom_chars.as_deref(), Some("ABC"));
        assert_eq!(kept.charset, CharsetId::Custom);

        let cli = CliOverrides {
            preset: Some("nixos".into()),
            ..CliOverrides::default()
        };
        let s = resolve_settings(&file, &cli).unwrap();
        assert!(s.custom_chars.is_none());
        assert_eq!(s.charset, CharsetId::Nix);
    }

    #[test]
    fn unknown_cli_preset_errors() {
        let cli = CliOverrides {
            preset: Some("nope".into()),
            ..CliOverrides::default()
        };
        let err = resolve_settings(&FileConfig::default(), &cli).unwrap_err();
        assert!(err.to_string().contains("unknown preset"));
    }

    #[test]
    fn missing_config_file_uses_defaults() {
        let path =
            std::env::temp_dir().join(format!("matrix-missing-config-{}.toml", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert!(!path.exists());
        let cfg = FileConfig::load_from(&path).unwrap();
        assert!(cfg.speed.is_none());
        assert!(cfg.color.is_none());
        assert!(cfg.charset.is_none());
        assert!(cfg.preset.is_none());
        assert!(cfg.chars.is_none());
    }

    #[test]
    fn broken_config_file_is_an_error() {
        let path =
            std::env::temp_dir().join(format!("matrix-broken-config-{}.toml", std::process::id()));
        std::fs::write(&path, "speed = [\n").unwrap();
        let err = FileConfig::load_from(&path).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("failed to parse"), "{msg}");
        assert!(msg.contains(&path.display().to_string()), "{msg}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn valid_config_file_loads() {
        let path =
            std::env::temp_dir().join(format!("matrix-valid-config-{}.toml", std::process::id()));
        std::fs::write(&path, "color = \"amber\"\ncharset = \"hex\"\n").unwrap();
        let cfg = FileConfig::load_from(&path).unwrap();
        assert_eq!(cfg.resolved_palette().unwrap(), Some(PaletteId::Amber));
        assert_eq!(cfg.resolved_charset().unwrap(), Some(CharsetId::Hex));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unknown_file_palette_errors() {
        let file = FileConfig {
            color: Some("chartreuse".into()),
            ..FileConfig::default()
        };
        let err = file.resolved_palette().unwrap_err();
        assert!(err.to_string().contains("unknown palette 'chartreuse'"));
        let err = resolve_settings(&file, &CliOverrides::default()).unwrap_err();
        assert!(err.to_string().contains("unknown palette 'chartreuse'"));
    }

    #[test]
    fn unknown_file_charset_errors() {
        let file = FileConfig {
            charset: Some("emoji".into()),
            ..FileConfig::default()
        };
        let err = file.resolved_charset().unwrap_err();
        assert!(err.to_string().contains("unknown charset 'emoji'"));
        let err = resolve_settings(&file, &CliOverrides::default()).unwrap_err();
        assert!(err.to_string().contains("unknown charset 'emoji'"));
    }

    #[test]
    fn omitted_file_palette_and_charset_stay_default() {
        let bare = FileConfig::default();
        assert!(bare.resolved_palette().unwrap().is_none());
        assert!(bare.resolved_charset().unwrap().is_none());
        let s = resolve_settings(&bare, &CliOverrides::default()).unwrap();
        assert_eq!(s.palette, PaletteId::Matrix);
        assert_eq!(s.charset, CharsetId::Classic);
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
