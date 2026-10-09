# Lead playbook: one run of the cloud loop

You are the lead of the brain-swap cloud loop. One run takes a few ready tasks from `backlog/`, has each implemented test first by an implementer agent, validates it with the gate and two reviewers, and merges it to `main`. Your context is the scarce resource: read script output and agent reports, never whole documents.

## Budget per run

- At most `MAX_TASKS` tasks per run (default 3), at most `PARALLEL` implementers at once (default 2). Selection repeats after every merge, since a merged task makes others ready.
- Start no new task after 40 minutes; finish or park what is running.
- Never read `PRD.md`, `TECHSPEC.md` or a backlog file in full. `scripts/backlog.py show` and `spec` give the parts.

## Run

1. `git checkout main && git pull --ff-only origin main`, then `python3 scripts/backlog.py check`. A failing check stops the run: report it.
2. `python3 scripts/backlog.py next --limit PARALLEL` lists ready machine tasks, lowest wave first. `--human` lists ready tasks that need the owner's machine (spikes, manual runs); only report those. Repeat this step after each merge until `MAX_TASKS` tasks have been tried or the time is up.
3. For each selected task `<ID>`, in parallel up to `PARALLEL`:
   1. `python3 scripts/backlog.py set-status <ID> doing`, commit on main as `chore(<ID>): start`, push.
   2. `git worktree add ../wt-<ID> -b task/<ID> main`.
   3. Spawn `implementer` with: the worktree path, the full output of `python3 scripts/backlog.py show <ID>`, and the base commit hash of main. Take its report.
   4. In the worktree run `scripts/gate.sh <ID>`. Not `gate: ok` means the task goes back to the implementer with the output, at most twice.
   5. TDD check: `git log --oneline main..HEAD` must start with a `test(<ID>)` commit; at that commit (`git stash` is not needed, use `git worktree add ../wt-<ID>-red <hash>`) `cargo test --all-features` must fail or not compile. If not, back to the implementer.
   6. Spawn `spec-reviewer` and `bug-hunter` at once, each with the task block, the worktree path and the base commit. Two `VERDICT: pass` lines are required. Any blocker or major goes to the implementer as a fix round (same branch, further commits), then gate and both reviewers run again. At most two fix rounds in total.
   7. Merge: on main, `git merge --squash task/<ID>`, then `python3 scripts/backlog.py set-status <ID> done`, and one commit `<ID>: <title>` whose body says what was built and why, lists the tests, and carries every `assumption:` line from the branch. Run `scripts/gate.sh <ID>` on main (another task may have landed), then `git push origin main`. Remove the worktree (`git worktree remove ../wt-<ID>`) and delete the local branch. Task branches are never pushed; the cloud git proxy refuses remote branch deletes, so nothing you push can be cleaned up.
   8. If the gate, the TDD check or the reviewers still fail after the last round: push the branch as `wip/<ID>`, `set-status <ID> blocked: <one line>` committed on main and pushed, remove the worktree, continue with the next task.
4. End the run with the report below. If a task is blocked or a wave is now complete, also send a notification to the owner with one line.

## Rules you enforce

- The gate decides, not opinion: nothing merges without `gate: ok`, two pass verdicts and the TDD check.
- Reviewers report, implementers fix. You never edit source code yourself; you edit only Status lines through the script.
- One commit per task on main, message without any AI model, tool or session reference, no em or en dashes.
- No new crates beyond TECHSPEC section 13; a task that seems to need one is blocked with that reason.
- Spikes and manual tasks are never started by you.
- The sandbox may hold a stale clone: step 1's pull is mandatory, and a push rejected as non-fast-forward means pull again and retry, never force.
- If the sandbox's permission classifier denies a command, split it into plain separate commands and retry; never route around a denial.

## Report (last message of the run)

```
run: <date> main <hash before> -> <hash after>
done: <ID> <title> (<n> tests) ...
blocked: <ID> <reason> ...
human: <ready human task IDs>
next: <IDs that became ready>
cost: <tasks tried> tasks, <fix rounds> fix rounds
```
