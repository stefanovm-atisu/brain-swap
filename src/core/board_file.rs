//! `board.md` reader (TECHSPEC 4.3).

use crate::core::frontmatter::{FmRead, read};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardFile {
    pub letter: char,
    pub columns: Vec<String>,
    pub next: Option<u32>,
    pub writable: bool,
    pub warnings: Vec<String>,
}

const DEFAULT_COLUMNS: [&str; 3] = ["Todo", "Doing", "Done"];

pub fn parse(board_name: &str, bytes: Option<&[u8]>) -> BoardFile {
    let derived = board_name
        .chars()
        .next()
        .unwrap_or('A')
        .to_ascii_uppercase();
    let mut b = BoardFile {
        letter: derived,
        columns: DEFAULT_COLUMNS.map(String::from).to_vec(),
        next: None,
        writable: true,
        warnings: Vec::new(),
    };
    let text = String::from_utf8_lossy(bytes.unwrap_or_default());
    let fm = match read(&text) {
        FmRead::None => return b,
        FmRead::Invalid { line, problem } => {
            b.writable = false;
            b.warnings.push(format!("board.md:{line}: {problem}"));
            return b;
        }
        FmRead::Valid(fm) => fm,
    };
    if let Some(v) = fm.get("letter").map(str::trim) {
        match v
            .chars()
            .map(|c| c.to_ascii_uppercase())
            .collect::<Vec<_>>()[..]
        {
            [c @ 'A'..='Z'] => b.letter = c,
            _ => b
                .warnings
                .push(format!("board.md: invalid letter '{v}', using {derived}")),
        }
    }
    if let Some(v) = fm.get("columns") {
        b.columns.clear();
        for c in v.split(',').map(str::trim).filter(|c| !c.is_empty()) {
            if b.columns
                .iter()
                .any(|d| d.to_lowercase() == c.to_lowercase())
            {
                b.warnings
                    .push(format!("board.md: duplicate column '{c}' dropped"));
            } else {
                b.columns.push(c.to_string());
            }
        }
        if b.columns.is_empty() {
            b.columns = DEFAULT_COLUMNS.map(String::from).to_vec();
            b.warnings
                .push("board.md: no columns, using Todo, Doing, Done".to_string());
        }
    }
    if let Some(v) = fm.get("next").map(str::trim) {
        b.next = v.parse().ok().filter(|n| *n > 0);
        if b.next.is_none() {
            b.warnings.push(format!("board.md: invalid next '{v}'"));
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(text: &str) -> BoardFile {
        i3(parse("work", Some(text.as_bytes())))
    }

    /// Asserts I3: columns non-empty and unique ignoring case.
    fn i3(b: BoardFile) -> BoardFile {
        assert!(!b.columns.is_empty());
        for (i, c) in b.columns.iter().enumerate() {
            assert!(
                !b.columns[..i]
                    .iter()
                    .any(|d| d.to_lowercase() == c.to_lowercase())
            );
        }
        b
    }

    fn defaults(b: &BoardFile) {
        assert_eq!(b.letter, 'W');
        assert_eq!(b.columns, ["Todo", "Doing", "Done"]);
        assert_eq!(b.next, None);
    }

    #[test]
    fn fr03_missing_board_md_gives_defaults() {
        let b = i3(parse("work", None));
        defaults(&b);
        assert!(b.writable);
        assert!(b.warnings.is_empty());
    }

    #[test]
    fn fr03_reads_letter_columns_next() {
        let b = p("---\nletter: W\ncolumns: Todo, Doing, Done\nnext: 13\n---\n");
        assert_eq!(b.letter, 'W');
        assert_eq!(b.columns, ["Todo", "Doing", "Done"]);
        assert_eq!(b.next, Some(13));
        assert!(b.warnings.is_empty());
    }

    #[test]
    fn ts4_3_lower_case_letter_is_upper_cased() {
        let b = p("---\nletter: w\n---\n");
        assert_eq!(b.letter, 'W');
        assert!(b.warnings.is_empty());
    }

    #[test]
    fn ts4_3_invalid_letter_warns_and_derives() {
        for v in ["7", "WX"] {
            let b = p(&format!("---\nletter: {v}\n---\n"));
            assert_eq!(b.letter, 'W');
            assert_eq!(
                b.warnings,
                [format!("board.md: invalid letter '{v}', using W")]
            );
        }
    }

    #[test]
    fn ts4_3_columns_are_a_trimmed_comma_list() {
        let b = p("---\ncolumns: Backlog ,Doing,  Review, Done\n---\n");
        assert_eq!(b.columns, ["Backlog", "Doing", "Review", "Done"]);
    }

    #[test]
    fn i3_duplicate_columns_dropped_with_warning() {
        let b = p("---\ncolumns: Todo, todo, Done\n---\n");
        assert_eq!(b.columns, ["Todo", "Done"]);
        assert_eq!(b.warnings, ["board.md: duplicate column 'todo' dropped"]);
        let b = p("---\ncolumns: Ärger, ärger, Done\n---\n");
        assert_eq!(b.columns, ["Ärger", "Done"]);
        assert_eq!(b.warnings, ["board.md: duplicate column 'ärger' dropped"]);
    }

    #[test]
    fn i3_empty_columns_give_defaults() {
        let b = p("---\ncolumns:\n---\n");
        assert_eq!(b.columns, ["Todo", "Doing", "Done"]);
        assert_eq!(
            b.warnings,
            ["board.md: no columns, using Todo, Doing, Done"]
        );
    }

    #[test]
    fn ts4_3_board_md_without_frontmatter_gives_defaults() {
        let b = p("# work\n");
        defaults(&b);
        assert!(b.writable);
        assert!(b.warnings.is_empty());
    }

    #[test]
    fn ts4_3_invalid_frontmatter_warns_and_blocks_writes() {
        let b = p("---\nletter W\n---\n");
        defaults(&b);
        assert!(!b.writable);
        assert_eq!(b.warnings.len(), 1);
        assert!(
            b.warnings[0].starts_with("board.md:2: "),
            "{:?}",
            b.warnings
        );
        assert!(b.warnings[0].len() > "board.md:2: ".len());
    }

    #[test]
    fn ts4_3_bad_next_warns() {
        let b = p("---\nnext: x\n---\n");
        assert_eq!(b.next, None);
        assert_eq!(b.warnings, ["board.md: invalid next 'x'"]);
    }
}
