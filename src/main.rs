//! matrix — realtime Matrix-style digital rain for the terminal.

mod app;
mod charset;
mod column;
mod config;
mod input;
mod palette;
mod presets;
mod renderer;

use charset::CharsetId;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use config::{FileConfig, Settings};
use palette::PaletteId;
use presets::PresetId;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(
    name = "matrix",
    version = VERSION,
    about = "Realtime Matrix-style digital rain for the terminal",
    long_about = "Launch a polished Matrix-style digital rain animation in your terminal.\n\n\
Keys: q quit · space pause · +/- speed · [ ] density · c palette · p preset · ? help\n\n\
Config: ~/.config/matrix/config.toml (see config.example.toml)",
    after_help = "Presets: hacker, screensaver, dense, sparse, slow, hyperspeed, nixos\n\
Palettes: matrix, amber, cyberblue, red, purple, mono\n\
Charsets: classic, ascii, binary, hex, katakana, nix"
)]
struct Cli {
    /// Fall speed multiplier (0.05–8)
    #[arg(long, short = 's', value_name = "FACTOR")]
    speed: Option<f32>,

    /// Column density as fraction of width (0.05–1)
    #[arg(long, short = 'd', value_name = "FRAC")]
    density: Option<f32>,

    /// Frame rate cap
    #[arg(long, short = 'f', value_name = "FPS")]
    fps: Option<u32>,

    /// Color palette
    #[arg(long, short = 'c', value_name = "NAME")]
    color: Option<String>,

    /// Named preset
    #[arg(long, short = 'p', value_name = "NAME")]
    preset: Option<String>,

    /// Glyph charset
    #[arg(long, value_name = "NAME")]
    charset: Option<String>,

    /// Custom glyph string (overrides --charset)
    #[arg(long, value_name = "CHARS")]
    chars: Option<String>,

    /// Trail length factor (0.1–1)
    #[arg(long, value_name = "FRAC")]
    trail: Option<f32>,

    /// Force truecolor on/off
    #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
    truecolor: Option<bool>,

    /// List available presets and exit
    #[arg(long)]
    list_presets: bool,

    /// List available charsets and exit
    #[arg(long)]
    list_charsets: bool,

    /// List available palettes and exit
    #[arg(long)]
    list_palettes: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate shell completions to stdout
    Completions {
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    Powershell,
    Zsh,
}

impl From<CompletionShell> for Shell {
    fn from(s: CompletionShell) -> Self {
        match s {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Elvish => Shell::Elvish,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::Powershell => Shell::PowerShell,
            CompletionShell::Zsh => Shell::Zsh,
        }
    }
}

fn main() {
    if let Err(e) = real_main() {
        // Avoid wrecking the terminal if we already left alt screen.
        eprintln!("matrix: {e:#}");
        std::process::exit(1);
    }
}

fn real_main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Completions { shell }) = cli.command {
        let mut cmd = Cli::command();
        generate(Shell::from(shell), &mut cmd, "matrix", &mut io::stdout());
        return Ok(());
    }

    if cli.list_presets {
        println!("Available presets:");
        for p in PresetId::all() {
            println!("  {:12}  {}", p.as_str(), p.description());
        }
        return Ok(());
    }
    if cli.list_charsets {
        println!("Available charsets:");
        for c in CharsetId::all() {
            println!("  {:12}  {}", c.as_str(), c.description());
        }
        println!("  {:12}  via --chars \"...\"", "custom");
        return Ok(());
    }
    if cli.list_palettes {
        println!("Available palettes:");
        for p in PaletteId::all() {
            println!("  {:12}  {}", p.as_str(), p.description());
        }
        return Ok(());
    }

    // Ensure example + default config exist (non-fatal).
    let _ = FileConfig::ensure_example();
    let file_cfg = FileConfig::load().unwrap_or_default();

    let mut settings = Settings::default();

    // Preset first (file then CLI), then overlay individual knobs.
    if let Some(p) = file_cfg.resolved_preset() {
        apply_preset_settings(&mut settings, p);
    }
    if let Some(ref name) = cli.preset {
        let p = PresetId::parse(name)
            .ok_or_else(|| anyhow::anyhow!("unknown preset '{name}' (try --list-presets)"))?;
        apply_preset_settings(&mut settings, p);
    }

    // File config knobs
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
    if let Some(p) = file_cfg.resolved_palette() {
        settings.palette = p;
    }
    if let Some(c) = file_cfg.resolved_charset() {
        settings.charset = c;
    }
    if let Some(ref ch) = file_cfg.chars {
        settings.custom_chars = Some(ch.clone());
        settings.charset = CharsetId::Custom;
    }
    if let Some(v) = file_cfg.truecolor {
        settings.truecolor = v;
    }

    // CLI overrides
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

    settings.clamp();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    app::run(settings, running)
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
