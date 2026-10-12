//! The generated board of the NFR-02 performance tests.

use std::fs;
use std::path::Path;

/// `board.md` of board `work` (letter W) and canonical Feature cards `W-1` to `W-<cards>`,
/// columns cycling Todo, Doing, Done, each with `notes` notes a minute apart.
pub fn write_big_board(dir: &Path, cards: usize, notes: usize) {
    fs::create_dir_all(dir).unwrap();
    let board = format!(
        "---\nletter: W\ncolumns: Todo, Doing, Done\nnext: {}\n---\n# work\n",
        cards + 1
    );
    fs::write(dir.join("board.md"), board).unwrap();
    // Minutes from 2026-10-01T00:00, one per created stamp and note.
    let stamp = |m: usize| {
        format!(
            "2026-10-{:02}T{:02}:{:02}:00+03:00",
            1 + m / 1440,
            m % 1440 / 60,
            m % 60
        )
    };
    for i in 1..=cards {
        let column = ["Todo", "Doing", "Done"][(i - 1) % 3];
        let base = (i - 1) * (notes + 1);
        let mut text = format!(
            "---\ncolumn: {column}\ntemplate: Feature\ncreated: {}\n---\n# card W-{i}\n\n## Goal\n\n## Acceptance\n\n## Context\n\n## Timeline\n",
            stamp(base)
        );
        for n in 1..=notes {
            text += &format!(
                "\n### {}\n- Doing: step {n} of W-{i}\n- Next: step {} of W-{i}\n- Watch out: nothing\n<!-- where: cwd=/w -->\n",
                stamp(base + n),
                n + 1
            );
        }
        fs::write(dir.join(format!("W-{i}.md")), text).unwrap();
    }
}
