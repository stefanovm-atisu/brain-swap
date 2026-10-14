# SP-8 staleness scripts

Ten scripted sessions, S1 to S10, that score the shipped `/bs-back` staleness judgement (TECHSPEC 14 SP-8, 8.4 step 3, 8.6; PRD FR-51 to FR-53). Five are stale and five fresh. The ground truth of each script is fixed here, before the first run (E0-F3-T1); E0-F3-T2 runs each script once and fills the score table; E0-F3-T3 adds a second table only if the first scores below 9 of 10.

Ground truth follows from FR-53 alone: a note is stale when the conversation shows work on the card after what the note describes. The note's age and author are never the reason. Missing notes are not scored, since the CLI decides them (`latest note: none`).

## Before you start

1. Open a herdr pane and prepare the scratch environment (XDG folders under `~/bs-test`, work board `~/bs-test/data/brain-swap/work`, the four skills already in `~/.claude/skills`):

   ```
   source ~/bs-test/env.sh
   cd ~/bs-test
   mkdir -p sp8
   ```

2. Every script creates its own card with `brain-swap new "<title>"`, so the scripts never disturb each other or the cards W-1 to W-15 and H-1. The command prints `created W-nn: <title> (Todo, work)`. Write `W-nn` down: every `<ID>` in that script means this ID. Run in order on an untouched board, S1 gets W-16 and S10 gets W-25, but always trust the printed ID.
3. Start Claude Code with `claude` in `~/bs-test`, in the same pane, after step 1.
4. Type each prompt exactly as written, one at a time, and wait for Claude to finish before the next. Approve any permission prompt Claude raises for a work prompt (writing a file under `~/bs-test/sp8`, running `date`); approvals are not prompts and do not change the script.
5. Each script takes under ten minutes. Work prompts write small files under `~/bs-test/sp8`; no real code is involved.
6. Score only Claude's reply to the final `/bs-back`. The inline `brain-swap context` at the top of the skill runs before Claude sees the prompt; it is not a tool call.

## Expected output forms

Fresh (FR-51), no tool call:

```
<ID> <title> (note <age> old)
Doing: ...
Next: ...
Watch out: ...
```

Stale (FR-52), exactly one tool call `brain-swap park --card <ID> --session ... --auto` with the catch-up note on stdin, and the reply:

```
no note since <age> ago on <ID>
<ID> <title> (note just saved)
Doing: ...
Next: ...
Watch out: ...
```

A verdict is right when Claude's action matches the ground truth (A-E0-14): no park for fresh, exactly one `park --auto` for stale. Wording that differs from these forms (an age under a minute prints as `now`, which gives `no note since now ago`) goes to the remarks column as a deviation and does not change the score.

## The ten scripts at a glance

| Script | Case | Ground truth |
|---|---|---|
| S1 | by this session, a work prompt after the park | stale |
| S2 | by this session, only an unrelated prompt after the park | fresh |
| S3 | by this session, the park is the last tool call | fresh |
| S4 | empty conversation after `/clear`, reference restored by a pick | fresh |
| S5 | new session linked to another session's note | fresh |
| S6 | another session's note, this conversation shows later work on the card | stale |
| S7 | user-worded note without `auto`, followed by work | stale |
| S8 | note aged 40 minutes by hand, with later work (AS-2) | stale |
| S9 | two parks, with work after the second | stale |
| S10 | user-worded note followed at once by `/bs-back` | fresh |

## S1: work after this session's park

- Ground truth: **stale**. Step 6 adds a checklist item after the park of step 5; the note cannot describe it, so the conversation shows work on the card after what the note describes.
- Setup: card `sp8 s1 release checklist`. At `/bs-back` the latest note is the step 5 park: `auto`, by this session, about a minute old.

1. In the shell:

   ```
   brain-swap new "sp8 s1 release checklist"
   ```

2. Start `claude`.
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s1-release.md holding a three-item markdown checklist for releasing version 1.2: tag the commit, build the binaries, announce in the team chat.
   ```
5. ```
   /bs-park
   ```
6. ```
   Add a fourth item to the checklist in ~/bs-test/sp8/s1-release.md: update the changelog.
   ```
7. ```
   /bs-back
   ```

Expected: stale form, `no note since <age> ago on <ID>`, one `park --auto`, then `<ID> sp8 s1 release checklist (note just saved)`; the saved note mentions the changelog item.

## S2: only an unrelated prompt after this session's park

- Ground truth: **fresh**. The only prompt after the park asks for the weekday, which is not work on the card; the note still describes all card work in the conversation.
- Setup: card `sp8 s2 shopping list`. At `/bs-back` the latest note is the step 5 park: `auto`, by this session. The `date` call of step 6 means the park is no longer the last tool call, so this script also tells the 8.4 rule from the stricter 8.6 rule (which would call it stale, a wrong verdict).

1. In the shell:

   ```
   brain-swap new "sp8 s2 shopping list"
   ```

2. Start `claude`.
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s2-shopping.md with a shopping list of five items for a pasta dinner.
   ```
5. ```
   /bs-park
   ```
6. ```
   Unrelated to the card: run date and tell me today's weekday.
   ```
7. ```
   /bs-back
   ```

Expected: fresh form, no tool call, first line `<ID> sp8 s2 shopping list (note <age> old)`.

## S3: the park is the last tool call

- Ground truth: **fresh**. Nothing happens after the park; the note describes all card work in the conversation.
- Setup: card `sp8 s3 meeting agenda`. At `/bs-back` the latest note is the step 5 park: `auto`, by this session, under a minute old.

1. In the shell:

   ```
   brain-swap new "sp8 s3 meeting agenda"
   ```

2. Start `claude`.
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s3-agenda.md with a four-point agenda for a 30 minute team retro.
   ```
5. ```
   /bs-park
   ```
6. ```
   /bs-back
   ```

Expected: fresh form, no tool call, first line `<ID> sp8 s3 meeting agenda (note <age> old)`.

## S4: empty conversation after /clear, reference restored by a pick

- Ground truth: **fresh**. After `/clear` the conversation is empty, so it shows no work on the card after the note.
- Setup: card `sp8 s4 packing list`. At `/bs-back` the latest note is the step 5 park, `auto`, written by the session before `/clear` (`by another session` once the pick links the new one).

1. In the shell:

   ```
   brain-swap new "sp8 s4 packing list"
   ```

2. Start `claude`.
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s4-packing.md with a packing list of six items for a weekend hike.
   ```
5. ```
   /bs-park
   ```
6. ```
   /clear
   ```
7. ```
   /bs-back
   ```

   Claude shows the numbered card list and asks "Back to which card?". If no list appears, `/clear` kept the session ID: write that in remarks and score the reply as it stands.
8. Type the number printed in front of `<ID>` in the list (usually `1`).

Expected: Claude runs exactly one tool call, `brain-swap link <ID> --session ...` (the pick, not a verdict), and no park; then the fresh form with first line `<ID> sp8 s4 packing list (note <age> old)`.

## S5: new session linked to another session's note

- Ground truth: **fresh**. Session B shows only the link and its printed note; it shows no work on the card after what the note describes.
- Setup: card `sp8 s5 reading list`. At `/bs-back` the latest note is session A's park: `auto`, `by another session`.

1. In the shell:

   ```
   brain-swap new "sp8 s5 reading list"
   ```

2. Start `claude` (session A).
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s5-reading.md listing three books on software design, one per line with its author.
   ```
5. ```
   /bs-park
   ```
6. ```
   /exit
   ```
7. Start `claude` again in the same pane (session B).
8. ```
   /bs-link <ID>
   ```
9. ```
   /bs-back
   ```

Expected: fresh form, no tool call, first line `<ID> sp8 s5 reading list (note <age> old)`.

## S6: another session's note, later work in this conversation

- Ground truth: **stale**. Session B fills the Steps section after session A's note; the note cannot describe it.
- Setup: card `sp8 s6 bug report`. At `/bs-back` the latest note is session A's park: `auto`, `by another session`.

1. In the shell:

   ```
   brain-swap new "sp8 s6 bug report"
   ```

2. Start `claude` (session A).
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s6-bug.md with a bug report template that has three empty sections: Steps, Expected, Actual.
   ```
5. ```
   /bs-park
   ```
6. ```
   /exit
   ```
7. Start `claude` again in the same pane (session B).
8. ```
   /bs-link <ID>
   ```
9. ```
   Fill the Steps section of ~/bs-test/sp8/s6-bug.md with three steps that reproduce a login page hanging after submit.
   ```
10. ```
    /bs-back
    ```

Expected: stale form, `no note since <age> ago on <ID>`, one `park --auto`, then `<ID> sp8 s6 bug report (note just saved)`; the saved note mentions the filled Steps section.

## S7: user-worded note without auto, followed by work

- Ground truth: **stale**. Step 6 fills the Accounts heading after the note of step 5; the note names that as Next, so it does not describe it as done.
- Setup: card `sp8 s7 onboarding note`. At `/bs-back` the latest note is the step 5 park in the user's words: no `auto`, by this session.

1. In the shell:

   ```
   brain-swap new "sp8 s7 onboarding note"
   ```

2. Start `claude`.
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s7-onboarding.md with an outline of three headings for a new hire's first day: Accounts, Tools, People.
   ```
5. ```
   /bs-park doing: wrote the onboarding outline, next: fill the Accounts heading, watch: keep it under 20 lines
   ```

   Optional check in a second shell: `grep '^### ' ~/bs-test/data/brain-swap/work/<ID>.md` shows a stamp line without `auto`. If it ends in `auto`, write that in remarks and go on.
6. ```
   Fill the Accounts heading in ~/bs-test/sp8/s7-onboarding.md with two bullet points: email and VPN.
   ```
7. ```
   /bs-back
   ```

Expected: stale form, `no note since <age> ago on <ID>`, one `park --auto`, then `<ID> sp8 s7 onboarding note (note just saved)`; the saved note says the Accounts heading is filled.

## S8: note aged 40 minutes by hand, with later work (AS-2)

- Ground truth: **stale**. Step 6 adds a total row after the park of step 5; the note cannot describe it. The 40 minute age only reproduces AS-2; it is not the reason.
- Setup: card `sp8 s8 trip budget`. At `/bs-back` the latest note is the step 5 park: `auto`, by this session, its stamp moved 40 minutes back by hand in step 7.

1. In the shell:

   ```
   brain-swap new "sp8 s8 trip budget"
   ```

2. Start `claude`.
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s8-budget.md with a budget table for a three-day trip: travel, hotel and food, each with an amount in euros.
   ```
5. ```
   /bs-park
   ```
6. ```
   Add a total row to the table in ~/bs-test/sp8/s8-budget.md.
   ```
7. Age the note by 40 minutes, in a second shell (another herdr pane), not with `!` inside Claude Code, which would put the command into the conversation. Or ask the lead to make this edit.
   - File: `~/bs-test/data/brain-swap/work/<ID>.md`.
   - Line: the last line starting with `### ` (under `## Timeline`; the card has only this one note), for example `### 2026-10-15T10:12:40+03:00 auto`.
   - Edit: subtract 40 minutes from the time and keep the date (adjust it only if you cross midnight), the seconds, the `+03:00` offset and ` auto`. The example becomes `### 2026-10-15T09:32:40+03:00 auto`.
   - The same edit as commands (set `id` to the card ID first):

     ```
     id=W-nn
     f=~/bs-test/data/brain-swap/work/$id.md
     old=$(grep '^### ' "$f" | tail -1 | cut -d' ' -f2)
     new=$(date -d "$old 40 min ago" +%Y-%m-%dT%H:%M:%S%:z)
     sed -i "s/^### $old/### $new/" "$f"
     grep '^### ' "$f"
     ```

8. Back in Claude Code, at once:

   ```
   /bs-back
   ```

Expected: stale form, `no note since 40 min ago on <ID>` (41 is fine), one `park --auto`, then `<ID> sp8 s8 trip budget (note just saved)`; the saved note mentions the total row.

## S9: two parks, work after the second

- Ground truth: **stale**. Step 8 adds the room line after the second park; the latest note cannot describe it.
- Setup: card `sp8 s9 workshop plan`. At `/bs-back` the latest note is the second park (step 7): `auto`, by this session; the first park (step 5) is older.

1. In the shell:

   ```
   brain-swap new "sp8 s9 workshop plan"
   ```

2. Start `claude`.
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s9-workshop.md with a title and a two-item schedule for a 2 hour workshop: intro, hands-on.
   ```
5. ```
   /bs-park
   ```
6. ```
   Add a third item, Q&A, to the schedule in ~/bs-test/sp8/s9-workshop.md.
   ```
7. ```
   /bs-park
   ```
8. ```
   Add the line "Room: B2" right under the title in ~/bs-test/sp8/s9-workshop.md.
   ```
9. ```
   /bs-back
   ```

Expected: stale form, `no note since <age> ago on <ID>`, one `park --auto`, then `<ID> sp8 s9 workshop plan (note just saved)`; the saved note mentions the room line.

## S10: user-worded note followed at once by /bs-back

- Ground truth: **fresh**. Nothing happens after the park; the note describes all card work in the conversation, in the user's words.
- Setup: card `sp8 s10 newsletter draft`. At `/bs-back` the latest note is the step 5 park in the user's words: no `auto`, by this session.

1. In the shell:

   ```
   brain-swap new "sp8 s10 newsletter draft"
   ```

2. Start `claude`.
3. ```
   /bs-link <ID>
   ```
4. ```
   Create ~/bs-test/sp8/s10-newsletter.md with a three-sentence newsletter intro about the office moving to a new floor.
   ```
5. ```
   /bs-park doing: drafted the newsletter intro, next: add the move date, watch: confirm the floor number with facilities
   ```
6. ```
   /bs-back
   ```

Expected: fresh form, no tool call, first line `<ID> sp8 s10 newsletter draft (note <age> old)`, with the three parts as typed in step 5.

## Score: the 8.4 rule

Run: 2026-10-14, Claude Code 2.1.296 (launched through mise), herdr 0.8.2, brain-swap installed from main 139c57e (`/bs-back` unchanged since E3-F1), skills copied by hand into `~/.claude/skills`, scratch board `~/bs-test`. Each script ran in a fresh Claude Code session in one herdr pane; the lead scored each reply from the session transcript and made the S8 stamp edit.

In "tool calls" list every tool call of the `/bs-back` reply (S4 also lists its `link`); in "output lines" copy the reply's lines verbatim.

| Script | Ground truth | Verdict | Right | Tool calls | Output lines | Remarks |
|---|---|---|---|---|---|---|
| S1 | stale | stale | right | 1: park --card W-16 --auto | "The last note was saved just now, but it was out of date, so I saved a new one." / "W-16 sp8 s1 release checklist (note just saved)" + 3 parts in a code block | deviation: first line not `no note since <age> ago on W-16` (note under a minute old; Claude paraphrased); reply wrapped in a code fence |
| S2 | fresh | fresh | right | 0 | "W-17 sp8 s2 shopping list (note under a minute old)" + 3 parts | deviation: age printed as "under a minute old" instead of the CLI age form |
| S3 | fresh | fresh | right | 0 | "W-18 sp8 s3 meeting agenda (note less than a minute old)" + 3 parts | deviation: age as "less than a minute old" |
| S4 | fresh | fresh | right | 1: link W-19 (the pick), no park | picker verbatim with "Back to which card?"; after `1`: "W-19 sp8 s4 packing list (note under a minute old)" + 3 parts | /clear changed the session ID (picker shown, W-19 first `linked here`); age wording "under a minute old" |
| S5 | fresh | fresh | right | 0 (session B; its only call was the /bs-link link) | "W-20 sp8 s5 reading list (note now old)" + 3 parts, in a code fence | deviation: "note now old" (the `now` age form under a minute); code fence |
| S6 | stale | stale | right | 1 Bash call: `date -Iseconds; brain-swap park --card W-21 ... --auto` (one park) | "The last note on W-21 was saved under a minute ago, but it didn't include the Steps section I just filled in, so I saved a catch-up note." / "W-21 sp8 s6 bug report (note just saved)" + 3 parts in a code fence | deviation: first line paraphrased, not `no note since <age> ago on W-21`; an extra `date` chained before the park; code fence |
| S7 | stale | stale | right | 1: park --card W-22 --auto | "no note since now ago on W-22" / then the just-saved note | user-worded park saved without auto (12:36:16), catch-up auto (12:36:35); deviation: "now ago" wording (age under a minute) |
| S8 | stale | stale | right | 1: park --card W-23 --auto | "no note since 40 min ago on W-23" / "W-23 sp8 s8 trip budget (note just saved)" + 3 parts | exact FR-52 form; stamp aged by the lead with the script snippet |
| S9 | stale | stale | right | 1: park --card W-24 --auto | "no note since just now on W-24" / "W-24 sp8 s9 workshop plan (note just saved)" + 3 parts in a code fence | deviation: "just now" instead of `<age> ago`; code fence; the note mentions Room: B2 |
| S10 | fresh | fresh | right | 0 | "W-25 sp8 s10 newsletter draft (note less than a minute old)" + the 3 user-worded parts | user note without auto; deviation: "less than a minute old" |

Right: 10 of 10.

Wording deviations from FR-51 and FR-52 (they do not change the score; filed as E3-F1-T11):

- Age under a minute: the CLI prints the age `now`, and the replies improvised around it: `(note now old)` (S5), `(note under a minute old)` (S2, S4), `(note less than a minute old)` (S3, S10), `no note since now ago` (S7), `no note since just now` (S9).
- Stale first line paraphrased instead of `no note since <age> ago on <ID>`: S1 ("The last note was saved just now, but it was out of date, so I saved a new one.") and S6.
- The printed note wrapped in a code fence: S1, S5, S6, S9.
- S6 chained a `date -Iseconds` before the one park in the same Bash call.
