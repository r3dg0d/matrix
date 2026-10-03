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
use config::{resolve_settings, CliOverrides, FileConfig};
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
    let settings = resolve_settings(
        &file_cfg,
        &CliOverrides {
            speed: cli.speed,
            density: cli.density,
            fps: cli.fps,
            color: cli.color.clone(),
            preset: cli.preset.clone(),
            charset: cli.charset.clone(),
            chars: cli.chars.clone(),
            trail: cli.trail,
            truecolor: cli.truecolor,
        },
    )?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    app::run(settings, running)
}
