//! Key binding syntax of TECHSPEC 5.3, terminal-free (A-E1-24).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Tab,
    BackTab,
    Space,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Binding {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyPress {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

const NAMED: [(&str, Key); 15] = [
    ("enter", Key::Enter),
    ("esc", Key::Esc),
    ("tab", Key::Tab),
    ("backtab", Key::BackTab),
    ("space", Key::Space),
    ("backspace", Key::Backspace),
    ("delete", Key::Delete),
    ("up", Key::Up),
    ("down", Key::Down),
    ("left", Key::Left),
    ("right", Key::Right),
    ("home", Key::Home),
    ("end", Key::End),
    ("pageup", Key::PageUp),
    ("pagedown", Key::PageDown),
];

pub fn parse_binding(s: &str) -> Result<Binding, String> {
    let mut parts: Vec<&str> = s.split('+').collect();
    let name = parts.pop().unwrap_or_default();
    let mut chars = name.chars();
    let key = match (chars.next(), chars.next()) {
        (None, _) => return Err("empty binding".into()),
        (Some(c), None) if !c.is_control() => Key::Char(c),
        _ => NAMED
            .iter()
            .find(|(n, _)| *n == name)
            .map(|&(_, k)| k)
            .or_else(|| (1..=12).find(|n| format!("f{n}") == name).map(Key::F))
            .ok_or(format!("unknown key '{name}'"))?,
    };
    let mut b = Binding {
        key,
        ctrl: false,
        alt: false,
        shift: false,
    };
    for m in parts {
        match m {
            "ctrl" => b.ctrl = true,
            "alt" => b.alt = true,
            "shift" => b.shift = true,
            _ => return Err(format!("unknown modifier '{m}'")),
        }
    }
    if let (Key::Char(c), true) = (b.key, b.shift) {
        // ponytail: ASCII upper-casing only, use char::to_uppercase if non-ASCII shift bindings matter
        b.key = Key::Char(c.to_ascii_uppercase());
        b.shift = false;
    }
    Ok(b)
}

impl Binding {
    /// SHIFT is ignored for printable characters (crossterm reports `H` with SHIFT set).
    pub fn matches(&self, press: &KeyPress) -> bool {
        self.key == press.key
            && self.ctrl == press.ctrl
            && self.alt == press.alt
            && (matches!(self.key, Key::Char(_)) || self.shift == press.shift)
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (on, m) in [
            (self.ctrl, "ctrl+"),
            (self.alt, "alt+"),
            (self.shift, "shift+"),
        ] {
            if on {
                f.write_str(m)?;
            }
        }
        match self.key {
            Key::Char(c) => write!(f, "{c}"),
            Key::F(n) => write!(f, "f{n}"),
            k => f.write_str(NAMED.iter().find(|(_, x)| *x == k).map_or("", |(n, _)| n)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(key: Key, ctrl: bool, alt: bool, shift: bool) -> Binding {
        Binding {
            key,
            ctrl,
            alt,
            shift,
        }
    }

    fn press(key: Key, ctrl: bool, alt: bool, shift: bool) -> KeyPress {
        KeyPress {
            key,
            ctrl,
            alt,
            shift,
        }
    }

    #[test]
    fn ts5_3_parses_single_character() {
        assert_eq!(
            parse_binding("o"),
            Ok(b(Key::Char('o'), false, false, false))
        );
    }

    #[test]
    fn ts5_3_character_is_case_sensitive() {
        assert_eq!(parse_binding("H").unwrap().key, Key::Char('H'));
        assert_eq!(parse_binding("h").unwrap().key, Key::Char('h'));
    }

    #[test]
    fn ts5_3_shift_h_normalises_to_upper_h() {
        assert_eq!(parse_binding("shift+h"), parse_binding("H"));
    }

    #[test]
    fn ts5_3_parses_named_keys() {
        let cases = [
            ("enter", Key::Enter),
            ("esc", Key::Esc),
            ("tab", Key::Tab),
            ("backtab", Key::BackTab),
            ("space", Key::Space),
            ("backspace", Key::Backspace),
            ("delete", Key::Delete),
            ("up", Key::Up),
            ("down", Key::Down),
            ("left", Key::Left),
            ("right", Key::Right),
            ("home", Key::Home),
            ("end", Key::End),
            ("pageup", Key::PageUp),
            ("pagedown", Key::PageDown),
        ];
        for (s, key) in cases {
            assert_eq!(parse_binding(s), Ok(b(key, false, false, false)), "{s}");
        }
    }

    #[test]
    fn ts5_3_parses_function_keys_1_to_12() {
        assert_eq!(parse_binding("f1").unwrap().key, Key::F(1));
        assert_eq!(parse_binding("f12").unwrap().key, Key::F(12));
        assert!(parse_binding("f0").is_err());
        assert!(parse_binding("f13").is_err());
        assert_eq!(parse_binding("f01"), Err("unknown key 'f01'".to_string()));
    }

    #[test]
    fn ts5_3_parses_modifier_combinations() {
        assert_eq!(
            parse_binding("ctrl+d"),
            Ok(b(Key::Char('d'), true, false, false))
        );
        assert_eq!(
            parse_binding("alt+x"),
            Ok(b(Key::Char('x'), false, true, false))
        );
        assert_eq!(
            parse_binding("ctrl+alt+left"),
            Ok(b(Key::Left, true, true, false))
        );
    }

    #[test]
    fn ts5_3_rejects_malformed_bindings() {
        assert_eq!(parse_binding(""), Err("empty binding".to_string()));
        assert_eq!(parse_binding("ctrl+"), Err("empty binding".to_string()));
        assert_eq!(parse_binding("foo"), Err("unknown key 'foo'".to_string()));
        assert_eq!(
            parse_binding("hyper+x"),
            Err("unknown modifier 'hyper'".to_string())
        );
        assert_eq!(parse_binding("\t"), Err("unknown key '\t'".to_string()));
        assert_eq!(
            parse_binding("\u{7}"),
            Err("unknown key '\u{7}'".to_string())
        );
    }

    #[test]
    fn ts5_3_matches_upper_h_with_and_without_shift() {
        let h = parse_binding("H").unwrap();
        assert!(h.matches(&press(Key::Char('H'), false, false, true)));
        assert!(h.matches(&press(Key::Char('H'), false, false, false)));
    }

    #[test]
    fn ts5_3_ctrl_and_alt_must_match_exactly() {
        let ctrl_d = parse_binding("ctrl+d").unwrap();
        assert!(ctrl_d.matches(&press(Key::Char('d'), true, false, false)));
        assert!(!ctrl_d.matches(&press(Key::Char('d'), false, false, false)));
        assert!(!ctrl_d.matches(&press(Key::Char('d'), true, true, false)));
        let d = parse_binding("d").unwrap();
        assert!(!d.matches(&press(Key::Char('d'), true, false, false)));
    }

    #[test]
    fn ts5_3_display_round_trips() {
        for s in ["ctrl+d", "H", "enter", "f5"] {
            assert_eq!(parse_binding(s).unwrap().to_string(), s);
        }
    }
}
