//! Color palettes with truecolor and ANSI-256 fallback.

use crossterm::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaletteId {
    Matrix,
    Amber,
    Cyberblue,
    Red,
    Purple,
    Mono,
}

impl PaletteId {
    pub fn all() -> &'static [PaletteId] {
        &[
            PaletteId::Matrix,
            PaletteId::Amber,
            PaletteId::Cyberblue,
            PaletteId::Red,
            PaletteId::Purple,
            PaletteId::Mono,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PaletteId::Matrix => "matrix",
            PaletteId::Amber => "amber",
            PaletteId::Cyberblue => "cyberblue",
            PaletteId::Red => "red",
            PaletteId::Purple => "purple",
            PaletteId::Mono => "mono",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "matrix" | "green" | "default" => Some(PaletteId::Matrix),
            "amber" | "orange" | "crt" => Some(PaletteId::Amber),
            "cyberblue" | "blue" | "cyan" => Some(PaletteId::Cyberblue),
            "red" | "crimson" => Some(PaletteId::Red),
            "purple" | "violet" | "magenta" => Some(PaletteId::Purple),
            "mono" | "white" | "bw" | "grayscale" => Some(PaletteId::Mono),
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
            PaletteId::Matrix => "Classic green digital rain",
            PaletteId::Amber => "Warm CRT amber",
            PaletteId::Cyberblue => "Cyan / electric blue",
            PaletteId::Red => "Crimson trails",
            PaletteId::Purple => "Violet neon",
            PaletteId::Mono => "Grayscale / white",
        }
    }
}

impl std::fmt::Display for PaletteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_color(self, truecolor: bool) -> Color {
        if truecolor {
            Color::Rgb {
                r: self.r,
                g: self.g,
                b: self.b,
            }
        } else {
            Color::AnsiValue(rgb_to_ansi256(self.r, self.g, self.b))
        }
    }

    pub fn scale(self, factor: f32) -> Self {
        let f = factor.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 * f) as u8,
            g: (self.g as f32 * f) as u8,
            b: (self.b as f32 * f) as u8,
        }
    }

    #[allow(dead_code)]
    pub fn brighten(self, amount: f32) -> Self {
        let a = amount.clamp(0.0, 1.0);
        Self {
            r: ((self.r as f32) + (255.0 - self.r as f32) * a).min(255.0) as u8,
            g: ((self.g as f32) + (255.0 - self.g as f32) * a).min(255.0) as u8,
            b: ((self.b as f32) + (255.0 - self.b as f32) * a).min(255.0) as u8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Palette {
    #[allow(dead_code)]
    pub id: PaletteId,
    pub lead: Rgb,
    pub bright: Rgb,
    pub mid: Rgb,
    pub dim: Rgb,
    pub bg: Rgb,
}

impl Palette {
    pub fn from_id(id: PaletteId) -> Self {
        match id {
            PaletteId::Matrix => Self {
                id,
                lead: Rgb::new(220, 255, 220),
                bright: Rgb::new(0, 255, 70),
                mid: Rgb::new(0, 180, 40),
                dim: Rgb::new(0, 80, 20),
                bg: Rgb::new(0, 0, 0),
            },
            PaletteId::Amber => Self {
                id,
                lead: Rgb::new(255, 240, 200),
                bright: Rgb::new(255, 176, 0),
                mid: Rgb::new(200, 120, 0),
                dim: Rgb::new(100, 50, 0),
                bg: Rgb::new(0, 0, 0),
            },
            PaletteId::Cyberblue => Self {
                id,
                lead: Rgb::new(220, 255, 255),
                bright: Rgb::new(0, 220, 255),
                mid: Rgb::new(0, 140, 220),
                dim: Rgb::new(0, 50, 100),
                bg: Rgb::new(0, 0, 8),
            },
            PaletteId::Red => Self {
                id,
                lead: Rgb::new(255, 220, 220),
                bright: Rgb::new(255, 40, 40),
                mid: Rgb::new(180, 20, 20),
                dim: Rgb::new(80, 10, 10),
                bg: Rgb::new(0, 0, 0),
            },
            PaletteId::Purple => Self {
                id,
                lead: Rgb::new(245, 220, 255),
                bright: Rgb::new(200, 80, 255),
                mid: Rgb::new(140, 40, 200),
                dim: Rgb::new(60, 20, 90),
                bg: Rgb::new(0, 0, 0),
            },
            PaletteId::Mono => Self {
                id,
                lead: Rgb::new(255, 255, 255),
                bright: Rgb::new(220, 220, 220),
                mid: Rgb::new(140, 140, 140),
                dim: Rgb::new(60, 60, 60),
                bg: Rgb::new(0, 0, 0),
            },
        }
    }

    /// Map trail age fraction (0.0 = head/fresh, 1.0 = dead) to a color.
    pub fn trail_color(&self, age: f32, is_lead: bool, truecolor: bool) -> Color {
        if is_lead {
            return self.lead.to_color(truecolor);
        }
        let t = age.clamp(0.0, 1.0);
        let rgb = if t < 0.15 {
            lerp_rgb(self.bright, self.mid, t / 0.15)
        } else if t < 0.55 {
            lerp_rgb(self.mid, self.dim, (t - 0.15) / 0.40)
        } else {
            self.dim.scale(1.0 - ((t - 0.55) / 0.45))
        };
        rgb.to_color(truecolor)
    }
}

fn lerp_rgb(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    Rgb {
        r: (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
        g: (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
        b: (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
    }
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // grayscale fast-path
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return (232 + ((r as u16 - 8) * 24) / 247) as u8;
    }
    let ri = ((r as u16 * 5) / 255) as u8;
    let gi = ((g as u16 * 5) / 255) as u8;
    let bi = ((b as u16 * 5) / 255) as u8;
    16 + 36 * ri + 6 * gi + bi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_wraps() {
        let mut p = PaletteId::Mono;
        for _ in 0..PaletteId::all().len() {
            p = p.next();
        }
        assert_eq!(p, PaletteId::Mono);
    }

    #[test]
    fn parse_aliases() {
        assert_eq!(PaletteId::parse("green"), Some(PaletteId::Matrix));
        assert_eq!(PaletteId::parse("cyan"), Some(PaletteId::Cyberblue));
    }
}
