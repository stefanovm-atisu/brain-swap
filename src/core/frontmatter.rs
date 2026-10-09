//! Frontmatter reader: the flat subset of TECHSPEC 4.2, parsed by hand.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter {
    pub entries: Vec<Entry>,
    pub close_line: usize,
    /// Byte offset after the closing `---` line.
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FmRead {
    None,
    Valid(Frontmatter),
    Invalid { line: usize, problem: String },
}

impl Frontmatter {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.key.eq_ignore_ascii_case(key))
            .map(|e| e.value.as_str())
    }
}

pub fn read(text: &str) -> FmRead {
    if !text.starts_with("---") {
        return FmRead::None;
    }
    let mut entries = Vec::new();
    let mut end = 0;
    for (i, raw) in text.split_inclusive('\n').enumerate() {
        end += raw.len();
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let n = i + 1;
        if i == 0 {
            if line != "---" {
                return FmRead::None;
            }
        } else if line == "---" {
            return FmRead::Valid(Frontmatter {
                entries,
                close_line: n,
                end,
            });
        } else if line.is_empty() || line.starts_with(['#', ' ', '\t']) || line.starts_with("- ") {
            // comment, blank line or continuation: kept verbatim, not part of any value
        } else {
            match line.split_once(':') {
                Some((key, rest))
                    if !key.is_empty()
                        && key
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                        && (rest.is_empty() || rest.starts_with(' ')) =>
                {
                    let v = rest.strip_prefix(' ').unwrap_or(rest);
                    let v = match v.as_bytes() {
                        [q @ (b'"' | b'\''), .., l] if q == l => &v[1..v.len() - 1],
                        _ => v,
                    };
                    entries.push(Entry {
                        key: key.to_string(),
                        value: v.to_string(),
                        line: n,
                    });
                }
                _ => {
                    return FmRead::Invalid {
                        line: n,
                        problem: format!("not a key, comment or continuation: {line}"),
                    };
                }
            }
        }
    }
    FmRead::Invalid {
        line: 1,
        problem: "no closing ---".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(text: &str) -> Frontmatter {
        match read(text) {
            FmRead::Valid(fm) => fm,
            other => panic!("expected Valid, got {other:?}"),
        }
    }

    #[test]
    fn ts4_2_reads_key_value_pairs() {
        let text = "---\ncolumn: Doing\ntemplate: Feature\n---\n# t\n";
        let fm = valid(text);
        assert_eq!(fm.get("column"), Some("Doing"));
        assert_eq!(fm.get("template"), Some("Feature"));
        for i in 0..=text.len() {
            read(&text[..i]);
        }
    }

    #[test]
    fn ts4_2_keys_match_ignoring_case() {
        assert_eq!(
            valid("---\nColumn: Doing\n---\n").get("column"),
            Some("Doing")
        );
    }

    #[test]
    fn ts4_2_strips_one_pair_of_quotes() {
        let fm = valid("---\ncolumn: \"In review\"\ntitle: \"\"x\"\"\n---\n");
        assert_eq!(fm.get("column"), Some("In review"));
        assert_eq!(fm.get("title"), Some("\"x\""));
    }

    #[test]
    fn ts4_2_key_without_value_is_empty() {
        assert_eq!(valid("---\ntemplate:\n---\n").get("template"), Some(""));
    }

    #[test]
    fn ts4_2_continuation_lines_belong_to_previous_key() {
        let fm = valid("---\ntags:\n  - a\n- b\n---\n");
        assert_eq!(fm.entries.len(), 1);
        assert_eq!(fm.entries[0].key, "tags");
        assert_eq!(fm.entries[0].value, "");
    }

    #[test]
    fn ts4_2_comments_and_blank_lines_are_allowed() {
        let fm = valid("---\ncolumn: Doing\n# note\n\ntemplate: Feature\n---\n");
        assert_eq!(fm.get("column"), Some("Doing"));
        assert_eq!(fm.get("template"), Some("Feature"));
    }

    #[test]
    fn ts4_2_block_only_at_byte_0() {
        assert_eq!(read("\n---\ncolumn: x\n---\n"), FmRead::None);
    }

    #[test]
    fn ts4_2_invalid_line_invalidates_block() {
        assert!(matches!(
            read("---\ncolumn Doing\n---\n"),
            FmRead::Invalid { line: 2, .. }
        ));
    }

    #[test]
    fn ts4_2_unclosed_block_is_invalid() {
        assert!(matches!(
            read("---\ncolumn: Doing\n# t\n"),
            FmRead::Invalid { .. }
        ));
    }

    #[test]
    fn ts4_2_crlf_block_reads() {
        let fm = valid("---\r\ncolumn: Doing\r\ntemplate: Feature\r\n---\r\n# t\r\n");
        assert_eq!(fm.get("column"), Some("Doing"));
        assert_eq!(fm.get("template"), Some("Feature"));
    }

    #[test]
    fn ts4_2_text_without_block_is_none() {
        assert_eq!(read("# title\n"), FmRead::None);
    }
}
