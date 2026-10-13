//! The generated board of the performance tests (A-E6-03).

use brain_swap::core::card_file::{render_new, render_note};
use brain_swap::core::model::{HerdrPlace, NewCard, NewNote};
use brain_swap::core::time::parse_stamp;
use jiff::SignedDuration;
use jiff::tz::TimeZone;
use std::fs;
use std::path::Path;

/// `board.md` of board `work` (letter W) and canonical Feature cards `W-1` to `W-<cards>`:
/// the last 5 in Doing, the 5 before in Todo, the rest in Done, each with `notes` notes,
/// stamps one hour apart in card and note order from 2026-01-01.
pub fn write_big_board(dir: &Path, cards: usize, notes: usize) {
    fs::create_dir_all(dir).unwrap();
    let board = format!(
        "---\nletter: W\ncolumns: Todo, Doing, Done\nnext: {}\n---\n# work\n",
        cards + 1
    );
    fs::write(dir.join("board.md"), board).unwrap();
    let t0 = parse_stamp("2026-01-01T00:00:00+03:00", &TimeZone::UTC).unwrap();
    let stamp = |h: usize| {
        t0.checked_add(SignedDuration::from_hours(h as i64))
            .unwrap()
    };
    for i in 1..=cards {
        let column = match cards - i {
            0..5 => "Doing",
            5..10 => "Todo",
            _ => "Done",
        };
        let base = (i - 1) * (notes + 1);
        let card = NewCard {
            title: format!("perf card {i} tunes the export pipeline"),
            template: "Feature".into(),
            column: column.into(),
            created: stamp(base),
            body: format!(
                "## Goal\n\nExport W-{i} invoices in one pass without a retry.\n\n\
                 ## Acceptance\n\nThe nightly export of W-{i} ends within ten minutes.\n\n\
                 ## Context\n\nThe export of W-{i} runs from the billing service cron job.\n"
            ),
        };
        let mut text = render_new(&card).0;
        for n in 1..=notes {
            let note = NewNote {
                at: stamp(base + n),
                auto: false,
                doing: format!("Step {n} of W-{i}: moved the invoice query to a batch."),
                next: format!(
                    "Step {} of W-{i}: time the batch against last night.",
                    n + 1
                ),
                watch_out: format!("Step {n} of W-{i}: the old cron still runs at two."),
                cwd: format!("/home/perf/Work/proj-{}", i % 5).into(),
                herdr: Some(HerdrPlace {
                    pane: "w4V:p9".into(),
                    tab: Some("w4V:t1".into()),
                    workspace: Some("w4V".into()),
                }),
                session: Some(format!("perf-{i}")),
            };
            text.extend(render_note(&note, "\n").0.bytes());
        }
        fs::write(dir.join(format!("W-{i}.md")), text).unwrap();
    }
}
