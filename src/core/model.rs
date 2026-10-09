//! Domain model (TECHSPEC 3).

use std::fmt;
use std::path::PathBuf;

/// A card ID such as `W-12` (I1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId {
    pub letter: char,
    pub number: u32,
}

impl CardId {
    /// Parses `W-12` or `w-12`; the letter is upper-cased.
    pub fn parse(s: &str) -> Option<CardId> {
        let (l, n) = s.split_once('-')?;
        let mut l = l.chars();
        let letter = l.next().filter(char::is_ascii_alphabetic)?;
        if l.next().is_some() || n.starts_with('0') || !n.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        Some(CardId {
            letter: letter.to_ascii_uppercase(),
            number: n.parse().ok()?,
        })
    }

    /// Reads a card file name `^[A-Z]-[1-9][0-9]*\.md$` (4.1).
    pub fn from_file_name(name: &str) -> Option<CardId> {
        name.strip_suffix(".md")
            .filter(|s| s.starts_with(|c: char| c.is_ascii_uppercase()))
            .and_then(CardId::parse)
    }

    /// The card file name `<id>.md` (I2).
    pub fn file_name(&self) -> String {
        format!("{self}.md")
    }
}

impl fmt::Display for CardId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.letter, self.number)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Place {
    pub cwd: Option<PathBuf>,
    pub herdr: Option<HerdrPlace>,
    pub session: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HerdrPlace {
    pub pane: String,
    pub tab: Option<String>,
    pub workspace: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> CardId {
        CardId::parse(s).unwrap()
    }

    #[test]
    fn i1_parse_reads_letter_and_number() {
        assert_eq!(
            CardId::parse("W-12"),
            Some(CardId {
                letter: 'W',
                number: 12
            })
        );
    }

    #[test]
    fn i1_parse_upper_cases_letter() {
        assert_eq!(CardId::parse("w-12"), CardId::parse("W-12"));
    }

    #[test]
    fn i1_parse_rejects_malformed_ids() {
        for s in ["W-0", "W-012", "W-", "WW-1", "1-2", "W12", "W-1x", ""] {
            assert_eq!(CardId::parse(s), None, "{s}");
        }
    }

    #[test]
    fn i1_parse_rejects_number_beyond_u32() {
        assert_eq!(CardId::parse("W-4294967296"), None);
    }

    #[test]
    fn i2_file_name_is_id_dot_md() {
        assert_eq!(
            (id("W-12").file_name(), id("W-12").to_string()),
            ("W-12.md".to_string(), "W-12".to_string())
        );
    }

    #[test]
    fn ts4_1_from_file_name_requires_card_pattern() {
        assert_eq!(CardId::from_file_name("W-12.md"), Some(id("W-12")));
        for s in [
            "w-12.md",
            "W-012.md",
            ".W-12.md.bs-tmp-4711",
            "W-12.md.bak",
            "README.md",
        ] {
            assert_eq!(CardId::from_file_name(s), None, "{s}");
        }
    }

    #[test]
    fn ts3_place_default_has_no_parts() {
        let p = Place::default();
        assert!(p.cwd.is_none() && p.herdr.is_none() && p.session.is_none());
        assert_eq!(
            p,
            Place {
                cwd: None,
                herdr: None,
                session: None
            }
        );
    }
}
