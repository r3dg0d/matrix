# matrix

Realtime **Matrix-style digital rain** for your terminal.

Polished Rust CLI using [crossterm](https://crates.io/crates/crossterm): alternate screen, hidden cursor, responsive resize, truecolor palettes with ANSI-256 fallback, live controls, XDG config, Nix flake, and shell completions.

> Glyph sets are **original** (halfwidth-kana-*inspired*, digits, symbols). Not a rip of any film assets.

## Install

```bash
# From source
cargo install --path . --root ~/.local

# Or copy the release binary
cargo build --release
install -Dm755 target/release/matrix ~/.local/bin/matrix
```

Ensure `~/.local/bin` is on your `PATH`.

### Nix

```bash
nix build
./result/bin/matrix

# Direnv / nix develop
nix develop -c cargo run --release
```

Optional system package (NixOS): add this flake's `packages.<system>.default` to `environment.systemPackages`, or overlay into your config later.

## Usage

```bash
matrix                     # classic green rain
matrix --preset nixos
matrix --color amber --charset katakana --speed 0.6
matrix --chars "01NEO" --density 0.4 --fps 60
matrix --list-presets
matrix --list-charsets
matrix --list-palettes
matrix --help
matrix --version
```

### Live keys

| Key | Action |
|-----|--------|
| `q` / Ctrl+C | Quit (restores terminal) |
| `space` | Pause / resume |
| `+` / `-` | Speed up / slow down |
| `]` / `[` | Density up / down |
| `c` | Cycle palette |
| `p` | Cycle preset |
| `?` / `h` | Help overlay |

### Presets

`hacker` · `screensaver` · `dense` · `sparse` · `slow` · `hyperspeed` · `nixos`

### Palettes

`matrix` (default) · `amber` · `cyberblue` · `red` · `purple` · `mono`

### Charsets

`classic` · `ascii` · `binary` · `hex` · `katakana` · `nix` · custom via `--chars`

## Config

XDG path: `~/.config/matrix/config.toml`

An example is written on first run (`config.example.toml` + seeded `config.toml`).

CLI flags override config. Preset (if set) applies first, then individual knobs.

## Completions

```bash
matrix completions bash > ~/.local/share/bash-completion/completions/matrix
matrix completions zsh  > ~/.zsh/completions/_matrix
matrix completions fish > ~/.config/fish/completions/matrix.fish
```

Pre-generated stubs also live under `completions/`.

## Man page

See `man/matrix.1`. After install:

```bash
man man/matrix.1
# or
mkdir -p ~/.local/share/man/man1 && cp man/matrix.1 ~/.local/share/man/man1/
```

## Binary name

The binary is `matrix`. If your environment already has a Matrix-protocol client named `matrix`, build with `--bin` rename or install as `matrix-rain` and alias.

## License

MIT — see [LICENSE](LICENSE).
