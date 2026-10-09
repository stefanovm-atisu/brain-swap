# E0 Spikes

Milestone: M0 (E0-F1, E0-F2), M2 (E0-F3). The spikes answer, on the author's machine with herdr 0.8.2 and Claude Code 2.1.294, the TECHSPEC section 14 questions that the design cannot settle on paper: how the session ID, permissions, `PATH`, `HERDR_*` and skill symlinks behave inside Claude Code (E0-F1), how a herdr popup, focus and the board key behave (E0-F2), and whether the shipped `/bs-back` judges staleness well enough (E0-F3). E0-F1 and E0-F2 come first because E3-F1, E5-F1, E5-F2 and E5-F4 build on their answers (15.1); E0-F3 needs the shipped pack and runs in M2. A spike writes no product code: it runs throwaway probes, records its answer in TECHSPEC section 16 and, when a fallback is taken, updates the sections its feature names. Spikes are exempt from the 12.2 definition of done; each task's DoD below is its own. Spec: TECHSPEC 14, 15, 16 (and 6.4, 7.7, 8, 9.1, 9.3, 9.4 where a fallback lands).

## Features

| ID | Title | Depends on | Tasks | Size |
|---|---|---|---|---|
| E0-F1 | Claude spike | none | 4 | L |
| E0-F2 | herdr spike | none | 3 | L |
| E0-F3 | Staleness spike | E3-F1 | 3 | M |

## E0-F1 Claude spike

- Depends on: none
- Covers: FR-41, FR-46, FR-57, FR-67
- Spec: TECHSPEC 14 (SP-1, SP-3, SP-4, SP-6), 2.2, 6.4, 8.1, 8.8, 9.1, 16
- Scope: Four time-boxed probes in a herdr pane with a real Claude Code session. SP-1 checks how `${CLAUDE_SESSION_ID}` expands in a user skill and whether it matches herdr's `agent_session.value` across `/compact`, auto-compaction and `/clear`. SP-3 checks whether `allowed-tools: Bash(brain-swap:*)` pre-approves the inline and heredoc calls of section 8 and whether `~/.cargo/bin` is on the inline shell's `PATH`. SP-4 checks `HERDR_*` inheritance and captures the JSON of the herdr commands 9.2 and 9.3 read, including after a pane move; SP-6 checks bare skill names through symlinks, model invocation and `$CLAUDE_CONFIG_DIR/skills`. Each answer becomes one `SP-n` line in TECHSPEC 16, and a taken fallback updates 6.4, 8 or 9.1 in the same commit.
- Provides: TECHSPEC 16 answer lines `SP-1`, `SP-3`, `SP-4`, `SP-6` (read by E3-F1, E3-F2, E5-F1, E5-F2); under `SP-4`, trimmed JSON samples of `herdr pane get` (agent pane, shell pane, `pane_not_found` stderr), `herdr workspace list` and `herdr pane list --workspace <id>` for the E5-F1 fake herdr script to answer with; under `SP-1`, the literal text a failed `${CLAUDE_SESSION_ID}` substitution arrives as, checked against I8.
- Requires: none
- Feature DoD:
  - [ ] SP-1 answered in TECHSPEC 16; 6.4 and the `--session` rule of 8.1 updated if the fallback is taken (E0-F1-T1).
  - [ ] SP-3 answered in TECHSPEC 16; 8.1 and 8.8 updated if a fallback is taken (E0-F1-T2).
  - [ ] SP-4 answered in TECHSPEC 16 with the JSON samples; 9.1 and the `pane current` clause of 2.2 updated (E0-F1-T3).
  - [ ] SP-6 answered in TECHSPEC 16; 8.8 updated if the fallback is taken (E0-F1-T4).

### E0-F1-T1 SP-1 session ID in skills

- Status: todo
- Depends on: none
- Covers: FR-41, FR-57; TECHSPEC 14 SP-1, 6.4, 8.1, 8.7, 16
- Size: S
- Scope: Answer SP-1: does `${CLAUDE_SESSION_ID}` expand in a user skill's inline commands and in its body, does it equal herdr's `agent_session.value` for the pane, does it survive auto-compaction, and does it change on `/clear` as FR-57 expects? The probe is a throwaway user skill `~/.claude/skills/bs-probe-sid/SKILL.md` (a plain directory, A-E0-04) with `disable-model-invocation: true`, `allowed-tools: Bash(echo:*)`, the inline line ``Inline: !`echo "sid=${CLAUDE_SESSION_ID}"` ``, the body line `Body: ${CLAUDE_SESSION_ID}` and the instruction "Reply with the Inline and Body lines verbatim." Two hours at most, no product code.
- Not in scope: the 6.4 resolution chain in code (E1-F7, E2-F3); `HERDR_*` inheritance in Claude's shells (E0-F1-T3); skill symlinks (E0-F1-T4).
- Procedure:
  1. In a herdr pane, note `HERDR_PANE_ID`, write the probe skill, start `claude` and run `/bs-probe-sid`; record the inline and the body value.
  2. Ask Claude to run `herdr pane get "$HERDR_PANE_ID"` with the Bash tool; record `result.pane.agent_session.value` and whether it equals both values of step 1.
  3. Run `/compact`, then `/bs-probe-sid` and the step 2 command again; record whether all three values still equal step 1.
  4. Fill the context until Claude Code compacts on its own (for example, have Claude read a generated `seq 1 400000` file in chunks), then repeat the step 3 checks. If auto-compaction is not reached within the box, record `auto-compaction: not reached` and let the step 3 result stand for compaction (A-E0-05).
  5. Run `/clear`, then `/bs-probe-sid` and the step 2 command; record the new ID, that it differs from step 1, and whether herdr reports the new one.
  6. Run `claude -p "/bs-probe-sid"` in the same folder; record whether the variable is substituted there and, if not, the exact text the inline command received, and check it against I8 (empty, or containing `$`, `{` or `}`, is a placeholder).
  7. Decide: if steps 1 to 3 hold, the 6.4 chain stays as written and no fallback is needed; if expansion fails anywhere the pack runs, the fallback is the 6.4 chain (`BRAIN_SWAP_SESSION`, then herdr's agent session), and 6.4 and the `--session` rule of 8.1 say which step the pack relies on.
  8. Remove `~/.claude/skills/bs-probe-sid/` and the generated file.
- DoD:
  - [ ] TECHSPEC 16 gains the line `SP-1 (<date>, herdr 0.8.2, Claude Code 2.1.294)` answering each of: inline expands, body expands, equals herdr, survives `/compact`, survives auto-compaction (or `not reached`), changes on `/clear`, unsubstituted text and its I8 verdict, fallback taken or `none`; `SP-1` is removed from the `Open until the spikes answer` sentence (A-E0-01).
  - [ ] If the fallback is taken, 6.4 and 8.1 are updated in the same commit; an answer that contradicts FR-41, FR-57 or A-14 is named in the line and the PRD is left for the owner (A-E0-02).
  - [ ] The probe skill is gone from `~/.claude/skills/`; no file under `src/`, `pack/`, `templates/` or `tests/` changed.
  - [ ] One commit saying what was found and why, with no AI attribution.

### E0-F1-T2 SP-3 permissions and PATH in Claude

- Status: todo
- Depends on: none
- Covers: FR-46; TECHSPEC 14 SP-3, 8.1, 8.8, 16
- Size: S
- Scope: Answer SP-3: does `allowed-tools: Bash(brain-swap:*)` pre-approve the inline `context` call and the Bash tool heredoc calls of section 8, and is `~/.cargo/bin` on the inline shell's `PATH`? The probes (A-E0-04) are a stub `~/.cargo/bin/brain-swap` shell script, installed only while no real binary is there, that appends its argv, stdin, `PATH` and `HERDR_*` to a log in the scratch folder, prints `brain-swap context v1` and exits 0; and a throwaway skill `~/.claude/skills/bs-probe-perm/SKILL.md` with the 8.3 frontmatter (`disable-model-invocation: true`, `allowed-tools: Bash(brain-swap:*)`), the inline line ``!`brain-swap context --session "${CLAUDE_SESSION_ID}"` `` and a body telling Claude to run, with the Bash tool, the 8.3 heredoc `brain-swap park --card W-1 --session "${CLAUDE_SESSION_ID}" --auto <<'BS_EOF'` with three part lines, then `brain-swap move W-1 Doing`. Two hours at most, no product code.
- Not in scope: the real skills (E3-F1); the absolute-path rewrite in `install claude` (E3-F2), which this spike only decides on.
- Procedure:
  1. Confirm `~/.cargo/bin/brain-swap` does not exist and install the stub; record whether the user's Claude settings already hold an allow rule for `brain-swap` (read only, never edit settings).
  2. In a herdr pane start `claude` and run `/bs-probe-perm`; for the inline call, the heredoc `park` and the `move`, record whether a permission prompt appeared and whether the stub logged the call, with the heredoc text intact on stdin.
  3. From the log, record the `PATH` of the inline shell and of the Bash tool and whether each contains `~/.cargo/bin`.
  4. After the skill turn ends, ask in plain words "move W-1 to Done"; record whether `brain-swap move W-1 Done` runs without a prompt (the FR-47 move runs outside the skill turn, and 8.8 suggests an allow rule for it).
  5. If step 3 shows `~/.cargo/bin` missing, repeat step 2 with the stub's absolute path in the inline line, in the body commands and in `allowed-tools: Bash(<absolute path>:*)`; record whether that pre-approves.
  6. Decide: no fallback when steps 2 and 3 pass; otherwise the documented allow rule (8.8 and README) against prompts, and absolute paths in the installed copies (8.8) against a missing `PATH`.
  7. Remove the stub and `~/.claude/skills/bs-probe-perm/`.
- DoD:
  - [ ] TECHSPEC 16 gains the line `SP-3 (<date>, Claude Code 2.1.294)` with a yes or no for: inline pre-approved, heredoc pre-approved, later move pre-approved, `~/.cargo/bin` on the inline `PATH`, on the Bash tool `PATH`, absolute path pre-approved (or `not tried`); `SP-3` is removed from the `Open until the spikes answer` sentence.
  - [ ] If a fallback is taken, 8.1 and 8.8 say which one (allow rule, absolute paths) in the same commit.
  - [ ] Stub and probe skill removed; no product file changed; one commit with no AI attribution.

### E0-F1-T3 SP-4 herdr from Claude

- Status: todo
- Depends on: none
- Covers: FR-41; TECHSPEC 14 SP-4, 2.2, 9.1, 9.2, 9.3, 16
- Size: S
- Scope: Answer SP-4: does Claude's Bash tool inherit `HERDR_*` (and does the inline shell, since 6.4 step 3 also runs inside the inline `context` call, A-E0-06); what exactly do `herdr workspace list`, `herdr pane list --workspace <id>` and `herdr pane get` print; and after a pane moves to another workspace, does `herdr pane current --current` resolve the caller's new pane or only echo `HERDR_PANE_ID`? The JSON samples, trimmed to the keys 9.2 and 9.3 read, become the reference for the E5-F1 fake herdr script. Two hours at most, no product code.
- Not in scope: `Runner`, `capture`, `pane_session` and the fake herdr script (E5-F1); `locate` and `focus` (E5-F2); focus semantics (E0-F2-T2).
- Procedure:
  1. In a herdr pane run `env | grep '^HERDR_'` in the plain shell and record the values; start `claude` there.
  2. Ask Claude to run `env | grep '^HERDR_'` with the Bash tool; record which of `HERDR_ENV`, `HERDR_PANE_ID`, `HERDR_TAB_ID`, `HERDR_WORKSPACE_ID`, `HERDR_BIN_PATH` and `HERDR_SOCKET_PATH` arrive and whether they equal step 1.
  3. Run a throwaway skill `~/.claude/skills/bs-probe-env/SKILL.md` (`allowed-tools: Bash(printenv:*)`, inline ``!`printenv HERDR_ENV HERDR_PANE_ID HERDR_TAB_ID HERDR_WORKSPACE_ID` ``); record what the inline shell sees.
  4. Record verbatim, with exit codes: `herdr workspace list`; `herdr pane list --workspace <id>` for a workspace holding a Claude pane and a plain-shell pane; `herdr pane get <claude pane>`; `herdr pane get <shell pane>`; `herdr pane get <an ID that never existed>` (stdout, the stderr JSON and its `error.code`, exit code).
  5. Check the keys 9.2 and 9.3 rely on: `result.pane`, the pane, tab and workspace ID keys, `agent`, `agent_session.value`, `agent_status`, the workspace ID key of the list, and `pane_not_found` in the stderr JSON; record every one that differs.
  6. Move the Claude pane with `herdr pane move <pane> --new-workspace`; record its new ID (`herdr pane list --workspace <new id>`), that the old ID now gives `pane_not_found`, and whether `agent_session.value` is unchanged on the new pane.
  7. In the moved pane ask Claude to run `herdr pane current --current` and `herdr pane get "$HERDR_PANE_ID"`; record whether the first names the new pane or the stale `HERDR_PANE_ID`.
  8. Decide: if step 7 resolves the new pane, the 9.1 moved-pane check is enabled (2.2 keeps `pane current`, T-15 allows the extra call); otherwise the fallback: notes from a moved pane keep the old ID, and 2.2 and 9.1 drop `pane current`.
  9. Remove the probe skill.
- DoD:
  - [ ] TECHSPEC 16 gains the line `SP-4 (<date>, herdr 0.8.2, Claude Code 2.1.294)` answering inheritance (Bash tool, inline shell), the move (new ID, session kept) and `pane current --current`, followed by the step 4 samples trimmed to the step 5 keys; `SP-4` is removed from the `Open until the spikes answer` sentence.
  - [ ] 9.1 (and 2.2, and 9.3 where a key name differs) updated in the same commit; a contradiction with FR-15, FR-60 or A-20 is named for the owner (A-E0-02).
  - [ ] Probe skill removed; no product file changed; one commit with no AI attribution.

### E0-F1-T4 SP-6 skill install by symlink

- Status: todo
- Depends on: none
- Covers: FR-46, FR-67; TECHSPEC 14 SP-6, 2.2, 8.1, 8.8, 16
- Size: S
- Scope: Answer SP-6: does a symlinked `~/.claude/skills/<name>/` give a bare `/<name>` without a restart that Claude cannot invoke on its own, and does Claude Code load skills from `$CLAUDE_CONFIG_DIR/skills` when that is set? The probe (A-E0-04) is a skill `bs-probe-link` written twice in the scratch folder, `pack/v1/skills/bs-probe-link/SKILL.md` and `pack/v2/skills/bs-probe-link/SKILL.md`, with the 8.1 frontmatter (`disable-model-invocation: true`) and a body that replies `PROBE v1` or `PROBE v2`, mirroring the versioned copy of 8.8. Two hours at most, no product code.
- Not in scope: `install claude` (E3-F2), which implements what this spike settles.
- Procedure:
  1. Start `claude` in a herdr pane while `~/.claude/skills/` exists; while it runs, link `~/.claude/skills/bs-probe-link` to the v1 copy; record whether `/bs-probe-link` appears as a bare name and replies `PROBE v1` without a restart. If not, try `/reload-skills` and record whether that command exists and helps.
  2. Ask in plain words "run the bs-probe-link skill" and "which skills can you invoke?"; record that Claude neither invokes nor lists it (FR-46).
  3. Replace the link atomically with one to the v2 copy (`ln -s <v2> tmp && mv -T tmp ~/.claude/skills/bs-probe-link`); run `/bs-probe-link` in the same session and record `v1` or `v2`.
  4. Start `CLAUDE_CONFIG_DIR=<scratch>/ccd claude` (log in if asked; if that is impossible within the box, record `unanswered`, A-E0-03); with no `skills/` folder there at start, create `<scratch>/ccd/skills/bs-probe-link` as a link to v1 during the session; record whether it loads, with or without `/reload-skills`, and whether `/bs-probe-link` from `~/.claude/skills/` is absent there.
  5. Decide: no fallback when step 1 works; otherwise copies (8.8 writes directories instead of links and its rule for "ours" says how a copy is recognised). If step 4 says no, `CLAUDE_CONFIG_DIR` is dropped from 2.2, 8.8 and T-09 (A-E0-07).
  6. Remove the links, the scratch copies and `<scratch>/ccd`.
- DoD:
  - [ ] TECHSPEC 16 gains the line `SP-6 (<date>, Claude Code 2.1.294)` with a yes or no for: bare name via symlink, no restart (or `/reload-skills` needed), model cannot invoke, retargeted link picked up, `$CLAUDE_CONFIG_DIR/skills` loaded.
  - [ ] If a fallback is taken, 8.8 (and 2.2, T-09, T-12 as affected) updated in the same commit; a contradiction with FR-67, A-22 or A-26 is named for the owner (A-E0-02).
  - [ ] Probe links and copies removed; no product file changed; one commit with no AI attribution.

## E0-F2 herdr spike

- Depends on: none
- Covers: FR-59, FR-60, FR-64
- Spec: TECHSPEC 14 (SP-2, SP-5, SP-7), 7.7, 9.3, 9.4, 16
- Scope: Three probes against the running herdr 0.8.2 server and the author's herdr config. SP-2 binds a probe script as a `popup` key command and checks what the popup's command sees (`HERDR_*`, `PATH`), whether the popup closes when its command exits, and whether a focus issued from inside it holds after it closes. SP-5 checks `herdr agent focus` on a pane without an agent and whether `workspace focus` and `tab focus` cross workspaces, which reaching a moved pane (FR-60) needs. SP-7 checks that `prefix+alt+b` is free. Answers go to TECHSPEC 16, and 7.7, 9.3 and 9.4 are updated to match.
- Provides: TECHSPEC 16 answer lines `SP-2`, `SP-5`, `SP-7` (read by E5-F2, E5-F3, E5-F4); under `SP-5`, the exit codes and stderr of failing `agent focus`, `workspace focus` and `tab focus` calls for the E5-F2-T9 `focus-fails` scenario and the E5-F2-T5 failure tests; under `SP-7`, the confirmed key for the 9.4 snippet.
- Requires: none
- Feature DoD:
  - [ ] SP-2 answered in TECHSPEC 16; 7.7 and the 9.4 snippet updated (focus before or after exit, an `env` prefix if `HERDR_*` is missing) (E0-F2-T1).
  - [ ] SP-5 answered in TECHSPEC 16; `focus` in 9.3 updated (E0-F2-T2).
  - [ ] SP-7 answered in TECHSPEC 16; the 9.4 key confirmed or replaced (E0-F2-T3).

### E0-F2-T1 SP-2 jump from a popup

- Status: todo
- Depends on: none
- Covers: FR-59, FR-64; TECHSPEC 14 SP-2, 7.7, 9.4, 16
- Size: S
- Scope: Answer SP-2: does a herdr `popup` command see `HERDR_*` and the user's `PATH`, does the popup close when its command exits, and does `herdr agent focus` issued from it hold after the popup exits? Because the FR-59 fallback path also runs from the popup, `workspace focus` plus `tab focus` is checked the same way (A-E0-08). The probe (A-E0-04) is a script in the scratch folder, bound by absolute path as a `[[keys.command]]` with `type = "popup"`, `width = "90%"` and `height = "90%"` (the 9.4 shape) on a key free in the author's config. It appends `env` to a log, then reads one key: `q` exits; `a` runs `herdr agent focus <target pane>` and exits; `t` runs `herdr workspace focus <ws>` and `herdr tab focus <tab>` and exits; `d` starts `setsid -f sh -c 'sleep 0.3; herdr agent focus <target pane>'` and exits at once; every branch logs its exit codes. Two hours at most, no product code.
- Not in scope: the TUI jump (E5-F3); `install herdr` (E5-F4); the key choice (E0-F2-T3).
- Procedure:
  1. Back up `~/.config/herdr/config.toml`, add the probe binding, run `herdr config check` and `herdr server reload-config`.
  2. Prepare a target Claude pane in another workspace and a target plain-shell pane in another tab; write their pane, tab and workspace IDs into the probe's target file.
  3. Press the key; from the log record `HERDR_ENV`, `HERDR_PANE_ID` (which pane the popup claims), `HERDR_TAB_ID`, `HERDR_WORKSPACE_ID`, `HERDR_BIN_PATH`, `HERDR_SOCKET_PATH`, and whether `PATH` contains `~/.cargo/bin`.
  4. Press `q`; record whether the popup closes when its command exits.
  5. Reopen and press `a`; record the exit code and which pane holds focus once the popup has closed.
  6. Reopen and press `t` (targeting the shell pane's workspace and tab); record whether that tab holds focus afterwards.
  7. Reopen and press `d`; record whether the detached focus lands after the popup has closed.
  8. Decide: if step 5 holds, 7.7 stays (focus, then exit); if only step 7 works, the section 14 fallback (exit first, then a detached `brain-swap jump <ID>`) goes into 7.7, and the line notes for the owner that a detached jump cannot show the 7.7 Message and needs `--note <n>` for a Detail jump; if step 3 lacks `HERDR_*`, the 9.4 snippet becomes `command = "env HERDR_ENV=1 HERDR_BIN_PATH=<herdr path> <brain-swap path>"` (plus `HERDR_SOCKET_PATH` if the herdr CLI needed it).
  9. Restore the herdr config and run `herdr server reload-config`.
- DoD:
  - [ ] TECHSPEC 16 gains the line `SP-2 (<date>, herdr 0.8.2)` with: each popup variable present or not, `~/.cargo/bin` on the popup `PATH` or not, closes on exit (yes or no), agent focus from the popup holds, tab focus from the popup holds, detached focus lands, order chosen; `SP-2` is removed from the `Open until the spikes answer` sentence.
  - [ ] 7.7 and 9.4 updated in the same commit to the chosen order and snippet; a contradiction with FR-59 or A-07 is named for the owner (A-E0-02).
  - [ ] herdr config restored and reloaded; no product file changed; one commit with no AI attribution.

### E0-F2-T2 SP-5 focus across workspaces

- Status: todo
- Depends on: none
- Covers: FR-59, FR-60; TECHSPEC 14 SP-5, 9.3, 16
- Size: S
- Scope: Answer SP-5: what does `herdr agent focus` do on a pane without an agent, and do `herdr workspace focus` and `herdr tab focus` cross workspaces (a moved pane lives in another workspace, FR-60)? Also confirm the 9.3 rule for `LivePane.agent` (an `agent` or `agent_session` key, or `agent_status` other than `unknown`) on real pane JSON, including a pane whose Claude has exited (A-E0-09). Two hours at most, no product code.
- Not in scope: `focus` in code and its `FakeRunner` tests (E5-F2); popup behaviour (E0-F2-T1).
- Procedure:
  1. Arrange workspace A (tab A1: a plain-shell pane) and workspace B (tab B1: a Claude pane; tab B2: a plain-shell pane); record all IDs from `herdr workspace list` and `herdr pane list --workspace <id>`.
  2. From the shell pane in A run `herdr agent focus <shell pane in B2>`; record exit code, stdout, stderr and what holds focus.
  3. From A run `herdr agent focus <Claude pane in B1>`; record whether focus crosses to workspace B and lands on that pane.
  4. From A run `herdr tab focus <B2>` alone; record whether it crosses workspaces.
  5. From A run `herdr workspace focus <B>`, then `herdr tab focus <B2>`; record where focus ends.
  6. Run `herdr agent focus <an ID that never existed>`, `herdr tab focus <a bad ID>` and `herdr workspace focus <a bad ID>`; record exit codes and stderr.
  7. Record `herdr pane get` for the Claude pane, the shell pane, and the Claude pane after quitting Claude in it; check that the `LivePane.agent` rule gives true, false, and the value 9.3 needs for the exited pane.
  8. Decide what `focus` in 9.3 needs: keep or drop `workspace focus` before `tab focus`, and whether the agent rule changes.
- DoD:
  - [ ] TECHSPEC 16 gains the line `SP-5 (<date>, herdr 0.8.2)` with the answers of steps 2 to 7, including the failure exit codes and stderr of step 6.
  - [ ] 9.3 updated in the same commit where a step contradicts it; a contradiction with FR-59, FR-60 or AS-6 (tab focus) is named for the owner (A-E0-02).
  - [ ] No product file changed; one commit with no AI attribution.

### E0-F2-T3 SP-7 board key

- Status: todo
- Depends on: none
- Covers: FR-64; TECHSPEC 14 SP-7, 9.4, 16
- Size: S
- Scope: Answer SP-7: is `prefix+alt+b` free in herdr 0.8.2's defaults and in the author's herdr config, and does the author's terminal and window manager deliver it to herdr? Also check the reason 9.4 gives for it (`prefix+b` is herdr's sidebar toggle). Two hours at most (expected well under), no product code.
- Not in scope: `install herdr` (E5-F4).
- Procedure:
  1. Search `~/.config/herdr/config.toml` for `alt+b` and for the sidebar binding; list herdr 0.8.2's default bindings (`herdr --skill` or the herdr docs) and look up `prefix+alt+b` and `prefix+b` there.
  2. Back up the config, add the 9.4 snippet with `command` set to a probe (`sh -c 'echo key ok; sleep 2'`), run `herdr config check` and `herdr server reload-config`.
  3. Press `ctrl+a`, then `alt+b`, from a Claude pane, a shell pane and with the herdr sidebar focused; record whether the popup opens each time and nothing else fires (window manager, terminal, herdr).
  4. If the key is taken anywhere, pick the nearest free combination and repeat step 3 with it.
  5. Restore the config and reload.
- DoD:
  - [ ] TECHSPEC 16 gains the line `SP-7 (<date>, herdr 0.8.2)` with: free by default (yes or no), free in the author's config, delivered by terminal and window manager, `prefix+b` is the sidebar toggle (yes or no), key chosen; `SP-7` is removed from the `Open until the spikes answer` sentence.
  - [ ] If the key changes, 9.4 is updated and the line names PRD section 6 setup and A-18 for the owner (A-E0-02).
  - [ ] herdr config restored; one commit with no AI attribution.

## E0-F3 Staleness spike

- Depends on: E3-F1
- Covers: FR-52, FR-53
- Spec: TECHSPEC 14 (SP-8), 8.4, 8.6, 16
- Scope: Score the shipped `/bs-back` staleness judgement in 10 scripted sessions, 5 stale and 5 fresh. The scripts are written first, so the ground truth is fixed before any run; one run per script records Claude's verdict and output; if fewer than 9 are right, the stricter 8.6 rule replaces step 3 of `/bs-back` in TECHSPEC 8.4 and in the shipped SKILL.md, and the same scripts run again.
- Provides: `docs/spikes/sp8-staleness.md` (the scripts S1 to S10 with ground truth, and the score tables) (A-E0-10); TECHSPEC 16 answer line `SP-8`; when the fallback is taken, the stricter step 3 in `pack/claude/skills/bs-back/SKILL.md` and TECHSPEC 8.4.
- Requires: E3-F1 `pack/claude/skills/{bs-card,bs-park,bs-back,bs-link}/SKILL.md` as section 8 writes them, and `tests/pack.rs`; E2-F3 `brain-swap context`, `link`, `park` and `show` in a binary installed with `cargo install --path .`.
- Feature DoD:
  - [ ] Ten scripts, five stale and five fresh, with ground truth fixed before the first run (E0-F3-T1).
  - [ ] The shipped `/bs-back` scored and the score recorded in TECHSPEC 16 (E0-F3-T2).
  - [ ] 9 of 10 or more: reached in E0-F3-T2, or with the 8.6 fallback applied to TECHSPEC 8.4 and the shipped SKILL.md and re-scored (E0-F3-T3).

### E0-F3-T1 SP-8 session scripts

- Status: todo
- Depends on: E3-F1-T4
- Covers: FR-53; TECHSPEC 14 SP-8, 8.4, 8.6
- Size: S
- Scope: Write `docs/spikes/sp8-staleness.md` (A-E0-10): ten numbered scripts S1 to S10, each with its ground truth (stale or fresh) and the reason under FR-53 and 8.4 step 3, the setup (card, notes present, which session wrote them, any stamp aged by hand), the exact user prompts in order (including where `/bs-park`, `/bs-link`, `/clear` and the final `/bs-back` go), and the expected `/bs-back` output form (FR-51 for fresh, FR-52 for stale). Five are stale and five fresh, and together they cover: by this session, a work prompt after the park (stale); by this session, only an unrelated prompt after it (fresh); by this session, the park is the last tool call (fresh); an empty conversation after `/clear`, reference restored by a pick (fresh); a new session linked to another session's note (fresh); another session's note, this conversation shows later work on the card (stale); a user-worded note without `auto` followed by work (stale); a note aged 40 minutes by hand, with later work (stale, AS-2); two parks, with work after the second (stale); a user-worded note followed at once by `/bs-back` (fresh). Missing notes are not scored, since the CLI decides them (`latest note: none`).
- Not in scope: running the scripts (E0-F3-T2); the skill text (E3-F1).
- Procedure:
  1. Draft S1 to S10 against the cases above, each runnable in under ten minutes in a herdr pane with the E3-F1 skills linked by hand (A-E0-11).
  2. For each script check that the ground truth follows from FR-53 alone (the conversation shows work on the card after what the note describes), not from the note's age or author.
  3. Add an empty score table with the columns: script, ground truth, verdict, right, tool calls, output lines, remarks.
- DoD:
  - [ ] `docs/spikes/sp8-staleness.md` holds ten scripts, five stale and five fresh, each with ground truth, reason, setup, prompts and expected output form, and together covering every case listed in Scope.
  - [ ] The docs test (no U+2014 or U+2013 in repository markdown) passes; one commit with no AI attribution.

### E0-F3-T2 SP-8 run and score

- Status: todo
- Depends on: E0-F3-T1
- Covers: FR-52, FR-53; TECHSPEC 14 SP-8, 8.4, 16
- Size: S
- Scope: Run S1 to S10 once each against the shipped `/bs-back`: real Claude Code in herdr, a binary installed from the E3-F1 commit with `cargo install --path .`, the four skills symlinked by hand into `~/.claude/skills/` (A-E0-11), and a scratch folder as the work board. A verdict is right when Claude's action matches the ground truth (A-E0-14): fresh means no tool call and the first line `<ID> <title> (note <age> old)`; stale means the line `no note since <age> ago on <ID>`, exactly one `brain-swap park --card <ID> --session ... --auto` call and the first line `<ID> <title> (note just saved)`. Two hours at most.
- Not in scope: changing the skill (E0-F3-T3); fixing FR-51 or FR-52 wording deviations (bug tasks in E3-F1).
- Procedure:
  1. Install the binary, link the four skills, back up `~/.config/brain-swap/` and `~/.local/state/brain-swap/`, and point the work board at a scratch folder.
  2. For each script, set up as written, type the prompts exactly, and fill its row of the score table: verdict, right or wrong, tool calls seen, output lines verbatim.
  3. Count the right verdicts; list wording deviations from FR-51 and FR-52 separately (they do not change the score).
  4. Restore the backups and remove the hand links.
- DoD:
  - [ ] The score table in `docs/spikes/sp8-staleness.md` is filled for S1 to S10.
  - [ ] TECHSPEC 16 gains the line `SP-8 (<date>, Claude Code 2.1.294): <n> of 10 with the 8.4 rule`, linking the table; when `n` is 9 or more it ends `fallback: not needed` and `SP-8` is removed from the `Open until the spikes answer` sentence.
  - [ ] Every wording deviation is filed as a bug task in E3-F1 (next free task number).
  - [ ] One commit with no AI attribution.

### E0-F3-T3 SP-8 fallback rule

- Status: todo
- Depends on: E0-F3-T2
- Covers: FR-53; TECHSPEC 8.4, 8.6, 16
- Size: S
- Scope: Runs only if E0-F3-T2 scored below 9; otherwise it is closed without a change, its DoD met by the `fallback: not needed` of E0-F3-T2. Apply the 8.6 fallback: step 3 of `/bs-back` becomes "current only when the note is `by this session` and its park is the last tool call", keeping the clause that an empty or fresh conversation is current (A-E0-12), in TECHSPEC 8.4 and with the same words in `pack/claude/skills/bs-back/SKILL.md`; then run S1 to S10 again. The change is prompt text only, no Rust code. Two hours at most.
- Not in scope: the other skills; any rule beyond the 8.6 one (A-E0-13).
- Procedure:
  1. Edit TECHSPEC 8.4 step 3 and the SKILL.md step 3 to the same new wording; run `cargo test --all-features` so `tests/pack.rs` still passes.
  2. Reinstall with `cargo install --path .`, relink the skills by hand, and run S1 to S10 as in E0-F3-T2 into a second score table.
  3. If 9 or more are right, the fallback stands; if not, keep whichever rule scored higher, record both scores and stop for the owner's decision (A-E0-13).
- DoD:
  - [ ] Either the task is closed as `not needed: E0-F3-T2 scored 9 or more`, or TECHSPEC 8.4 and the shipped SKILL.md carry the same step 3, `tests/pack.rs` passes and the second score table is filled.
  - [ ] The TECHSPEC 16 `SP-8` line gains `fallback: <n> of 10 with the 8.6 rule`; at 9 or more `SP-8` is removed from the `Open until the spikes answer` sentence, below 9 it stays and the line ends `owner decision needed`.
  - [ ] One commit with no AI attribution.

## Assumptions

- A-E0-01 Does each spike record its answer in TECHSPEC 16 as one bullet `**SP-n** (<date>, <versions>): ...` under a new `Spike answers:` line after T-21, and remove its ID from the `Open until the spikes answer` sentence (which does not list SP-5 and SP-6)? Affects every E0 task.
- A-E0-02 When a spike answer contradicts a PRD line (an FR, an AS or an A-nn), does the spike name that PRD ID in its section 16 line and leave the PRD for the owner, editing only TECHSPEC? Affects every E0 task, notably E0-F2-T3 (PRD section 6 setup and A-18 name `prefix+alt+b`).
- A-E0-03 When a spike's two hours run out with a question open, is that question recorded as `unanswered` and its section 14 fallback taken? Affects every E0 task.
- A-E0-04 May the spikes place throwaway probes on the author's machine (a stub `~/.cargo/bin/brain-swap` while no real binary is installed, `~/.claude/skills/bs-probe-*` skills, temporary herdr key bindings), provided each is removed or restored before the commit? Affects E0-F1-T1 to E0-F1-T4 and E0-F2-T1 to E0-F2-T3.
- A-E0-05 If auto-compaction cannot be reached within the box, does a manual `/compact` stand in for it in SP-1, leaving the "maybe compaction" wording of FR-57 as it is? Affects E0-F1-T1.
- A-E0-06 Does SP-4 also check the inline shell's `HERDR_*`, since 6.4 step 3 (`pane_session`) runs inside the inline `context` call of 8.3 and 8.4? Affects E0-F1-T3.
- A-E0-07 If Claude Code ignores `$CLAUDE_CONFIG_DIR/skills`, is `CLAUDE_CONFIG_DIR` dropped from 2.2, 8.8 and T-09 so that `install claude` always uses `~/.claude/skills/`? Affects E0-F1-T4.
- A-E0-08 Does SP-2 also check `workspace focus` plus `tab focus` issued from the popup, since the FR-59 fallback path runs there too? Affects E0-F2-T1.
- A-E0-09 Does SP-5 confirm the 9.3 `LivePane.agent` rule on real pane JSON, including a pane whose Claude has exited? Affects E0-F2-T2.
- A-E0-10 Do the SP-8 scripts and score tables live in a new `docs/spikes/sp8-staleness.md`, with the TECHSPEC 16 line giving the score and linking the file? Affects E0-F3-T1 to E0-F3-T3.
- A-E0-11 Does SP-8 run on the E3-F1 skills symlinked into `~/.claude/skills/` by hand (as the E3-F1 DoD does for its manual runs), so E0-F3 needs no E3-F2? Affects E0-F3-T1 and E0-F3-T2.
- A-E0-12 Does the 8.6 fallback keep the 8.4 step 3 clause that an empty or fresh conversation means current (without it, a new session returning to another session's note drafts a catch-up from an empty conversation, against FR-53)? Affects E0-F3-T3.
- A-E0-13 If the 8.6 fallback also scores below 9 of 10, does the spike keep whichever rule scored higher, record both scores and stop for the owner's decision instead of inventing a third rule? Affects E0-F3-T3.
- A-E0-14 Is a verdict scored right when Claude's action matches the ground truth (no tool call for current, exactly one `park --auto` for stale), with FR-51 and FR-52 wording deviations recorded separately as E3-F1 bugs rather than counted against the score? Affects E0-F3-T2 and E0-F3-T3.
