use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hotkey {
    pub modifiers: Vec<Modifier>,
    pub key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modifier {
    Super,
    Ctrl,
    Alt,
    Shift,
}

impl Hotkey {
    pub fn new(modifiers: Vec<Modifier>, key: impl Into<String>) -> Self {
        Self {
            modifiers,
            key: key.into(),
        }
    }

    /// Parse a hotkey string like "Super+Shift+T".
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            anyhow::bail!("empty hotkey string");
        }
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();

        let mut modifiers = Vec::new();
        for &part in &parts[..parts.len() - 1] {
            let modifier = match part.to_lowercase().as_str() {
                "super" | "meta" | "win" => Modifier::Super,
                "ctrl" | "control" => Modifier::Ctrl,
                "alt" => Modifier::Alt,
                "shift" => Modifier::Shift,
                other => anyhow::bail!("unknown modifier: {other}"),
            };
            modifiers.push(modifier);
        }

        let key = parts
            .last()
            .ok_or_else(|| anyhow::anyhow!("missing key"))?
            .to_string();

        Ok(Self { modifiers, key })
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, m) in self.modifiers.iter().enumerate() {
            if i > 0 {
                write!(f, "+")?;
            }
            let name = match m {
                Modifier::Super => "Super",
                Modifier::Ctrl => "Ctrl",
                Modifier::Alt => "Alt",
                Modifier::Shift => "Shift",
            };
            write!(f, "{name}")?;
        }
        if !self.modifiers.is_empty() {
            write!(f, "+")?;
        }
        write!(f, "{}", self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_hotkey() {
        let hk = Hotkey::parse("Super+T").unwrap();
        assert_eq!(hk.modifiers, vec![Modifier::Super]);
        assert_eq!(hk.key, "T");
    }

    #[test]
    fn parse_multi_modifier() {
        let hk = Hotkey::parse("Super+Shift+C").unwrap();
        assert_eq!(hk.modifiers, vec![Modifier::Super, Modifier::Shift]);
        assert_eq!(hk.key, "C");
    }

    #[test]
    fn display_roundtrip() {
        let hk = Hotkey::parse("Ctrl+Alt+Delete").unwrap();
        assert_eq!(hk.to_string(), "Ctrl+Alt+Delete");
    }

    #[test]
    fn parse_invalid() {
        assert!(Hotkey::parse("").is_err());
    }
}
