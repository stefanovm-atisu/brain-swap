# Success metrics

How each success metric of PRD section 9 is measured over two weeks of daily use, with the product as built, the transcripts Claude Code keeps and a stopwatch. brain-swap itself records nothing for these metrics.

Transcripts live in `~/.claude/projects/<folder>/*.jsonl`, one file per session, where `<folder>` is the session's working directory with `/` replaced by `-`. The commands below use two filters:

```sh
U='select(.type=="user") | .message.content | strings'
A='select(.type=="assistant") | .message.content[]? | select(.type=="text") | .text'
```

## SM-1 Board to pane

- Definition: median at most 5 seconds and 4 key presses from the herdr board key to typing in the right pane, over 10 timed returns.
- Instrument: a stopwatch and a key count.
- Sample: start the stopwatch on the first key of the board key, count every key press, stop at the first character typed in the target pane.
- Record: one row per return.
- Pass when: the median of the seconds column is at most 5 and the median of the keys column is at most 4.

| Date | Card | Seconds | Keys |
|------|------|---------|------|

## SM-2 Zero-pick parking

- Definition: at least 9 of 10 `/bs-park` calls in a day need no pick.
- Instrument: the day's transcripts.
- Sample: calls are `jq -r "$U" *.jsonl | grep -c '<command-name>/bs-park</command-name>'`; picks are `jq -r "$A" *.jsonl | grep -c 'Park to which card?$'`. A call that goes through `o` asks twice (once per board), so subtract one per `Which board` question: `jq -r "$A" *.jsonl | grep -c '^Which board'`.
- Record: one row per day.
- Pass when: on each day, picks are at most one tenth of calls.

| Date | Calls | Picks |
|------|-------|-------|

## SM-3 Note coverage

- Definition: at most 1 in 5 `/bs-back` returns needs a catch-up note.
- Instrument: the transcripts of the two weeks.
- Sample: returns are `jq -r "$U" *.jsonl | grep -c '<command-name>/bs-back</command-name>'`; catch-ups are `jq -r "$A" *.jsonl | grep -c -E '^no note (since|yet)'`.
- Record: one row per week.
- Pass when: catch-ups are at most one fifth of returns over the two weeks.

| Week | Returns | Catch-ups |
|------|---------|-----------|

## SM-4 Calibration

- Definition: in at least 8 of 10 sampled returns, the first prompt after the note continues the task without scrolling the transcript.
- Instrument: a yes or no noted right after a return.
- Sample: after `/bs-back` prints the note, write yes when the next prompt was typed from the note alone, no when the transcript had to be scrolled first.
- Record: one row per sampled return.
- Pass when: at least 8 of 10 rows say yes.

| Date | Card | Yes or no |
|------|------|-----------|
