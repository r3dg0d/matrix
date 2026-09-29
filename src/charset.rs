//! Character sets for digital rain glyphs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CharsetId {
    Classic,
    Ascii,
    Binary,
    Hex,
    Katakana,
    Nix,
    Custom,
}

impl CharsetId {
    pub fn all() -> &'static [CharsetId] {
        &[
            CharsetId::Classic,
            CharsetId::Ascii,
            CharsetId::Binary,
            CharsetId::Hex,
            CharsetId::Katakana,
            CharsetId::Nix,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            CharsetId::Classic => "classic",
            CharsetId::Ascii => "ascii",
            CharsetId::Binary => "binary",
            CharsetId::Hex => "hex",
            CharsetId::Katakana => "katakana",
            CharsetId::Nix => "nix",
            CharsetId::Custom => "custom",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "classic" | "default" | "matrix" => Some(CharsetId::Classic),
            "ascii" | "letters" => Some(CharsetId::Ascii),
            "binary" | "bin" => Some(CharsetId::Binary),
            "hex" | "hexadecimal" => Some(CharsetId::Hex),
            "katakana" | "kana" | "jp" => Some(CharsetId::Katakana),
            "nix" | "nixos" => Some(CharsetId::Nix),
            "custom" => Some(CharsetId::Custom),
            _ => None,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            CharsetId::Classic => "Mixed halfwidth kana-inspired + digits (original, not film IP)",
            CharsetId::Ascii => "ASCII letters, digits, and symbols",
            CharsetId::Binary => "0 and 1 only",
            CharsetId::Hex => "0-9 A-F",
            CharsetId::Katakana => "Halfwidth katakana-inspired glyphs (original set)",
            CharsetId::Nix => "NixOS-flavored: λ → ⊢ λ nix attrs",
            CharsetId::Custom => "User-provided via --chars",
        }
    }
}

impl std::fmt::Display for CharsetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Glyph pool used by the rain simulator.
#[derive(Debug, Clone)]
pub struct Charset {
    #[allow(dead_code)]
    pub id: CharsetId,
    chars: Vec<char>,
}

impl Charset {
    pub fn from_id(id: CharsetId) -> Self {
        let chars = match id {
            CharsetId::Classic => classic_chars(),
            CharsetId::Ascii => ascii_chars(),
            CharsetId::Binary => vec!['0', '1'],
            CharsetId::Hex => "0123456789ABCDEF".chars().collect(),
            CharsetId::Katakana => katakana_chars(),
            CharsetId::Nix => nix_chars(),
            CharsetId::Custom => classic_chars(),
        };
        Self { id, chars }
    }

    pub fn from_custom(s: &str) -> anyhow::Result<Self> {
        let chars: Vec<char> = s.chars().filter(|c| !c.is_control() && *c != ' ').collect();
        if chars.is_empty() {
            anyhow::bail!("--chars must contain at least one printable non-space character");
        }
        Ok(Self {
            id: CharsetId::Custom,
            chars,
        })
    }

    pub fn pick(&self, rng: &mut impl rand::Rng) -> char {
        self.chars[rng.gen_range(0..self.chars.len())]
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }
}

fn classic_chars() -> Vec<char> {
    // Original halfwidth-kana-inspired mix + digits/symbols — not a film rip.
    let mut v: Vec<char> = "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝﾞﾟ0123456789:;.=*+-<>|"
        .chars()
        .collect();
    v.extend("αβγδεζηθικλμνξοπρστυφχψω".chars());
    v
}

fn ascii_chars() -> Vec<char> {
    (33u8..127).map(|b| b as char).collect()
}

fn katakana_chars() -> Vec<char> {
    "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜｦﾝｧｨｩｪｫｬｭｮｯｰ･｡｢｣"
        .chars()
        .collect()
}

fn nix_chars() -> Vec<char> {
    "λ→⊢∀∃∈∉⊂⊃∪∩{}[];:=.#@nixosNIXOS0123456789abcdefghijklmnopqrstuvwxyz"
        .chars()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_aliases() {
        assert_eq!(CharsetId::parse("classic"), Some(CharsetId::Classic));
        assert_eq!(CharsetId::parse("BIN"), Some(CharsetId::Binary));
        assert_eq!(CharsetId::parse("kana"), Some(CharsetId::Katakana));
        assert_eq!(CharsetId::parse("nope"), None);
    }

    #[test]
    fn custom_rejects_empty() {
        assert!(Charset::from_custom("   ").is_err());
        assert!(Charset::from_custom("abc").is_ok());
    }

    #[test]
    fn all_sets_nonempty() {
        for id in CharsetId::all() {
            assert!(!Charset::from_id(*id).is_empty());
        }
    }
}
