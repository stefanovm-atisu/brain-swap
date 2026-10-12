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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    MoveLeft,
    MoveRight,
    Open,
    Jump,
    New,
    Edit,
    SwitchBoard,
    ScrollDown,
    ScrollUp,
    Help,
    Quit,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    Board,
    Detail,
    Picker,
    TitleInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMap(Vec<Vec<Binding>>);

use Action::*;

const ACTIONS: [(Action, &str, &[&str]); 17] = [
    (Up, "up", &["k", "up"]),
    (Down, "down", &["j", "down"]),
    (Left, "left", &["h", "left"]),
    (Right, "right", &["l", "right"]),
    (MoveLeft, "move_left", &["H"]),
    (MoveRight, "move_right", &["L"]),
    (Open, "open", &["o"]),
    (Jump, "jump", &["enter"]),
    (New, "new", &["n"]),
    (Edit, "edit", &["e"]),
    (SwitchBoard, "switch_board", &["b"]),
    (ScrollDown, "scroll_down", &["ctrl+d"]),
    (ScrollUp, "scroll_up", &["ctrl+u"]),
    (Help, "help", &["?"]),
    (Quit, "quit", &["q", "esc"]),
    (Confirm, "confirm", &["enter"]),
    (Cancel, "cancel", &["esc"]),
];

impl Action {
    pub fn config_name(self) -> &'static str {
        ACTIONS[self as usize].1
    }
}

impl KeyContext {
    fn actions(self) -> &'static [Action] {
        match self {
            KeyContext::Board => &[
                Up,
                Down,
                Left,
                Right,
                MoveLeft,
                MoveRight,
                Open,
                Jump,
                New,
                Edit,
                SwitchBoard,
                Help,
                Quit,
            ],
            KeyContext::Detail => &[Up, Down, ScrollDown, ScrollUp, Jump, Edit, Help, Quit],
            KeyContext::Picker => &[Up, Down, Confirm, Cancel],
            KeyContext::TitleInput => &[Confirm, Cancel],
        }
    }

    /// First pair of actions sharing a binding in this context, with that binding.
    fn conflict(self, m: &KeyMap) -> Option<(Action, Action, Binding)> {
        let acts = self.actions();
        acts.iter().enumerate().find_map(|(i, &a)| {
            acts[i + 1..].iter().find_map(|&b| {
                m.keys(a)
                    .iter()
                    .find(|k| m.keys(b).contains(k))
                    .map(|&k| (a, b, k))
            })
        })
    }
}

impl KeyMap {
    pub fn defaults() -> KeyMap {
        KeyMap(
            ACTIONS
                .iter()
                .map(|(_, _, ks)| ks.iter().map(|k| parse_binding(k).unwrap()).collect())
                .collect(),
        )
    }

    pub fn build(user: &[(String, Vec<String>)]) -> (KeyMap, Vec<String>) {
        let defaults = KeyMap::defaults();
        let mut map = defaults.clone();
        let mut user_set = [false; 17];
        let mut warnings = Vec::new();
        for (name, strs) in user {
            let Some(i) = ACTIONS.iter().position(|(_, n, _)| n == name) else {
                warnings.push(format!("key config: {name}: unknown action"));
                continue;
            };
            match strs.iter().map(|s| parse_binding(s)).collect() {
                Ok(bs) => (map.0[i], user_set[i]) = (bs, true),
                Err(e) => warnings.push(format!("key config: {name}: {e}")),
            }
        }
        // Defaults are conflict-free, so every conflict involves a user-set action and each round drops one.
        while let Some((a, b, k)) = [
            KeyContext::Board,
            KeyContext::Detail,
            KeyContext::Picker,
            KeyContext::TitleInput,
        ]
        .iter()
        .find_map(|c| c.conflict(&map))
        {
            for (x, y) in [(a, b), (b, a)] {
                if user_set[x as usize] {
                    user_set[x as usize] = false;
                    map.0[x as usize] = defaults.0[x as usize].clone();
                    warnings.push(format!(
                        "key config: {}: conflicts with {} on '{k}'",
                        x.config_name(),
                        y.config_name()
                    ));
                }
            }
        }
        (map, warnings)
    }

    pub fn action(&self, ctx: KeyContext, press: &KeyPress) -> Option<Action> {
        ctx.actions()
            .iter()
            .copied()
            .find(|&a| self.keys(a).iter().any(|k| k.matches(press)))
    }

    pub fn keys(&self, action: Action) -> &[Binding] {
        &self.0[action as usize]
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

    fn user(entries: &[(&str, &[&str])]) -> Vec<(String, Vec<String>)> {
        entries
            .iter()
            .map(|(a, ks)| (a.to_string(), ks.iter().map(|k| k.to_string()).collect()))
            .collect()
    }

    fn bs(keys: &[&str]) -> Vec<Binding> {
        keys.iter().map(|k| parse_binding(k).unwrap()).collect()
    }

    fn p(k: &str) -> KeyPress {
        let b = parse_binding(k).unwrap();
        press(b.key, b.ctrl, b.alt, b.shift)
    }

    #[test]
    fn ts5_3_defaults_are_conflict_free() {
        let m = KeyMap::defaults();
        for ctx in [
            KeyContext::Board,
            KeyContext::Detail,
            KeyContext::Picker,
            KeyContext::TitleInput,
        ] {
            let mut seen: Vec<(Binding, Action)> = Vec::new();
            for &a in ctx.actions() {
                for &k in m.keys(a) {
                    if let Some((_, other)) = seen.iter().find(|(b, _)| *b == k) {
                        panic!("{ctx:?}: '{k}' bound to {other:?} and {a:?}");
                    }
                    seen.push((k, a));
                }
            }
        }
    }

    #[test]
    fn fr43_user_entry_replaces_defaults() {
        let (m, w) = KeyMap::build(&user(&[("open", &["x"])]));
        assert!(w.is_empty());
        assert_eq!(m.keys(Action::Open), bs(&["x"]));
        assert_eq!(m.action(KeyContext::Board, &p("o")), None);
    }

    #[test]
    fn fr44_bad_binding_warns_and_keeps_default() {
        let (m, w) = KeyMap::build(&user(&[("open", &["foo"])]));
        assert_eq!(m.keys(Action::Open), bs(&["o"]));
        assert_eq!(w, ["key config: open: unknown key 'foo'"]);
    }

    #[test]
    fn fr44_unknown_action_warns() {
        let (m, w) = KeyMap::build(&user(&[("jumpp", &["x"])]));
        assert_eq!(m, KeyMap::defaults());
        assert_eq!(w, ["key config: jumpp: unknown action"]);
    }

    #[test]
    fn fr44_swap_of_two_keys_is_accepted() {
        let (m, w) = KeyMap::build(&user(&[("open", &["n"]), ("new", &["o"])]));
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(m.action(KeyContext::Board, &p("n")), Some(Action::Open));
        assert_eq!(m.action(KeyContext::Board, &p("o")), Some(Action::New));
    }

    #[test]
    fn fr44_user_binding_colliding_with_default_reverts_only_user_action() {
        let (m, w) = KeyMap::build(&user(&[("new", &["j"])]));
        assert_eq!(m.keys(Action::New), bs(&["n"]));
        assert_eq!(m.keys(Action::Down), bs(&["j", "down"]));
        assert_eq!(w, ["key config: new: conflicts with down on 'j'"]);
    }

    #[test]
    fn fr44_both_user_set_actions_revert_on_conflict() {
        let (m, w) = KeyMap::build(&user(&[("open", &["x"]), ("edit", &["x"])]));
        assert_eq!(m.keys(Action::Open), bs(&["o"]));
        assert_eq!(m.keys(Action::Edit), bs(&["e"]));
        assert_eq!(
            w,
            [
                "key config: open: conflicts with edit on 'x'",
                "key config: edit: conflicts with open on 'x'"
            ]
        );
    }

    #[test]
    fn ts5_3_conflict_is_checked_per_context() {
        let (m, w) = KeyMap::build(&user(&[("scroll_down", &["o"])]));
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(m.keys(Action::ScrollDown), bs(&["o"]));
        let (m, w) = KeyMap::build(&user(&[("scroll_down", &["e"])]));
        assert_eq!(m.keys(Action::ScrollDown), bs(&["ctrl+d"]));
        assert_eq!(w, ["key config: scroll_down: conflicts with edit on 'e'"]);
    }

    #[test]
    fn ts5_3_reverted_action_returns_to_defaults_in_every_context() {
        let (m, w) = KeyMap::build(&user(&[("up", &["o"])]));
        assert_eq!(m.keys(Action::Up), bs(&["k", "up"]));
        assert_eq!(w, ["key config: up: conflicts with open on 'o'"]);
        for ctx in [KeyContext::Detail, KeyContext::Picker] {
            assert_eq!(m.action(ctx, &p("o")), None, "{ctx:?}");
            assert_eq!(m.action(ctx, &p("k")), Some(Action::Up), "{ctx:?}");
        }
    }

    #[test]
    fn ts5_3_conflict_check_repeats_until_clean() {
        let (m, w) = KeyMap::build(&user(&[
            ("new", &["o"]),
            ("open", &["x"]),
            ("edit", &["x"]),
        ]));
        assert_eq!(m, KeyMap::defaults());
        assert_eq!(
            w,
            [
                "key config: open: conflicts with edit on 'x'",
                "key config: edit: conflicts with open on 'x'",
                "key config: new: conflicts with open on 'o'"
            ]
        );
    }

    #[test]
    fn fr43_action_lookup_by_context() {
        let m = KeyMap::defaults();
        assert_eq!(m.action(KeyContext::Detail, &p("j")), Some(Action::Down));
        assert_eq!(
            m.action(KeyContext::Picker, &p("enter")),
            Some(Action::Confirm)
        );
        assert_eq!(m.action(KeyContext::Board, &p("enter")), Some(Action::Jump));
    }
}
