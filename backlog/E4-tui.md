# E4 TUI

Milestone: M3. E4 delivers the ratatui board that `brain-swap` opens without a subcommand: the start-up gate of that bare invocation (unknown `--board` exit 3, no terminal exit 2), the Elm-style state machine (`App`, `Mode`, `Input`, `Outcome`, `Effect`, a pure `update`), the runtime that draws, polls keys every 250 ms, fingerprints the board folder every second and runs effects through `core::store` and the editor, the board view with its columns, cards, ages and preview panel, the detail view over a card's timeline, the template and board pickers with the title input, and the external editor with the 7.6 merge. It sits in M3 because principle 1 ships the cue (files, CLI, pack) before the board (TECHSPEC 15); E4-F1 may start as soon as E1-F5 is done, E4-F3 right after E4-F1, E4-F2 and E4-F4 once E1-F6 is also done, and E4-F5 after E2-F3 (TECHSPEC 15.1). The jump from the board and the detail view (Enter, FR-25, FR-28, the 7.7 messages) is E5-F3, which builds on the types and seams this epic provides. Spec: TECHSPEC 5.2 and 5.3 (as consumed by the TUI), 7 (7.1 to 7.6), 10 (lock never held across the editor), 11 (status line, warnings in Help, reload log), 12.1 (TUI bullet, `edit:after_tmp` crash, performance), 12.2, the E4 entries of 15; PRD 7.5 to 7.7 (FR-19 to FR-32 except FR-25 and FR-28), FR-44, FR-45, NFR-02, NFR-10 and the TUI parts of AS-6, AS-7, AS-8 and AS-9.

## Features

| ID | Title | Depends on | Tasks | Size |
|---|---|---|---|---|
| E4-F1 | Shell | E1-F3, E1-F5 | 10 | L |
| E4-F2 | Board view | E4-F1, E1-F6 | 6 | L |
| E4-F3 | Detail view | E4-F1 | 4 | M |
| E4-F4 | Pickers | E4-F1, E1-F6 | 4 | M |
| E4-F5 | Editor | E4-F1, E1-F6, E2-F3 | 7 | L |

## E4-F1 Shell

- Depends on: E1-F3, E1-F5
- Covers: FR-19, FR-23, FR-32, FR-44, FR-45, NFR-02, NFR-10
- Spec: TECHSPEC 5.2 (actions per mode), 5.3 (matching as consumed), 6.2 (the TUI with an unknown `--board` exits 3 before the alternate screen; exit 2 without a terminal), 6.3 (`brain-swap` opens the TUI; exit 2 without a terminal), 7.1, 7.2 (status line, Help, Message, ConfigError), 7.4, 7.5, 10 (config), 11 (status line, warnings, reload log), 12.1 (TUI, performance); PRD 7.5 (FR-19, FR-23), 7.7 (FR-32), 7.9 (FR-44, FR-45), A-23, A-27, AS-9 (notice)
- Scope: The skeleton every other E4 feature plugs into. `src/tui/app.rs` holds the 7.1 types and the pure `update`, with the initial selection of FR-19 and A-27, the crossterm key conversion, the mode-to-context mapping of 5.2, Board `help` and `quit`, the status line for `Failed` outcomes and start-up warnings, and the Help, Message and ConfigError screens. `src/tui/runtime.rs` runs the loop with an injectable terminal guard, event source and clock (so tests need no tty), restores the terminal on exit and on panic, fingerprints the board folder every second and turns a change into `Reloaded` that keeps the selection by card ID. `tui::gate` checks the bare invocation (config load with the first-run notice, unknown `--board` exit 3, no terminal exit 2), and `tui::prepare` and `tui::run` turn its `tui::TuiStart` into a running TUI: ConfigError for a broken config, the board named by `--board` or `default_board`, key binding warnings in the status line and in Help. Keyboard accessibility and the NFR-02 timings are pinned by tests.
- Provides: `tui::app::App { mode: Mode, board: Board, sel: Selection, keys: KeyMap, status: Option<String>, now: Stamp, warnings: Vec<String> }` with `App::new(board: Board, keys: KeyMap, now: Stamp) -> App`, `App::config_error(text: String, now: Stamp) -> App`, `App::selected_card(&self) -> Option<&Card>`; `tui::app::Selection { column: usize, card: Option<CardId> }` (column indexes `Board::column_list()`); `tui::app::initial_selection(board: &Board) -> Selection`; `tui::app::Mode` (`Board`, `Detail { card: CardId, note: usize, scroll: u16 }`, `TemplatePick { sel: usize }`, `TitleInput { template: String, buf: String }`, `BoardPick { sel: usize }`, `Message { text: String, back: Box<Mode> }`, `Help { back: Box<Mode> }`, `ConfigError(String)`); `tui::app::Input` (`Key(KeyEvent)`, `Tick(Stamp)`, `Reloaded(Board)`, `Done(Outcome)`); `tui::app::Outcome` (`Moved(CardId)`, `Created(CardId)`, `Edited(CardId)`, `Failed(String)`; E5-F3 adds `Jumped`, `NotJumped`, `FocusFailed`, A-E4-03); `tui::app::Effect` (`Move(CardId, String)`, `Create { template: String, title: String, column: String }`, `Edit(CardId)`, `LoadBoard(String)`, `Quit`; E5-F3 adds `Jump(CardId, usize)`); `tui::app::update(app: &mut App, input: Input) -> Vec<Effect>`; `tui::app::key_press(event: &KeyEvent) -> Option<KeyPress>`; `tui::app::context(mode: &Mode) -> Option<KeyContext>`; `tui::view_frame::draw(frame: &mut Frame, app: &App)` (status line on the last row, the mode's screen above it); `tui::view_help::draw`, `tui::view_message::draw` (Message and ConfigError); `tui::runtime::TerminalGuard` (`enter`, `leave`), `tui::runtime::CrosstermGuard`, `tui::runtime::Events` (`poll(&mut self, timeout: Duration) -> io::Result<Option<KeyEvent>>`), `tui::runtime::CrosstermEvents`, `tui::runtime::Clock` (`now(&self) -> Stamp`), `tui::runtime::SystemClock`, `tui::runtime::Parts { guard, events, clock }` (E4-F5 adds `launcher`, E5-F3-T3 adds `runner: Box<dyn Runner>`; either may land first), `tui::runtime::Fingerprint`, `tui::runtime::fingerprint(dir: &Path, letter: char) -> Fingerprint`, `tui::runtime::Runtime<B: Backend>` with `Runtime::new(env: Env, prepared: Prepared, terminal: Terminal<B>, parts: Parts) -> Runtime<B>`, `Runtime::app(&self) -> &App`, `Runtime::draw(&mut self) -> io::Result<()>`, `Runtime::dispatch(&mut self, input: Input) -> Option<i32>` (the exit code once quitting), `Runtime::tick(&mut self) -> Option<i32>`, `Runtime::run(self) -> i32`, `Runtime::scan_count(&self) -> u64`; `tui::Prepared { app: App, config: Option<Config> }` (E4-F4 adds `templates`); `tui::TuiStart { config: Result<Loaded, Error>, board: Option<String> }` (the first-run notice and the loader warnings come from `Loaded`); `tui::gate(env: &Env, board: Option<&str>, terminal: bool) -> Result<TuiStart, Error>`; `tui::prepare(env: &mut Env, start: TuiStart) -> Result<Prepared>`; `tui::run(env: Env, board: Option<String>, terminal: bool) -> i32` (the function `main.rs` calls for a bare invocation, E2-F1-T1); test helpers `tests/common/tui.rs` (`NOW`, `TuiFixture` with `new`, `without_config`, `env`, `board_dir`, `home`, `scenario`, `board`, `app`, `runtime`; `write_card`, `key`, `press`, `render`, `rows`, `assert_no_colour`, `FakeGuard`, `FakeEvents`, `FakeClock`) and `tests/common/big_board.rs` (`write_big_board(dir: &Path, cards: usize, notes: usize)`)
- Requires: E1-F1 (`core::env::Env`, `core::error::Error` with `exit_code()` and `Display`, `core::time::Stamp`, `core::log::event`, `core::log::guard_board_folders`, `tests/common/env.rs` `core_env`, `tests/common/spawn.rs` `Scenario`), E1-F3 (`core::config::Config`, `core::config::load`, `core::config::Loaded`, `Config::board`, `core::keys::Key`, `Binding`, `KeyPress`, `parse_binding`, `Action` with `config_name()`, `KeyContext`, `KeyMap` with `defaults()`, `build()`, `action()`, `keys()`), E1-F5 (`core::board::load`, `core::model::Board`, `Board::column_list`, `Board::cards_in`, `Board::open_cards`, `Board::card`, `Card::activity`, fixtures `tests/fixtures/boards/work_4_9/` and `tests/fixtures/boards/as7/`)
- Feature DoD:
  - [ ] A snapshot test of the initial selection: the newest open card, and the first column with none (E4-F1-T1).
  - [ ] A file change shows after one `Tick` (E4-F1-T7).
  - [ ] A `Failed` outcome sets the status line and a key press clears it (E4-F1-T3).
  - [ ] Help lists every action with its current keys, then all warnings (E4-F1-T4).
  - [ ] In ConfigError every key emits only `Quit` (E4-F1-T5).
  - [ ] The terminal is restored on quit, on an error and on panic, and ConfigError exits 1 (E4-F1-T6).
  - [ ] An invalid or conflicting binding warns in the status line and keeps its default; a broken config starts in ConfigError (E4-F1-T8).
  - [ ] A bare invocation with an unknown `--board` exits 3 and one without a terminal exits 2, both before the alternate screen (E4-F1-T8).
  - [ ] Every action is reachable by default without a modifier beyond Shift, scrolling aside, and no shell screen uses colour (E4-F1-T9).
  - [ ] An idle runtime scans the board folder once per second (E4-F1-T7).
  - [ ] First frame and key reaction meet NFR-02 in the `#[ignore]` release test (E4-F1-T10).

### E4-F1-T1 App state and initial selection

- Status: todo
- Depends on: E1-F5-T3, E1-F3-T5, E1-F3-T3, E1-F1-T6
- Covers: FR-19; TECHSPEC 7.1 (`App`, `Mode`, `Input`, `Outcome`, `Effect`, `update`, start selection), 3 (I5); PRD A-27
- Size: M
- Scope: `src/tui/app.rs` with the 7.1 types as listed in Provides: `App` with the 7.1 fields plus `warnings` (A-E4-02), `Selection`, `Mode` (all eight variants), `Input` (all four), `Outcome` and `Effect` without the jump variants (A-E4-03), and `update(app, input) -> Vec<Effect>`, which in this task handles no input and returns no effect. `initial_selection(board)`: among `Board::open_cards()` the card with the newest `Card::activity()`, ties to the higher number, with the index of its column in `Board::column_list()` (an extra column counts, I5); with no open card, column 0 and its first card if it has one. `App::new(board, keys, now)` sets `mode: Board`, `status: None`, empty `warnings` and the initial selection; `App::selected_card()` looks `sel.card` up in the board. Test helper `tests/common/tui.rs`: `NOW` (`2026-10-08T11:52:00+03:00`), `TuiFixture::new(fixture: Option<&str>)` (a temp dir with `home/`, the named `tests/fixtures/boards/<fixture>/` copied to `<home>/.local/share/brain-swap/work`, `env = core_env(&home, NOW)`, the default config written once through `core::config::load`), `TuiFixture::without_config(fixture: Option<&str>)` (the same, but no config file is written, for first-run tests), `TuiFixture::board()` (loads `work`), `TuiFixture::app()` (`App::new` with `KeyMap::defaults()` and `env.now`), and `write_card(dir, id, column, created, notes: &[(&str, &str)])` writing a canonical card titled `card <id>` whose notes are `(stamp, next)` pairs with Doing `doing <id>`, Watch out `nothing` and place `cwd=/w`.
- Not in scope: key handling (E4-F1-T2); drawing (E4-F1-T3, E4-F2); keeping the selection on reload (E4-F1-T7); the selection after a board switch (E4-F4-T4).
- Tests first:
  1. `fr19_selects_open_card_with_newest_activity`: given cards written with `write_card` on 2026-10-08 at +03:00 (W-1 in Todo created 09:00 without notes, W-2 in Doing with a note at 11:00, W-3 in Todo with a note at 10:00, W-4 in Done with a note at 11:30), when `App::new` runs, then `sel` is column 1 with card W-2 and `mode` is `Board`.
  2. `fr19_activity_tie_selects_higher_number`: given W-5 in Todo and W-7 in Doing with notes stamped `2026-10-08T10:00:00+03:00`, when `App::new` runs, then `sel` is column 1 with card W-7.
  3. `fr19_no_open_card_focuses_first_column`: given W-1 and W-2 both in Done, when `App::new` runs, then `sel` is column 0 with no card.
  4. `fr19_empty_board_focuses_first_column`: given `TuiFixture::new(None)` whose board folder does not exist, when the board is loaded and `App::new` runs, then `sel` is column 0 with no card.
  5. `i5_card_in_extra_column_can_be_selected_first`: given W-1 in Todo created 09:00 and W-2 with `column: Doign` and a note at 11:00, when `App::new` runs, then `sel` is column 3 (`Doign (unknown)`) with card W-2.
  6. `fr19_initial_selection_snapshot`: given the boards of tests 1 and 3, when `initial_selection` runs on each, then `insta::assert_debug_snapshot!` pins both `Selection` values (TUI state snapshot).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/tui/app.rs` uses nothing from `std::fs`, `std::io` or `std::process` (grep), so `update` stays pure.

### E4-F1-T2 Key events, mode contexts and quit

- Status: todo
- Depends on: E4-F1-T1, E1-F3-T6
- Covers: TECHSPEC 5.2 (actions per mode, Board ignores `scroll_*`), 5.3 (SHIFT ignored for printable characters, ctrl and alt exact), 7.1 (Board `quit` emits `Quit`)
- Size: S
- Scope: `key_press(&KeyEvent) -> Option<KeyPress>`: release events give `None` (A-E4-26); `KeyCode::Char(' ')` is `Key::Space`, any other `Char(c)` is `Key::Char(c)` as reported; `Enter`, `Esc`, `Tab`, `BackTab`, `Backspace`, `Delete`, the arrows, `Home`, `End`, `PageUp`, `PageDown` and `F(1)` to `F(12)` map to their named keys; every other code gives `None`; CONTROL, ALT and SHIFT become `ctrl`, `alt`, `shift`. `context(&Mode)`: Board and Detail give their contexts, TemplatePick and BoardPick give `Picker`, TitleInput gives `TitleInput`, Help, Message and ConfigError give `None` (they take any key). `update` on `Input::Key` converts the event, looks the action up with `KeyMap::action(context, &press)` and, in Board, answers `quit` with `[Quit]`; every other action stays without effect until the task that owns it. Test helpers `key(spec: &str) -> KeyEvent` (5.3 syntax through `parse_binding`; an upper-case `Char` gets SHIFT as crossterm reports it) and `press(&mut App, &[&str]) -> Vec<Effect>` (all effects of the presses, in order).
- Not in scope: navigation, moves, open, new, edit, switch board (E4-F2 to E4-F5); `jump` (E5-F3); the status line (E4-F1-T3).
- Tests first:
  1. `ts7_1_quit_key_emits_quit`: given the `work_4_9` app, when `q` is pressed, then the effects are `[Quit]`, and pressing `esc` instead gives `[Quit]` too.
  2. `ts5_3_char_event_maps_to_char_key`: given `KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)`, when `key_press` runs, then it returns `KeyPress { key: Char('j'), ctrl: false, alt: false, shift: false }`.
  3. `ts5_3_upper_case_matches_with_and_without_shift`: given `Char('H')` with SHIFT and `Char('H')` without, when each is converted and looked up in Board with the defaults, then both give `MoveLeft`.
  4. `ts5_3_named_and_function_keys_map`: given events for Enter, Esc, Tab, BackTab, `Char(' ')`, Backspace, Delete, Up, Down, Left, Right, Home, End, PageUp, PageDown, `F(1)` and `F(12)`, when converted, then each gives its named `Key` (`Space` for the blank), and `F(13)` and `Insert` give `None`.
  5. `ts5_3_ctrl_and_alt_carry_over`: given `Char('d')` with CONTROL and `Char('x')` with ALT, when converted, then the first has `ctrl: true` and the second `alt: true`, and in Board the first gives no action.
  6. `ts7_1_key_release_is_ignored`: given a `q` event with `KeyEventKind::Release`, when `update` runs, then there is no effect.
  7. `ts5_2_rebound_quit_key`: given an app whose keys are `KeyMap::build(&[("quit", ["x"])])`, when `x` is pressed, then the effects are `[Quit]`, and `q` gives none.
  8. `ts5_2_board_ignores_scroll_keys`: given the defaults, when `ctrl+d` and `ctrl+u` are pressed in Board, then there is no effect and the mode stays `Board`.
  9. `ts5_2_mode_maps_to_key_context`: given one value of each `Mode` variant, when `context` runs, then Board gives `Board`, Detail `Detail`, TemplatePick and BoardPick `Picker`, TitleInput `TitleInput`, and Help, Message and ConfigError `None`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `KeyCode` appears in `src/tui/app.rs` only (grep), so `key_press` is the one conversion point.

### E4-F1-T3 Frame, status line and failed outcomes

- Status: todo
- Depends on: E4-F1-T2
- Covers: NFR-10; TECHSPEC 7.1 (`Failed` sets the status line), 7.2 (status line), 7.3 (nothing relies on colour), 11 (action errors in the status line)
- Size: S
- Scope: `src/tui/view_frame.rs` `draw(frame, app)`: the last row is the status line, ` <status>` or blank; the rows above go to the mode's screen (`view_board`, `view_detail`, `view_pick`, `view_help`, `view_message` as later tasks add them; an empty area until then). `update`: `Done(Failed(text))` sets `status` and changes nothing else; every key input clears `status` before it is handled (A-E4-08). Drawing uses only the reverse and bold modifiers, never a colour. Test helpers `render(&App, width, height) -> Buffer` (a `TestBackend` frame), `rows(&Buffer) -> Vec<String>` (trailing blanks trimmed) and `assert_no_colour(&Buffer)` (every cell has fg and bg `Color::Reset`).
- Not in scope: the screens themselves (E4-F1-T4, E4-F1-T5, E4-F2 to E4-F4); start-up texts in the status line (E4-F1-T8).
- Tests first:
  1. `ts7_2_status_line_is_last_row`: given an app with `status` `hello`, when rendered at 80x24, then row 23 is ` hello`.
  2. `ts7_1_failed_outcome_sets_status_line`: given the `work_4_9` app, when `update` gets `Done(Failed("board busy, try again"))`, then `status` is that text, mode and `sel` are unchanged and there is no effect.
  3. `ts7_1_failed_texts_are_shown_verbatim`: given `Done(Failed)` with `W-12.md: verify failed` and, separately, `editor: exit status: 1`, when rendered at 80x24 and at 120x40, then the last row reads exactly ` <text>`.
  4. `ts7_1_key_press_clears_status`: given `status` set, when the unbound key `z` is pressed, then `status` is `None` and there is no effect.
  5. `nfr10_status_line_uses_no_colour`: given the buffer of test 1, when `assert_no_colour` runs, then it passes.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/tui/` names no `Color::` value other than `Color::Reset` (grep).

### E4-F1-T4 Help and Message screens

- Status: todo
- Depends on: E4-F1-T3
- Covers: FR-32; TECHSPEC 7.1 (Board `help` opens Help; any key in Help or Message returns), 7.2 (Help: every action with its keys, then all warnings; Message), 11 (all warnings in Help), 12.1 (snapshots at 80x24 and 120x40)
- Size: M
- Scope: Board `help` sets `Help { back: Box::new(Board) }`; in Help and in Message any key, `q` and `esc` included, restores `back` with no effect. `src/tui/view_help.rs`: a row ` help`, then one row per `Action` in 5.2 order, ` <config_name padded to 14><bindings in 5.3 syntax joined by ", ">` taken from `KeyMap::keys`, then, when there is any warning, a blank row, ` warnings:` and one row per warning, `app.warnings` first, then `app.board.warnings`; rows beyond the screen are cut (A-E4-14). `src/tui/view_message.rs`: Message shows its `text` one row per line (A-E4-15).
- Not in scope: Help from Detail (E4-F3-T4); the texts of jump messages (E5-F3); start-up warnings (E4-F1-T8).
- Tests first:
  1. `fr32_help_key_opens_help`: given the `work_4_9` app, when `?` is pressed, then the mode is `Help { back: Board }` and there is no effect.
  2. `ts7_1_any_key_in_help_returns`: given Help over Board, when `x`, `q` or `esc` is pressed (each from a fresh Help), then the mode is `Board` and there is no effect.
  3. `fr32_help_lists_every_action_with_default_keys`: given the defaults, when Help is rendered at 80x24, then rows 1 to 17 name the 17 actions in 5.2 order with their keys, among them `up` with `k, up`, `scroll_down` with `ctrl+d` and `quit` with `q, esc`.
  4. `fr32_help_shows_current_keys_after_rebinding`: given keys built from `open = "x"`, when Help is rendered, then the `open` row shows `x` and no `o`.
  5. `fr32_help_lists_all_warnings`: given `app.warnings` `["config: unknown key 'colour'"]` and a board warning `W-13: unknown column 'Doign'`, when Help is rendered at 120x40, then the rows after ` warnings:` are those two texts in that order.
  6. `ts7_1_any_key_in_message_returns`: given `Message { text: "pane closed\n/w", back: Board }`, when `q` is pressed, then the mode is `Board` and the effects are empty (no `Quit`).
  7. `ts7_2_message_shows_each_line`: given that Message, when rendered at 80x24, then two rows read ` pane closed` and ` /w`.
  8. `ts7_2_help_and_message_snapshots`: given Help with the defaults and one warning, and the Message of test 6, when rendered at 80x24 and 120x40, then insta snapshots pin all four and `assert_no_colour` passes on each.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Snapshots reviewed with `cargo insta review` and committed (12.2 item 3).

### E4-F1-T5 ConfigError screen

- Status: todo
- Depends on: E4-F1-T3
- Covers: FR-45; TECHSPEC 7.1 (ConfigError, any key exits), 7.2 (ConfigError), 10 (config: never defaults that could write into wrong folders); PRD A-23
- Size: S
- Scope: `App::config_error(text, now)` builds an app in `ConfigError(text)` over an empty board (empty name and path, no cards, default meta) with the default keys. In ConfigError every key gives exactly `[Quit]`, so no `Move`, `Create`, `Edit` or `LoadBoard` can come from it; `Tick` and `Reloaded` change nothing. `view_message.rs` shows the text wrapped to the width (A-E4-15). The exit code 1 is the runtime's (E4-F1-T6).
- Not in scope: building the text from a config error (E4-F1-T8); the exit code (E4-F1-T6).
- Tests first:
  1. `fr45_config_error_any_key_emits_only_quit`: given `App::config_error("config /h/.config/brain-swap/config.toml:4:7: expected '='", NOW)`, when each of `n`, `L`, `e`, `enter`, `b` and `q` is pressed, then each returns exactly `[Quit]`.
  2. `fr45_config_error_screen_shows_error`: given that app, when rendered at 80x24, then the rows above the status line hold the whole text, wrapped.
  3. `fr45_config_error_ignores_reload_and_tick`: given that app, when `update` gets `Reloaded(<the work_4_9 board>)` and `Tick(NOW + 1 min)`, then the mode is still `ConfigError` and the board is still empty.
  4. `fr45_config_error_snapshots`: given that app, when rendered at 80x24 and 120x40, then insta snapshots pin both and `assert_no_colour` passes.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The empty board of `App::config_error` has an empty `path`, checked in test 3, so no effect can name a folder.

### E4-F1-T6 Runtime loop and terminal guard

- Status: todo
- Depends on: E4-F1-T5
- Covers: FR-45; TECHSPEC 7.1 (`runtime.rs` draws, polls input every 250 ms, runs effects, returns results as `Input::Done`), 7.5, 12.1 (injectable terminal guard, no tty)
- Size: M
- Scope: `src/tui/runtime.rs`: traits `TerminalGuard`, `Events` and `Clock`; production `CrosstermGuard` (raw mode and the alternate screen through `ratatui::crossterm`), `CrosstermEvents` (key press events; resize and other events give `None` and the next loop redraws), `SystemClock` (the system time in `env.tz`); `Parts`; `tui::Prepared { app, config }`; `Runtime::new`, `app`, `draw`, `dispatch` (runs `update`, then each effect in order, an effect whose arm a later task adds being ignored until then; `Quit` gives `Some(1)` when the mode is ConfigError, else `Some(0)`, A-E4-04) and `run`: `guard.enter()`, then loop: draw, `events.poll(250 ms)`, dispatch a key, until an exit code; a guard scope calls `guard.leave()` on every way out, including unwinding from a panic; an error from `poll` or `draw` ends with exit 1 after `leave` (A-E4-04). Test fakes in `tests/common/tui.rs`: `FakeGuard` (a shared log of `enter` and `leave`), `FakeEvents` (a queue of keys, timeouts, an error or a panic; records each requested timeout; a timeout advances the shared `FakeClock` by that timeout), `FakeClock`, and `TuiFixture::runtime(width, height)` building a `Prepared` from `TuiFixture::app()` and the loaded config with these fakes.
- Not in scope: the tick, fingerprint and reload (E4-F1-T7); `tui::run` with the real terminal and the panic hook (E4-F1-T8); the move, create, edit and load effects (E4-F2-T5, E4-F4-T3, E4-F5-T1, E4-F4-T4).
- Tests first:
  1. `ts7_5_terminal_entered_and_restored_on_quit`: given a runtime with `FakeEvents` `[q]`, when `run` runs, then it returns 0 and the guard log is `[enter, leave]`.
  2. `fr45_config_error_any_key_exits_1`: given a runtime over `App::config_error(..)` with `FakeEvents` `[x]`, when `run` runs, then it returns 1 and the guard log is `[enter, leave]`.
  3. `ts7_5_terminal_restored_on_panic`: given `FakeEvents` `[timeout, panic]`, when `run` runs inside `catch_unwind`, then the result is a panic and the guard log ends with `leave`.
  4. `ts7_5_terminal_restored_on_event_error`: given `FakeEvents` `[error]`, when `run` runs, then it returns 1 and the guard log is `[enter, leave]`.
  5. `ts7_1_input_polled_every_250_ms`: given `FakeEvents` `[timeout, timeout, q]`, when `run` runs, then the three recorded timeouts are each 250 ms.
  6. `ts7_1_screen_redrawn_after_input`: given a runtime at 80x24, when `dispatch(Done(Failed("x")))` and `draw` run, then the backend's last row is ` x`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `CrosstermGuard` is the only code in `src/` that enables raw mode or enters the alternate screen (grep).

### E4-F1-T7 Polling and reload

- Status: todo
- Depends on: E4-F1-T6, E1-F1-T8
- Covers: FR-23, NFR-02; TECHSPEC 7.1 (`Tick`, `Reloaded` keeps the selection), 7.4, 11 (reloads in the log), T-06, T-17
- Size: M
- Scope: `Fingerprint`: the sorted `(name, mtime, length)` of `board.md` and of the files in the board root matching `<letter>-<n>.md` for the board's letter (A-E4-23); a missing folder gives an empty fingerprint. `Runtime::new` takes the baseline fingerprint of the board folder; it is not counted by `scan_count`, and only `tick` increments it. `Runtime::tick()`: one fingerprint (counted by `scan_count`), compared with the previous one; on a change `core::board::load`, `log::event(env, "reload", <board name>)` and `dispatch(Reloaded(board))`; a load error keeps the board and sets the status line to the error (A-E4-18); then `dispatch(Tick(clock.now()))`. `run` calls `tick` whenever a second has passed on the clock since the last scan. `update`: `Tick(now)` sets `app.now`; `Reloaded(board)` with the current board's name replaces the board and keeps the selection by card ID, the focus following the card to its column; a vanished card gives the card at its old row in the same column, clamped to the last, or none when the column is empty; a column index past the new column list is clamped (A-E4-07). A `Reloaded` with another board name is E4-F4-T4.
- Not in scope: reload while in Detail (E4-F3-T4); ages redrawn after a tick (E4-F2-T3); the board switch (E4-F4-T4).
- Tests first:
  1. `ts7_1_tick_updates_now`: given the `work_4_9` app, when `update` gets `Tick(NOW + 5 min)`, then `app.now` equals that stamp and there is no effect.
  2. `fr23_file_change_shows_after_one_tick`: given a runtime on a `work_4_9` copy, when a note block is appended to `W-11.md` by hand and `tick` runs once, then `app().board` shows W-11 with one note.
  3. `fr23_reload_keeps_selection_by_card_id`: given W-12 selected, when W-12's title line is edited by hand, `write_card` adds W-13 in Doing with a note at 11:50, and `tick` runs, then W-12 is still selected in column 1 with the new title.
  4. `fr23_card_moved_by_hand_stays_selected`: given W-12 selected, when its `column:` line is set to `Todo` by hand and `tick` runs, then `sel` is column 0 with card W-12.
  5. `ts7_4_vanished_card_selects_nearest_in_column`: given Todo holding W-3, W-2, W-1 (notes at 10:20, 10:10, 10:00) with W-1 (row 2) selected, when `W-1.md` is deleted and `tick` runs, then W-2 (row 1) is selected, and when `W-2.md` and `W-3.md` are deleted and `tick` runs, then `sel` is column 0 with no card.
  6. `ts7_4_unchanged_folder_does_not_reload`: given `env.log` set, when `tick` runs twice without a change, then `scan_count()` is 2 and the log holds no `reload` line.
  7. `ts7_4_other_files_do_not_reload`: given `env.log` set, when `README.md` and `.W-12.md.bs-tmp-1` are written and `tick` runs, then the log holds no `reload` line.
  8. `ts7_4_board_md_change_reloads`: given `board.md` rewritten with `columns: Todo, Doing, Review, Done`, when `tick` runs, then `column_list()` of the app's board names `Review` third.
  9. `ts7_4_same_length_change_with_new_mtime_reloads`: given `W-12.md` overwritten with bytes of equal length and its mtime set one second later, when `tick` runs, then the board is reloaded (a `reload` line).
  10. `ts11_reload_is_logged_once_per_change`: given `env.log` set, when one card file changes and `tick` runs twice, then the log holds exactly one line with kind `reload` and text `work`.
  11. `nfr02_idle_runtime_scans_once_per_second`: given `FakeEvents` of eight 250 ms timeouts and then `q`, when `run` runs, then `scan_count()` is 2 and no reload happened.
  12. `ts7_4_missing_folder_created_later_reloads`: given `TuiFixture::new(None)` (no board folder), when the folder is created with `W-1.md` and `tick` runs, then the app's board holds W-1.
  13. `ts7_4_reload_error_keeps_board`: given the board folder made unreadable (mode 000; skipped as root), when `tick` runs, then the board is unchanged and `status` starts with `board work: `.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `fingerprint` uses only `read_dir` and file metadata and never opens a card file (review).

### E4-F1-T8 Start-up: gate, prepare and tui::run

- Status: todo
- Depends on: E4-F1-T4, E4-F1-T6, E1-F3-T3, E1-F1-T8
- Covers: FR-19, FR-44, FR-45; TECHSPEC 6.2 (the TUI with an unknown `--board` exits 3 before the alternate screen; exit 2 without a terminal), 6.3 (`brain-swap` opens the TUI; exit 2 without a terminal), 7.1 (start), 7.5 (on panic), 10 (config), 11 (loader warnings and the first load warning in the status line, all warnings in Help, `BRAIN_SWAP_LOG` inside a board folder); AS-9 (notice)
- Size: M
- Scope: `tui::gate(env, board, terminal)` checks the invocation without a subcommand, in this order (A-E4-27): `core::config::load(env)`, whose error is not raised but carried in `TuiStart.config` for ConfigError (a missing config is the first run: the loader writes it and returns `created: Some(path)`); when the config loaded, `board` resolved through `Config::board` (unknown: `Error::NotFound` with the 6.2 text, exit 3) and carried as its configured name; then `terminal` must be true, else `Error::Usage("not a terminal")`, exit 2. `tui::prepare(env, start)`: a `config` error gives `App::config_error(<its Display>)` with `config: None`, reading no board; otherwise `log::guard_board_folders(env, &config.boards)` (its warning kept, A-E4-25), `KeyMap::build(&config.keys)`, the board `start.board` or `config.default_board` through `Config::board` and `core::board::load` (an error is returned, A-E4-18), `App::new`, `app.warnings` = `Loaded.warnings`, the guard warning, then the key warnings, and `app.status` = `created config <path>` when `Loaded.created` is `Some(path)`, else the first of `app.warnings` followed by `board.warnings` (A-E4-08). `tui::run(env, board, terminal)`: `gate`, then `prepare` (an error from either prints `brain-swap: <message>` on stderr and returns its exit code, before the alternate screen and with nothing on stdout), a panic hook that restores the terminal before the previous hook prints, `CrosstermGuard`, `CrosstermEvents`, `SystemClock` and `Terminal<CrosstermBackend<Stdout>>`, then `Runtime::run`. If E2-F1-T1 has landed, `tui::run` replaces the `todo!()` stub it left in `src/tui/mod.rs`. `TuiFixture::runtime` now builds its `Prepared` through `tui::prepare`. The gate tests are in process (`TuiFixture`), so this task needs no E2 task.
- Not in scope: `main.rs` routing a bare invocation to `tui::run`, parsing `--board` and passing stdin and stdout both being terminals as `terminal` (E2-F1-T1); the config write and its text (E1-F3-T3); template warnings (E4-F4-T1); the AS-9 board rendering (E4-F2-T6).
- Tests first:
  1. `fr19_opens_default_board`: given the fixture's default config and `TuiStart { config: Ok(<the fixture's Loaded>), board: None }`, when `prepare` runs, then `app.board.name` is `work`.
  2. `fr19_opens_board_named_by_flag`: given `H-1.md` written in `<home>/.local/share/brain-swap/home` and `board: Some("home")`, when `prepare` runs, then `app.board.name` is `home` and H-1 is selected.
  3. `fr45_broken_config_starts_in_config_error`: given `TuiFixture::new(None)` and `TuiStart { config: Err(Error::Config { path, line: 4, col: 7, msg: "expected '='" }), board: None }`, when `prepare` runs, then the mode is `ConfigError(<that error's Display>)`, `config` is `None` and no folder exists under `<home>/.local/share/brain-swap/`.
  4. `as9_first_run_notice_in_status_line`: given a `Loaded` whose `created` is `Some(<path>)`, when `prepare` runs, then `app.status` is `created config <path>`.
  5. `fr44_bad_binding_warns_and_keeps_default`: given `[keys]` `help = "foo"` in the config, when `prepare` runs, then `status` is `key config: help: unknown key 'foo'` and pressing `?` opens Help.
  6. `fr44_conflicting_binding_reverts_and_warns`: given `help = "j"`, when `prepare` runs, then `status` names `help` and pressing `?` opens Help.
  7. `fr44_help_lists_all_start_warnings`: given `Loaded.warnings` `["config: unknown key 'colour'"]` and `help = "foo"`, when Help is rendered, then its warning rows are the config warning and then the key warning.
  8. `ts11_log_inside_board_folder_is_dropped_with_warning`: given `env.log` inside the work board folder, when `prepare` runs, then `app.warnings` holds `BRAIN_SWAP_LOG inside board work, ignored` and `env.log` is `None`.
  9. `ts11_first_load_warning_in_status_line`: given the `as7` fixture, no notice and no key warning, when `prepare` runs, then `status` equals the board's first warning, `W-13: unknown column 'Doign'`.
  10. `ts11_notice_wins_over_warnings`: given a `Loaded` with `created: Some(<path>)` and `help = "foo"`, when `prepare` runs, then `status` is `created config <path>` and Help still lists the key warning.
  11. `fr19_unlistable_board_folder_fails_before_screen`: given the work folder with mode 000 (skipped as root), when `prepare` runs, then it returns `Err(Error::Io(_))` with exit code 1.
  12. `ts6_3_gate_passes_with_terminal`: given the fixture's default config, when `tui::gate(&env, Some("HOME"), true)` runs, then it returns a `TuiStart` whose `config` is `Ok` and whose `board` is `Some("home")`; given the config file replaced by one whose line 4 lacks its `=`, when `tui::gate(&env, None, true)` runs, then it returns a `TuiStart` whose `config` is `Err(Error::Config { .. })`.
  13. `ts11_loader_warnings_reach_help`: given a config file holding `colour = "red"`, when `tui::gate(&env, None, true)` and then `prepare` run, then `app.warnings` starts with `config: unknown key 'colour'`.
  14. `ts6_2_tui_unknown_board_exits_3_before_screen`: given the fixture's default config, when `tui::gate(&env, Some("work2"), false)` runs, then it returns `Err(Error::NotFound(_))` with exit code 3 and the text `unknown board 'work2' (work, home, personal)` (the board is checked before the terminal), and `tui::run(env, Some("work2".into()), false)` returns 3.
  15. `ts6_3_bare_without_terminal_exits_2`: given the fixture's default config, when `tui::gate(&env, None, false)` runs, then it returns `Err(Error::Usage(_))` with exit code 2 and the text `not a terminal`, and `tui::run(env, None, false)` returns 2.
  16. `ts6_3_broken_config_without_terminal_exits_2`: given the broken config file of test 12, when `tui::gate(&env, Some("work2"), false)` runs, then it returns an error with exit code 2 (no board is resolved without a config).
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Nothing is written to stdout and no terminal guard is entered before `gate` and `prepare` pass (review of `tui::run`).
  - [ ] Manual, as soon as `main.rs` routes a bare invocation to `tui::run` (E2-F1-T1; if that task lands later, its commit records this run): `brain-swap` in a real terminal enters the alternate screen, `q` leaves the shell prompt intact, and `brain-swap --board home` opens home; recorded in the commit message (12.2, manual checks).

### E4-F1-T9 Keyboard accessibility

- Status: todo
- Depends on: E4-F1-T4, E4-F1-T5
- Covers: NFR-10; TECHSPEC 5.2 (defaults), 7.3 (nothing relies on colour)
- Size: S
- Scope: Tests over the default key map and the shell's screens, plus whatever fix they force. Every later snapshot test calls `assert_no_colour` (E4-F2 to E4-F4); this task pins the defaults and the screens of E4-F1.
- Not in scope: the board selection and focus marker rendering (E4-F2-T1); ways back from Detail and the pickers (E4-F3-T1, E4-F4-T1, E4-F4-T2, E4-F4-T4).
- Tests first:
  1. `nfr10_every_action_has_a_default_key`: given `KeyMap::defaults()`, when `keys(action)` is read for each of the 17 actions, then none is empty.
  2. `nfr10_defaults_need_no_modifier_beyond_shift_except_scrolling`: given the defaults, when every binding is checked, then each has `ctrl` and `alt` false except `scroll_down` (`ctrl+d`) and `scroll_up` (`ctrl+u`).
  3. `nfr10_shell_modes_have_a_default_way_out`: given Help, Message and ConfigError apps and the Board app, when `x`, `x`, `x` and `q` are pressed, then the first two return to Board and the last two emit `Quit`.
  4. `nfr10_help_fits_80x24`: given the defaults and no warning, when Help is rendered at 80x24, then all 17 action rows are present.
  5. `nfr10_shell_screens_use_no_colour`: given Help, Message, ConfigError and a status line, when each is rendered at 80x24, then `assert_no_colour` passes.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The doc comment of `assert_no_colour` states that every TUI snapshot test calls it.

### E4-F1-T10 Performance

- Status: todo
- Depends on: E4-F1-T7, E4-F1-T8
- Covers: NFR-02; TECHSPEC 12.1 (performance, `#[ignore]` release test on 300 cards with 20 notes)
- Size: S
- Scope: `tests/common/big_board.rs` `write_big_board(dir, cards, notes)` writes `board.md` and canonical card files (columns cycling Todo, Doing, Done, template Feature, a `created` stamp, the Feature skeleton, `## Timeline` and `notes` notes a minute apart with three parts and a place line). `tests/tui_perf.rs` holds `#[ignore]` tests run with `cargo test --release -- --ignored` (A-E4-24). E6-F1-T3 reruns them on the finished screens and E6-F1-T4 adds `hyperfine`; E6-F1 writes no second version of these two tests. E6-F1-T2 later changes the `write_big_board` layout in place to A-E6-03 of E6-release.md (W-1 to W-290 Done, W-291 to W-295 Todo, W-296 to W-300 Doing, place lines with pane `w4V:p9`, stamps one hour apart), keeping its signature. Each test prints its measured value under `--nocapture` as `nfr02 first frame: median <x> ms` and `nfr02 key reaction: p95 <x> ms`, which E6-F1-T4 records.
- Not in scope: CLI timings (E6-F1, NFR-01); the jump's added time (E5-F3, E6-F1).
- Tests first:
  1. `nfr02_first_frame_within_150_ms` (performance, `#[ignore]`): given a board of 300 cards with 20 notes each, when `tui::prepare` and the first `draw` on a 120x40 `TestBackend` run, then the median of 5 runs after one warm-up is under 150 ms.
  2. `nfr02_key_reaction_within_50_ms` (performance, `#[ignore]`): given that board in a runtime, when 100 key presses cycling `j`, `l`, `k`, `h` are each dispatched and drawn, then the 95th percentile is under 50 ms.
- DoD:
  - [ ] The tests above pass in a release build; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The measured times are stated in the commit message.
  - [ ] Each test prints its measured value (`nfr02 first frame: median <x> ms`, `nfr02 key reaction: p95 <x> ms`) when run with `--nocapture`.

## E4-F2 Board view

- Depends on: E4-F1, E1-F6
- Covers: FR-20, FR-21, FR-22, FR-29
- Spec: TECHSPEC 4.7 (extra columns, `H` and `L` on them), 4.9 (the drawing), 5.2 (Board navigation and move keys), 7.1 (Board `move_*` emits `Move`, `Moved` selects the card), 7.2 (Board), 7.3, 12.1 (snapshots at 80x24 and 120x40); PRD 7.5 (FR-20 to FR-22), 7.7 (FR-29), NFR-10 (as rendered), AS-7 and AS-9 (TUI rendering)
- Scope: The board screen of 4.9: a header with the board name, its path and the help key; columns side by side with counts and the focused one marked `>` in bold; cards as `<ID> <title>` over the latest note's floored age and a one-line preview of Next (else Doing) or `no note`, the selection in reverse video; the preview panel under the columns, hidden below 20 rows. Navigation stops at the ends, columns scroll vertically to keep the selection visible and narrow boards show a window of columns around the focused one. `H` and `L` move the selected card through `core::store::move_card` and keep it selected, with the 7.3 and 4.7 edge rules. Snapshots pin the 4.9 board at both sizes and the AS-7 and AS-9 boards.
- Provides: `tui::view_board::draw(frame: &mut Frame, area: Rect, app: &App)`; `tui::view_board::card_lines(card: &Card, now: &Stamp) -> (String, String)`; `tui::view_board::panel_lines(card: Option<&Card>, now: &Stamp) -> Vec<String>`; `tui::view_board::age_ago(at: Option<&Stamp>, now: &Stamp) -> String` (`now`, else `<age> ago`); `tui::view_board::visible_columns(count: usize, focused: usize, width: u16) -> Range<usize>`; constants `tui::view_board::MIN_COLUMN_WIDTH` (20) and `tui::view_board::PANEL_MIN_ROWS` (20); `tui::app::move_target(board: &Board, card: &Card, right: bool) -> Option<String>`; the runtime's `Effect::Move` arm; fixture `tests/fixtures/boards/six_columns/`
- Requires: E4-F1 (`App`, `Selection`, `update`, `Runtime::dispatch`, `Runtime::tick`, `tui::prepare`, `view_frame::draw`, `tests/common/tui.rs`), E1-F6 (`core::store::move_card`, `Moved`), E1-F1 (`core::time::age`, `core::time::Age`), E1-F5 (`Board::column_list`, `Board::cards_in`, `core::board::Column`, `Card::latest_note`, fixtures `work_4_9` and `as7`)
- Feature DoD:
  - [ ] Cards render ID, title, age and the Next (else Doing) preview or `no note`, with counts, focus marker and reverse-video selection (E4-F2-T1).
  - [ ] Navigation stops at the ends, columns scroll to keep the selection visible, narrow boards scroll around the focused column (E4-F2-T2).
  - [ ] Every age unit renders floored and ages refresh on `Tick` within 30 seconds (E4-F2-T3).
  - [ ] 4.9 snapshots at both sizes (E4-F2-T4).
  - [ ] No panel at 80x19 (E4-F2-T4).
  - [ ] `H` and `L` follow 7.3 and 4.7 and the moved card stays selected (E4-F2-T5).
  - [ ] The AS-7 hand edits and the AS-9 first-run board render as described (E4-F2-T6).

### E4-F2-T1 Columns and cards

- Status: todo
- Depends on: E4-F1-T3
- Covers: FR-20; TECHSPEC 4.9 (header, columns, cards), 7.2 (Board header), 7.3 (card, header count, focus marker, reverse-video selection, nothing relies on colour)
- Size: M
- Scope: `src/tui/view_board.rs` `draw(frame, area, app)`, called by `view_frame` in Board mode: a header row ` <board name>  <path, with HOME shown as ~>` with `<first help binding> help` at the right edge; a column header row; two rows per card. `card_lines(card, now)`: `<ID> <title>` and `  <age>  <preview>`, the preview being the first line of the latest note's Next, else of its Doing, and nothing when both are empty; `  no note` without notes (A-E4-10); `age_ago` gives `now`, else `<age> ago` (used by the panel). Columns: after a one-character left margin the width is split equally among the columns, each leaving two blank characters at its right; text is cut at the column width with no ellipsis (A-E4-09); a header reads `<name> (<count>)`, the focused one `> <name> (<count>)` in bold; the selected card's two rows are reverse video; extra columns come after the configured ones under their `(unknown)` names. Rows below the cards stay blank until E4-F2-T4.
- Not in scope: navigation and scrolling (E4-F2-T2); age refresh (E4-F2-T3); the preview panel (E4-F2-T4); moves (E4-F2-T5).
- Tests first:
  1. `fr20_card_lines_show_id_title_age_and_next`: given W-12 of `work_4_9` and `NOW`, when `card_lines` runs, then it returns `W-12 migrate invoices to v13` and `  39 min  open the MR, ask the DBA for the v13 deploy list`.
  2. `fr20_card_without_notes_shows_no_note`: given W-11 of `work_4_9`, when `card_lines` runs, then the second line is `  no note`.
  3. `fr20_preview_falls_back_to_doing`: given a card whose latest note has an empty Next and Doing `first` with the continuation `second`, when `card_lines` runs, then the second line is `  <age>  first`.
  4. `fr20_preview_is_first_line_of_next`: given a latest note whose Next is `a` with the continuation `b`, when `card_lines` runs, then the second line ends with `  a`.
  5. `fr20_empty_next_and_doing_show_only_age`: given a latest note with only Watch out set, 39 minutes old, when `card_lines` runs, then the second line is `  39 min`.
  6. `ts7_2_header_shows_board_path_and_help_key`: given the `work_4_9` app, when rendered at 80x24, then row 0 starts ` work  ~/.local/share/brain-swap/work` and ends with `? help`.
  7. `ts7_3_column_headers_show_counts_and_focus`: given W-12 selected, when rendered at 80x24, then row 1 holds `Todo (1)`, `> Doing (1)` and `Done (1)` in that order.
  8. `ts7_3_selection_is_reverse_video_and_focus_bold`: given W-12 selected, when rendered, then every cell of W-12's two rows inside its column has `Modifier::REVERSED`, no cell of another card has it, and the cells of `> Doing (1)` have `Modifier::BOLD`.
  9. `ts7_3_text_is_cut_at_column_width`: given the `work_4_9` app at 80x24, when rendered, then the Todo column's first card row is `W-11 ` followed by a prefix of `invoice PDF shows the wrong VAT` ending before the next column, and no row is longer than 80 characters.
  10. `ts4_7_extra_column_renders_last`: given the `as7` app, when rendered at 120x40, then the last column header is `Doign (unknown) (1)`.
  11. `nfr10_board_uses_no_colour`: given the buffer of test 7, when `assert_no_colour` runs, then it passes.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `card_lines` takes no width (cutting is the renderer's), so the panel, the snapshots and E5-F3 share one text source.

### E4-F2-T2 Navigation and scrolling

- Status: todo
- Depends on: E4-F2-T1
- Covers: TECHSPEC 5.2 (Board `up`, `down`, `left`, `right`), 7.3 (stops at the ends without wrapping, vertical scroll, narrow columns scroll around the focused one)
- Size: M
- Scope: `update` in Board: `down` and `up` move the selection within the focused column in `cards_in` order and stop at the ends; `left` and `right` move the focus one column (extra columns included) and select the card at the same row, clamped to the column's last card, or none in an empty column (A-E4-06); nothing wraps. `visible_columns(count, focused, width)`: every column when `width - 1 >= count * MIN_COLUMN_WIDTH`, else as many as fit, centred on the focused column and clamped to both ends (A-E4-09). Vertical scroll: the focused column's first shown card is `row - (shown - 1)` when its selected row lies below the shown rows, else 0; other columns show from their top (A-E4-06). Fixture `tests/fixtures/boards/six_columns/`: `board.md` with `letter: S` and `columns: A, B, C, D, E, F`, and `S-1.md` to `S-6.md`, one card per column.
- Not in scope: moving cards (E4-F2-T5); the panel's height (E4-F2-T4).
- Tests first:
  1. `ts7_3_down_moves_to_next_card_in_column`: given Todo holding W-3, W-2, W-1 (notes at 10:20, 10:10, 10:00) with W-3 selected, when `j` is pressed, then W-2 is selected and there is no effect.
  2. `ts7_3_up_and_down_stop_at_ends`: given W-3 selected, when `k` is pressed, then W-3 stays; given W-1 selected, when `j` is pressed, then W-1 stays.
  3. `ts7_3_right_and_left_move_focus_one_column`: given the `work_4_9` app with W-11 selected, when `l` is pressed, then column 1 is focused with W-12, and `h` brings back column 0 with W-11.
  4. `ts7_3_left_and_right_stop_at_first_and_last_column`: given column 0 focused, when `h` is pressed, then it stays; given the `as7` app with the extra column focused, when `l` is pressed, then it stays.
  5. `ts7_3_column_change_keeps_row_clamped`: given Todo with three cards and row 2 selected and Doing with one card, when `l` is pressed, then Doing's only card is selected.
  6. `ts7_3_empty_column_has_no_selection`: given a board whose Doing is empty, when `l` moves the focus there, then `sel.card` is `None` and `j` and `k` change nothing.
  7. `ts7_3_column_scrolls_to_keep_selection_visible`: given 30 Todo cards written with `write_card` and the newest selected, when `j` is pressed 25 times and the app is rendered at 80x24, then the selected card's ID is on screen, the newest card's ID is not, and Doing still shows its first card.
  8. `ts7_3_visible_columns_centre_on_focus`: given 6 columns and width 80, when `visible_columns` runs with focus 0, 3 and 5, then it returns `0..3`, `2..5` and `3..6`.
  9. `ts7_3_narrow_columns_scroll_around_focused`: given the `six_columns` app with `D` focused, when rendered at 80x24, then the header row holds `C (1)`, `> D (1)` and `E (1)` and no `A`, `B` or `F` header.
  10. `ts5_2_rebound_navigation_keys`: given keys built from `down = "x"`, when `x` is pressed, then the selection moves down, and `j` changes nothing.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `visible_columns` and the vertical offset are pure functions; `App` keeps no scroll state for the board (review).

### E4-F2-T3 Ages and refresh

- Status: todo
- Depends on: E4-F2-T1, E4-F1-T7
- Covers: FR-21; TECHSPEC 7.3 (age: `now`, `N min`, `N h`, `N d`, `?`, floored, redrawn at least every 30 seconds)
- Size: S
- Scope: Cards (and the panel in E4-F2-T4) render ages with `core::time::age(note.at, &app.now)`; `Tick` sets `app.now` (E4-F1-T7) and the runtime redraws after every tick, once a second, so ages refresh well within 30 seconds. No other age arithmetic exists in `src/tui/`.
- Not in scope: the age rule itself (E1-F1-T5); the detail view's age (E4-F3-T1).
- Tests first:
  1. `fr21_card_ages_are_floored`: given seven Todo cards whose latest notes are 59 s, 60 s, 59 min 59 s, 23 h 59 min and 42 h 47 min before `NOW`, 5 min after it, and one with the heading `### someday`, when their `card_lines` are built, then the second lines start `  now`, `  1 min`, `  59 min`, `  23 h`, `  1 d`, `  now` and `  ?`.
  2. `fr21_tick_refreshes_card_age`: given the `work_4_9` app showing `39 min` on W-12, when `update` gets `Tick(NOW + 60 s)` and the app is rendered, then W-12's second row shows `40 min`.
  3. `fr21_idle_runtime_refreshes_ages_within_30_seconds`: given a runtime whose `FakeClock` starts at `2026-10-08T11:52:30+03:00` (W-12 at `39 min`), when `FakeEvents` delivers 120 timeouts of 250 ms and then `q`, then the last frame before `q` shows `40 min` on W-12.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `src/tui/` computes no duration itself: every age comes from `core::time::age` (grep for `Span` and `as_secs` in `src/tui/`).

### E4-F2-T4 Preview panel and the 4.9 snapshots

- Status: todo
- Depends on: E4-F2-T3
- Covers: FR-22; TECHSPEC 4.9 (the drawing), 7.3 (preview panel, hidden below 20 rows), 12.1 (snapshots at 80x24 and 120x40); PRD A-19
- Size: M
- Scope: `panel_lines(card, now)` and its drawing just above the status line when the area has at least `PANEL_MIN_ROWS` rows: a separator row of `-` across the width after the margin, then ` <ID> latest: <age_ago>[, auto][, pane <pane>][, <cwd>]` and ` Doing:      `, ` Next:       `, ` Watch out:  ` with the first line of each part of the latest note; a card without notes gives ` <ID> no note yet` and no part rows; no selected card gives only the separator (A-E4-10). Card rows end above the panel.
- Not in scope: the detail view's full note (E4-F3); the jump target (E5-F3).
- Tests first:
  1. `fr22_panel_shows_latest_note_with_pane_and_cwd`: given W-12 of `work_4_9` at `NOW`, when `panel_lines` runs, then it returns `W-12 latest: 39 min ago, pane w4V:p9, /home/smilen/Work/ati.billing`, `Doing:      migration test passes on staging`, `Next:       open the MR, ask the DBA for the v13 deploy list` and `Watch out:  nothing new`.
  2. `fr22_panel_marks_auto`: given a latest note marked `auto`, 12 minutes old, with a cwd and no pane, when `panel_lines` runs, then the first line is `<ID> latest: 12 min ago, auto, <cwd>`.
  3. `fr22_panel_without_pane_shows_cwd_only`: given W-9 of `work_4_9`, when `panel_lines` runs, then the first line is `W-9 latest: 1 d ago, /home/smilen/Work/private/brain-swap`.
  4. `fr22_panel_without_notes_says_no_note_yet`: given W-11, when `panel_lines` runs, then it returns only `W-11 no note yet`.
  5. `fr22_panel_shows_first_line_of_each_part`: given a latest note whose Doing has a continuation line, when `panel_lines` runs, then the Doing row holds only the first line.
  6. `fr22_panel_with_no_selection_shows_only_separator`: given an empty Todo focused, when rendered at 80x24, then the separator row is drawn and no row holds `latest:`.
  7. `fr22_panel_hidden_below_20_rows`: given W-12 selected, when rendered at 80x19, then no row consists of `-` and none holds `latest:`, and when rendered at 80x20, then both are present.
  8. `ts4_9_board_snapshot_80x24`: given the `work_4_9` app at `NOW` with W-12 selected, when rendered at 80x24, then an insta snapshot pins the frame, whose rows show the 4.9 content: the header, `> Doing (1)`, the three cards with `no note`, `39 min  open the MR, ask` (cut) and `1 d  nothing left`, the separator and the four panel rows.
  9. `ts4_9_board_snapshot_120x40`: given the same app, when rendered at 120x40, then an insta snapshot pins the frame.
  10. `nfr10_panel_uses_no_colour`: given the buffers of tests 8 and 9, when `assert_no_colour` runs, then it passes.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Snapshots reviewed with `cargo insta review` and committed; where the 80x24 frame's column widths differ from the 4.9 drawing (A-E4-09), the commit message says so.

### E4-F2-T5 Moves

- Status: todo
- Depends on: E4-F2-T2, E4-F1-T6, E1-F6-T3
- Covers: FR-29; TECHSPEC 4.7 (`H` or `L` on an extra column moves to the first configured column; no move targets an extra column), 7.1 (`move_*` emits `Move`; `Moved` reloads and selects the card, no Message; `Failed` sets the status line), 7.3 (`H` on the first column and `L` on the last configured column do nothing)
- Size: M
- Scope: `move_target(board, card, right)`: the next or previous configured column; none for `H` in the first column and for `L` in the last configured column; for a card in an extra column the first configured column for both keys. Board `move_left` and `move_right` with a selected card and a target emit `Move(id, target)`. Runtime `Effect::Move`: `core::store::move_card(env, &app.board, id, &column)`; on `Ok` it reloads with `core::board::load` and dispatches `Reloaded` then `Done(Moved(id))` (A-E4-05); on `Err(e)` it dispatches `Done(Failed(e.to_string()))`. `update` on `Done(Moved(id))` selects `id`, focuses its column and opens no Message.
- Not in scope: the splice and its byte rules (E1-F6-T3); moves from the CLI (E2-F2-T6).
- Tests first:
  1. `fr29_l_emits_move_to_next_column`: given the `work_4_9` app with W-11 (Todo) selected, when `L` is pressed, then the effects are `[Move(W-11, "Doing")]`.
  2. `fr29_h_emits_move_to_previous_column`: given W-12 (Doing) selected, when `H` is pressed, then the effects are `[Move(W-12, "Todo")]`.
  3. `ts7_3_h_on_first_column_does_nothing`: given W-11 selected, when `H` is pressed, then there is no effect.
  4. `ts7_3_l_on_last_configured_column_does_nothing`: given W-9 (Done) selected, when `L` is pressed, then there is no effect.
  5. `ts4_7_moves_from_extra_column_target_first_column`: given the `as7` app with W-13 (`Doign (unknown)`) selected, when `H` is pressed, then the effects are `[Move(W-13, "Todo")]`, and `L` gives the same.
  6. `fr29_move_without_selected_card_does_nothing`: given an empty column focused, when `L` is pressed, then there is no effect.
  7. `fr29_moved_card_stays_selected`: given a runtime on a `work_4_9` copy with W-11 selected, when `L` is dispatched, then `W-11.md` has `column: Doing`, `sel` is column 1 with card W-11, `status` is `None` and the mode is `Board`.
  8. `fr29_move_failure_sets_status_line`: given `W-11.md` replaced by a file holding a `<<<<<<<` line, when `L` is dispatched, then `status` is the `Unreadable` error's text, the file is byte for byte unchanged and W-11 is still selected.
  9. `ts5_2_rebound_move_keys`: given keys built from `move_right = "m"`, when `m` is pressed on W-11, then the effects are `[Move(W-11, "Doing")]`, and `L` gives none.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The runtime's Move arm takes no lock itself and hands the store the loaded board only for its name, path and columns (review).

### E4-F2-T6 AS-7 and AS-9 boards

- Status: todo
- Depends on: E4-F2-T4, E4-F1-T8
- Covers: FR-20; TECHSPEC 4.7 (hand-made card, unknown column); AS-7 (TUI rendering), AS-9 (TUI board)
- Size: S
- Scope: Acceptance render tests for the hand edits of AS-7 and the first run of AS-9. Keeping the selection across the AS-7 edit is pinned by E4-F1-T7; the next number after a hand-made card by E4-F4-T3.
- Not in scope: loading rules (E1-F5-T5); the config write and its notice text (E1-F3-T3, E4-F1-T8).
- Tests first:
  1. `as7_hand_made_card_renders_in_first_column_with_no_note`: given a runtime on a `work_4_9` copy, when `W-20.md` holding only `# spike on caching` is written and `tick` runs, then the Todo column renders `W-20 spike on caching` over `  no note`.
  2. `as7_misspelt_column_renders_as_unknown_extra_column`: given a runtime on a fresh `work_4_9` copy, when `W-13.md` with `column: Doign` is written and `tick` runs, then the last column header is `Doign (unknown) (1)` and Help lists `W-13: unknown column 'Doign'`.
  3. `as7_board_snapshot_80x24`: given the `as7` app at `NOW`, when rendered at 80x24, then an insta snapshot pins the frame and `assert_no_colour` passes.
  4. `as9_first_run_shows_notice_and_empty_board`: given `TuiFixture::without_config(None)`, when `tui::gate(&env, None, true)` runs (its `core::config::load` writes the config and returns `created: Some(<path>)`) and `tui::prepare` runs on that `TuiStart`, then the frame at 80x24 shows `> Todo (0)`, `Doing (0)` and `Done (0)`, the last row is ` created config <path>`, `insta::assert_snapshot!` of the rendered frame with the temp root replaced through `str::replace` pins it, and no board folder exists.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Snapshots reviewed with `cargo insta review` and committed (12.2 item 3).

## E4-F3 Detail view

- Depends on: E4-F1
- Covers: FR-26, FR-27
- Spec: TECHSPEC 4.7 (notes without a place, bad stamps), 5.2 (Detail: `up` newer, `down` older, `scroll_*`, `help`, `quit` back), 7.1 (Board `open` shows Detail on the newest note; Detail `down`/`up` clamped), 7.2 (Detail), 7.3 (age), 7.4 (reload while in Detail), 12.1 (snapshots); PRD 7.6 (FR-26, FR-27), AS-1 step 6
- Scope: `open` on a selected card shows its detail on the newest note. The screen reads `<ID> <title>` with column and board, `note n/N` with stamp and age (and `auto`), the three parts, `Where: <cwd> (pane <pane>)`, a separator, the card body that `scroll_down` and `scroll_up` move, and a key hint line built from the current bindings. `j` goes to older notes and `k` to newer ones, clamped. Detail-mode Help returns to the same note, `quit` returns to the board, and a reload keeps the note shown or falls back to the board when the card is gone. Jumping from Detail (FR-28) is E5-F3; editing from Detail is E4-F5.
- Provides: `tui::view_detail::draw(frame: &mut Frame, area: Rect, app: &App)`; `tui::view_detail::note_lines(note: &Note, index: usize, count: usize, now: &Stamp) -> Vec<String>`; `tui::view_detail::where_line(place: &Place) -> String`; `tui::view_detail::hint_line(keys: &KeyMap) -> String`; `tui::app::SCROLL_STEP` (10); fixture `tests/fixtures/boards/detail/`
- Requires: E4-F1 (`App`, `Mode::Detail`, `update`, `view_frame::draw`, Help, `Reloaded` handling, `tests/common/tui.rs`), E1-F2 (`core::model::Note`), E1-F1 (`core::model::Place`, `core::model::HerdrPlace`, `core::time::format_stamp`, `core::time::age`), E1-F5 (`Board::card`, fixture `work_4_9`)
- Feature DoD:
  - [ ] `open` shows the newest note with its indicator, stamp, age, parts and place (E4-F3-T1).
  - [ ] First, middle and last note snapshots (E4-F3-T2).
  - [ ] The body scrolls with `scroll_down` and `scroll_up`, clamped, and the hint line shows the current keys (E4-F3-T3).
  - [ ] A card without notes, a note without a place or with a bad stamp, Help from Detail and reloads behave as stated (E4-F3-T4).

### E4-F3-T1 Open the detail view

- Status: todo
- Depends on: E4-F1-T3
- Covers: FR-26, FR-27; TECHSPEC 5.2 (Detail `quit` returns to Board; Board-only actions do nothing in Detail), 7.1 (Board `open`), 7.2 (Detail header, note block, `Where:` line)
- Size: M
- Scope: Board `open` with a selected card sets `Detail { card, note: <index of the last note>, scroll: 0 }` (index 0 without notes); with no card it does nothing. Detail `quit` returns to Board with the selection unchanged and no effect. `src/tui/view_detail.rs`, called by `view_frame` in Detail mode: ` <ID> <title> (<column>, <board>)`; `note_lines`: ` note <index + 1>/<count>, <format_stamp(at)>, <age_ago>[, auto]`, then ` Doing:      `, ` Next:       `, ` Watch out:  ` with each part's text, continuation lines as further rows indented to the text, wrapped to the width (A-E4-11); `where_line`: ` Where: <cwd> (pane <pane>)`, ` Where: <cwd>` without a pane, ` Where: none` without a place. Fixture `tests/fixtures/boards/detail/`: `board.md` (`letter: W`, `next: 7`); `W-5.md` (Research, Doing, title `measure merge latency`, created `2026-10-06T09:00:00+03:00`, a body of `## Question` directly under the title line followed by the lines `line 1` to `line 40`, three notes: `2026-10-07T09:00:00+03:00 auto` with Doing `profiled the merge`, Next `try a smaller lock`, Watch out `numbers are noisy` and place `cwd=/w/latency pane=w4V:p3 tab=w4V:t1 workspace=w4V session=s-1`; `2026-10-08T10:00:00+03:00` with Doing `two runs done` continued by `third run pending`, Next `compare with v12`, Watch out `nothing` and place `cwd=/w/latency`; `2026-10-08T11:40:00+03:00` with Doing `wrote the summary`, Next `share it`, Watch out `none` and no place line); `W-4.md` (Chore, Todo, `tidy notes`, no notes); `W-6.md` (Bug, Todo, `odd stamp`, one note headed `### someday` with place `cwd=/w/odd pane=w4V:p5`).
- Not in scope: note navigation (E4-F3-T2); the body and hint line (E4-F3-T3); edge cases (E4-F3-T4); `jump` in Detail (E5-F3); `edit` in Detail (E4-F5-T1).
- Tests first:
  1. `fr26_open_shows_newest_note`: given the `work_4_9` app with W-12 selected, when `o` is pressed, then the mode is `Detail { card: W-12, note: 1, scroll: 0 }` and there is no effect.
  2. `fr26_open_without_selected_card_does_nothing`: given an empty column focused, when `o` is pressed, then the mode stays `Board`.
  3. `fr27_detail_header_shows_id_title_column_and_board`: given Detail on W-12, when rendered at 80x24, then row 0 is ` W-12 migrate invoices to v13 (Doing, work)`.
  4. `fr26_note_line_shows_indicator_stamp_and_age`: given Detail on W-12 at `NOW`, when rendered, then row 1 is ` note 2/2, 2026-10-08T11:12:40+03:00, 39 min ago`.
  5. `fr27_detail_shows_three_parts`: given Detail on W-12, when rendered, then the next rows are ` Doing:      migration test passes on staging`, ` Next:       open the MR, ask the DBA for the v13 deploy list` and ` Watch out:  nothing new`.
  6. `fr27_where_line_shows_cwd_and_pane`: given Detail on W-12, when rendered, then the row after the parts is ` Where: /home/smilen/Work/ati.billing (pane w4V:p9)`.
  7. `ts5_2_quit_in_detail_returns_to_board`: given Detail on W-12, when `q` is pressed (and, from a fresh Detail, `esc`), then the mode is `Board`, W-12 is still selected and the effects are empty.
  8. `ts5_2_board_keys_do_nothing_in_detail`: given Detail on W-12, when `n`, `H`, `L`, `b` and `l` are pressed, then the mode is still that Detail and there is no effect.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `assert_no_colour` passes on the frame of test 3.

### E4-F3-T2 Note navigation and snapshots

- Status: todo
- Depends on: E4-F3-T1
- Covers: FR-26; TECHSPEC 5.2 (Detail `up` newer, `down` older), 7.1 (note index clamped), 12.1 (snapshots at 80x24 and 120x40); AS-1 step 6
- Size: S
- Scope: In Detail `down` moves `note` one older and `up` one newer, both clamped to the card's notes; `scroll` is kept. Snapshots of the first, middle and last note of `W-5`.
- Not in scope: body scrolling (E4-F3-T3).
- Tests first:
  1. `as1_j_shows_older_note_and_k_newest_again`: given Detail on W-12 showing `note 2/2`, when `j` is pressed, then the frame shows `note 1/2, 2026-10-08T10:31:05+03:00, 1 h ago, auto` and Doing `switched InvoiceRepository to v13, unit tests green`, and when `k` is pressed, then it shows `note 2/2` again.
  2. `fr26_note_navigation_clamps`: given Detail on W-12 at `note 2/2`, when `k` is pressed, then it stays `2/2`, and when `j` is pressed twice, then it shows `1/2`.
  3. `fr26_indicator_counts_from_oldest`: given the `detail` fixture with W-5 selected, when `o`, `j` and `j` are pressed, then the indicator reads `note 3/3`, `note 2/3` and `note 1/3` in turn.
  4. `ts5_2_rebound_detail_keys`: given keys built from `down = "x"`, when `x` is pressed in Detail on W-12, then `note 1/2` is shown and `j` changes nothing.
  5. `fr26_detail_note_snapshots`: given Detail on W-5, when rendered at 80x24 on note 3/3, 2/3 and 1/3 and at 120x40 on note 3/3, then four insta snapshots pin the frames and `assert_no_colour` passes on each.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Snapshots reviewed with `cargo insta review` and committed (12.2 item 3).

### E4-F3-T3 Body, scroll and key hints

- Status: todo
- Depends on: E4-F3-T1
- Covers: FR-27; TECHSPEC 5.2 (Detail `scroll_down`, `scroll_up`), 7.2 (separator, scrollable body, key hint line)
- Size: S
- Scope: Under the `Where:` row a separator row, then the card body from line `scroll`, cut at the width and not wrapped (A-E4-11); on the row above the status line the hint line from `hint_line(keys)`: ` <down> older  <up> newer  <scroll_down> down  <scroll_up> up  <jump> jump  <edit> edit  <help> help  <quit> back`, each the action's first binding in 5.3 syntax. `scroll_down` and `scroll_up` move `scroll` by `SCROLL_STEP`, clamped to 0 and to the body's last line (A-E4-12).
- Not in scope: what `jump` and `edit` do (E5-F3, E4-F5-T1).
- Tests first:
  1. `fr27_body_follows_separator`: given Detail on W-5 at 80x24, when rendered, then the row after the separator is ` ## Question` and the next is ` line 1`.
  2. `fr27_scroll_down_moves_body_by_step`: given that Detail, when `ctrl+d` is pressed, then `scroll` is 10 and the first body row is ` line 10`.
  3. `fr27_scroll_up_clamps_at_top`: given `scroll` 0, when `ctrl+u` is pressed, then it stays 0; given `scroll` 10, when `ctrl+u` is pressed, then it is 0.
  4. `fr27_scroll_down_clamps_at_last_line`: given Detail on W-5, when `ctrl+d` is pressed six times, then `scroll` equals the loaded body's line count minus one and the first body row is the body's last line.
  5. `fr27_scroll_kept_when_changing_note`: given `scroll` 10, when `j` is pressed, then `scroll` is still 10.
  6. `ts7_2_hint_line_shows_current_keys`: given the defaults, when Detail is rendered at 80x24, then row 22 is ` j older  k newer  ctrl+d down  ctrl+u up  enter jump  e edit  ? help  q back`, and with keys built from `down = "x"` it starts ` x older`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `hint_line` reads every key from `KeyMap::keys`, never from a literal (review).

### E4-F3-T4 Detail edge cases

- Status: todo
- Depends on: E4-F3-T2, E4-F3-T3, E4-F1-T4, E4-F1-T7
- Covers: FR-26, FR-27; TECHSPEC 4.7 (bad stamp shows `?`, a note without a place), 7.1 (Detail `help`), 7.4 (reload while in Detail)
- Size: S
- Scope: A card without notes shows ` no note yet` in place of the note block, and `up` and `down` do nothing. A bad stamp shows the heading text after `### ` and the age `? ago`. Detail `help` opens `Help { back: <the Detail> }` and any key returns to the same Detail. `Reloaded` in Detail keeps `note` (clamped to the new count) and `scroll`, and returns to Board when the card is gone, also for a Detail held in Help's `back` (A-E4-13).
- Not in scope: reload rules on the board (E4-F1-T7); `Edited` in Detail (E4-F5-T4).
- Tests first:
  1. `fr27_card_without_notes_shows_no_note_yet`: given Detail on W-4 of the `detail` fixture, when rendered, then a row reads ` no note yet` and no row starts ` note `, and `j` and `k` leave the mode unchanged.
  2. `fr27_note_without_place_shows_where_none`: given Detail on W-5 at note 3/3, when rendered, then the `Where:` row is ` Where: none`.
  3. `fr27_note_with_cwd_only_shows_cwd`: given Detail on W-5 at note 2/3, when rendered, then the `Where:` row is ` Where: /w/latency` and the Doing rows show `two runs done` and `third run pending`.
  4. `fr27_bad_stamp_shows_heading_text_and_question_mark`: given Detail on W-6, when rendered, then row 1 is ` note 1/1, someday, ? ago` and the `Where:` row is ` Where: /w/odd (pane w4V:p5)`.
  5. `fr32_help_from_detail_returns_to_detail`: given Detail on W-5 at note 2/3 with `scroll` 10, when `?` and then `x` are pressed, then the mode is `Detail { card: W-5, note: 1, scroll: 10 }`.
  6. `fr23_reload_in_detail_keeps_shown_note`: given Detail on W-5 at note 2/3, when a note is appended to `W-5.md` by hand and `Reloaded` is applied, then the mode is still note index 1 and the indicator reads `note 2/4`.
  7. `fr23_reload_with_card_gone_returns_to_board`: given Detail on W-5, and separately Help over that Detail, when `W-5.md` is deleted and `Reloaded` is applied, then the mode is `Board` in both cases.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] An insta snapshot of test 1's frame at 80x24 is reviewed and committed.

## E4-F4 Pickers

- Depends on: E4-F1, E1-F6
- Covers: FR-24, FR-30
- Spec: TECHSPEC 5.2 (pickers use `up`, `down`, `confirm`, `cancel` and template hotkeys; TitleInput rules; hotkeys checked after the action keys), 5.4 (hotkeys, templates without a key, user templates), 7.1 (Board `new` and `switch_board`, a hotkey opens TitleInput, TitleInput `confirm` emits `Create`, BoardPick `confirm` emits `LoadBoard`, `Created`, start selection after `LoadBoard`), 7.2 (template and board pickers, title input), 7.4 (polling the new board); PRD 7.5 (FR-24), 7.7 (FR-30), A-25, A-27; AS-6 (TUI creation), AS-7 (next number)
- Scope: The new-card flow and the board switch. `n` opens the template picker, where a template's hotkey (or `up`, `down` and `confirm` for one without) opens the title input; the title input takes every printable character, `backspace` and `cancel`, and `confirm` creates the card through `core::store::create` in the focused column and selects it. Templates come from the built-ins and the user folder, with hotkeys colliding with the picker keys dropped and reported. `b` opens the board picker over the configured boards; `confirm` loads the chosen board with the A-27 selection, and polling follows it.
- Provides: fields `App.templates: Vec<(String, Option<char>)>` (name and final hotkey, load order) and `App.boards: Vec<String>` (config order); field `tui::Prepared.templates: Vec<Template>`; `tui::view_pick::draw_templates(frame: &mut Frame, area: Rect, app: &App)`, `tui::view_pick::draw_title(frame: &mut Frame, area: Rect, app: &App)`, `tui::view_pick::draw_boards(frame: &mut Frame, area: Rect, app: &App)`; the runtime's `Effect::Create` and `Effect::LoadBoard` arms
- Requires: E4-F1 (`App`, `Mode::TemplatePick`, `Mode::TitleInput`, `Mode::BoardPick`, `update`, `Runtime`, `tui::prepare`, `tests/common/tui.rs`), E1-F4 (`core::template::load`, `core::template::find`, `core::template::Template`), E1-F3 (`KeyMap::check_template_hotkeys`, `Config::board`, `Config.boards`), E1-F6 (`core::store::create`, `core::store::CreateRequest`, `Created`), E1-F5 (`core::board::load`, fixtures `work_4_9` and `as7`), E1-F1 (`Env::templates_dir`)
- Feature DoD:
  - [ ] Template hotkeys, templates without a key, user templates and dropped hotkeys behave as 5.2 and 5.4 say (E4-F4-T1).
  - [ ] AS-6 creation emits one `Create` (E4-F4-T2).
  - [ ] `j` and `k` type into a title (E4-F4-T2).
  - [ ] A created card appears selected in the focused column, and the number skips a hand-made card (E4-F4-T3).
  - [ ] The switch-board key lists the configured boards and opens the chosen one with the A-27 selection, and polling follows it (E4-F4-T4).

### E4-F4-T1 Template picker

- Status: todo
- Depends on: E4-F1-T8, E1-F4-T3, E1-F3-T6
- Covers: FR-30; TECHSPEC 5.2 (pickers; template hotkeys checked after the action keys, colliding ones dropped), 5.4 (templates without a key are chosen with `up`, `down` and `confirm`; user files; load warnings), 7.1 (Board `new` opens TemplatePick; a hotkey opens TitleInput), 7.2 (template picker)
- Size: M
- Scope: `prepare` loads `core::template::load(&env.templates_dir())`, passes the loaded hotkeys to `KeyMap::check_template_hotkeys`, and stores `app.templates` and `Prepared.templates`; both warning lists join `app.warnings` after the key warnings (A-E4-08). Board `new` sets `TemplatePick { sel: 0 }`. In TemplatePick (`KeyContext::Picker`): `up` and `down` move `sel`, clamped; `confirm` opens `TitleInput { template: <name at sel>, buf: "" }`; `cancel` returns to Board; a printable character without ctrl or alt equal to a template's hotkey opens TitleInput for that template; any other key does nothing. `src/tui/view_pick.rs` `draw_templates`: ` new card in <focused column>: pick a template`, then one row per template, ` <hotkey or a blank>  <name>`, the selected row starting `>` in place of the margin and drawn in reverse video (A-E4-17).
- Not in scope: the title input (E4-F4-T2); creating the card (E4-F4-T3); the template loading rules themselves (E1-F4).
- Tests first:
  1. `fr30_new_key_opens_template_pick`: given the `work_4_9` app, when `n` is pressed, then the mode is `TemplatePick { sel: 0 }` and there is no effect.
  2. `fr30_hotkey_opens_title_input_for_template`: given that picker, when `b` is pressed, then the mode is `TitleInput { template: "Bug", buf: "" }`.
  3. `ts5_4_keyless_template_chosen_with_down_and_confirm`: given `spike.md` with `name: Spike` and no `key` in the templates folder and the app prepared, when `n`, four `j` and `enter` are pressed, then the mode is `TitleInput { template: "Spike", buf: "" }`.
  4. `ts5_2_template_pick_up_down_clamp`: given the picker over the four built-ins, when `k` is pressed, then `sel` stays 0, and when `j` is pressed five times, then `sel` is 3.
  5. `ts5_2_cancel_in_template_pick_returns_to_board`: given the picker, when `esc` is pressed, then the mode is `Board` and the effects are empty (no `Quit`).
  6. `fr30_user_template_hotkey_works`: given `spike.md` with `name: Spike` and `key: s`, when the app is prepared and `n`, `s` are pressed, then the mode is `TitleInput { template: "Spike", .. }`, and the picker frame lists ` s  Spike` last.
  7. `ts5_2_colliding_hotkey_dropped_with_warning`: given `spike.md` with `key: j`, when the app is prepared and `n`, `j` are pressed, then `sel` is 1 and `app.warnings` holds `template Spike: hotkey 'j' collides with down, dropped`.
  8. `ts5_4_template_load_warning_reaches_help`: given `bad.md` holding a 0xFF byte in the templates folder, when the app is prepared, then `app.warnings` holds a text starting `template bad.md: `.
  9. `ts5_2_board_keys_do_nothing_in_template_pick`: given the picker, when `q`, `H` and `o` are pressed, then the mode is still `TemplatePick { sel: 0 }` and there is no effect.
  10. `fr30_template_pick_snapshots`: given the picker over the built-ins with Todo focused, when rendered at 80x24 and 120x40, then insta snapshots pin both and `assert_no_colour` passes.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `check_template_hotkeys` runs after `KeyMap::build` in `prepare` (review, as its doc comment requires).

### E4-F4-T2 Title input

- Status: todo
- Depends on: E4-F4-T1
- Covers: FR-30; TECHSPEC 5.2 (TitleInput: printable characters and space inserted, `backspace`, `confirm` ignored while the trimmed title is empty, `cancel`, printable `confirm` or `cancel` bindings ignored there), 7.1 (`confirm` emits `Create` in the focused column), 7.2 (title input); PRD A-25; AS-6 (creation keys)
- Size: S
- Scope: In TitleInput (`KeyContext::TitleInput`): `confirm` with a non-blank trimmed `buf` emits `Create { template, title: <trimmed buf>, column }` and returns to Board, and is ignored while the trimmed `buf` is empty; `cancel` returns to Board; `backspace` removes the last character; every other printable character without ctrl or alt, and `space`, is appended; nothing else applies. `column` is the focused column's name, or the first configured column when an extra column is focused (A-E4-16). `draw_title`: ` new <template> card in <column>` and ` title: <buf>`, with the terminal cursor after the text.
- Not in scope: running `Create` (E4-F4-T3); the TitleInput key rules in the key map (E1-F3-T6).
- Tests first:
  1. `fr30_j_and_k_type_into_title`: given the `work_4_9` app, when `n`, `f`, `j` and `k` are pressed, then the mode is `TitleInput { template: "Feature", buf: "jk" }`.
  2. `ts5_2_title_input_takes_every_printable_character`: given TitleInput with an empty `buf`, when `q`, `?`, `H`, `space` and `é` are pressed, then `buf` is `q?H é` and there is no effect.
  3. `ts5_2_backspace_deletes_last_character`: given `buf` `abc`, when `backspace` is pressed, then `buf` is `ab`, and on an empty `buf` it stays empty.
  4. `ts5_2_confirm_ignored_while_title_blank`: given `buf` empty and, separately, `buf` of three spaces, when `enter` is pressed, then there is no effect and the mode is still TitleInput.
  5. `as6_creation_emits_one_create_in_focused_column`: given Todo focused, when `n`, `f`, the characters of `fix login` and `enter` are pressed, then the effects of the whole sequence are exactly `[Create { template: "Feature", title: "fix login", column: "Todo" }]` and the mode is `Board`.
  6. `fr30_title_is_trimmed_and_column_is_focused_one`: given Doing focused and `buf` `  spike  `, when `enter` is pressed, then the effects are `[Create { template: "Feature", title: "spike", column: "Doing" }]`.
  7. `ts5_2_cancel_in_title_input_returns_to_board`: given TitleInput with `buf` `abc`, when `esc` is pressed, then the mode is `Board` and there is no effect.
  8. `ts5_2_printable_confirm_binding_types_in_title`: given keys built from `confirm = ["enter", "y"]`, when `y` is pressed in TitleInput, then `buf` ends with `y`, and `enter` still confirms.
  9. `ts5_2_ctrl_and_alt_keys_are_not_inserted`: given `buf` `ab`, when `ctrl+d` and `alt+x` are pressed, then `buf` is still `ab`.
  10. `fr30_create_from_extra_column_uses_first_column`: given the `as7` app with `Doign (unknown)` focused, when `n`, `f`, `x` and `enter` are pressed, then the effect's `column` is `Todo`.
  11. `fr30_title_input_snapshots`: given TitleInput for Feature in Todo with `buf` `fix login`, when rendered at 80x24 and 120x40, then insta snapshots pin both and `assert_no_colour` passes.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Snapshots reviewed with `cargo insta review` and committed (12.2 item 3).

### E4-F4-T3 Create in the runtime

- Status: todo
- Depends on: E4-F4-T2, E4-F1-T6, E1-F6-T5
- Covers: FR-30; TECHSPEC 7.1 (`Create` effect; `Created` reloads and selects the card, no Message; `Failed`); PRD A-25; AS-6 (the card appears in the focused column, selected), AS-7 (the next card is W-21)
- Size: S
- Scope: Runtime `Effect::Create { template, title, column }`: `core::template::find(&prepared.templates, &template)` and `core::store::create(env, &app.board, &CreateRequest { title, template, column: Some(column), body: None })`; on `Ok` it reloads, dispatches `Reloaded` then `Done(Created(id))` (A-E4-05); on `Err(e)` it dispatches `Done(Failed(e.to_string()))`. `update` on `Done(Created(id))` selects `id` in its column and opens no Message. The TUI resolves no session and sets no reference.
- Not in scope: the store's allocation and rendering rules (E1-F6-T5); moving the new card with `L` (E4-F2-T5; AS-6 end to end is E5-F3).
- Tests first:
  1. `as6_created_card_appears_selected_in_focused_column`: given a runtime on a `work_4_9` copy with Doing focused, when `n`, `f`, `fix login` and `enter` are dispatched, then `W-13.md` exists with `column: Doing`, `template: Feature` and title `fix login`, `sel` is column 1 with card W-13, `status` is `None` and the mode is `Board`.
  2. `as7_tui_creation_skips_past_hand_made_card`: given a runtime on an `as7` copy, when `n`, `f`, `x` and `enter` are dispatched, then the new card is W-21.
  3. `fr30_create_failure_sets_status_line`: given `board.md` replaced by `---\nletter W\n---\n`, when a card is created through the keys, then `status` starts with `board.md:2: ` and no new card file exists.
  4. `fr30_create_on_missing_board_folder_creates_it`: given `TuiFixture::new(None)` (no board folder), when a card is created through the keys, then `W-1.md` exists and is selected.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] After test 1 the state folder holds no `sessions/` entry (the TUI sets no reference).

### E4-F4-T4 Board picker and switching boards

- Status: todo
- Depends on: E4-F1-T8, E4-F1-T7
- Covers: FR-24; TECHSPEC 7.1 (Board `switch_board` opens BoardPick; `confirm` emits `LoadBoard`; `cancel`; the start selection after `LoadBoard`), 7.2 (board picker), 7.4 (the runtime fingerprints the new board); PRD A-27
- Size: M
- Scope: `prepare` stores `app.boards` in config order. Board `switch_board` sets `BoardPick { sel: <index of the current board> }` (A-E4-17); `up` and `down` move `sel`, clamped; `confirm` returns to Board and emits `LoadBoard(<name at sel>)`; `cancel` returns to Board. Runtime `Effect::LoadBoard(name)`: `Config::board` and `core::board::load`; `Ok` dispatches `Reloaded(board)`, `Err(e)` dispatches `Done(Failed(e.to_string()))` and keeps the current board (A-E4-18). `update` on `Reloaded` with another board name replaces the board, sets `initial_selection` and returns to Board (A-E4-05). From then on `tick` fingerprints the new board's folder. `draw_boards`: ` switch board`, then one row per board, ` <name>  <path with HOME shown as ~>`, the selected row starting `>` and drawn in reverse video.
- Not in scope: the gate's unknown `--board` and opening on `--board` at start (E4-F1-T8).
- Tests first:
  1. `fr24_switch_board_lists_configured_boards`: given the `work_4_9` app prepared with the default config, when `b` is pressed, then the mode is `BoardPick { sel: 0 }` and the frame lists `work`, `home` and `personal` in that order.
  2. `fr24_confirm_emits_load_board`: given that picker, when `j` and `enter` are pressed, then the effects are `[LoadBoard("home")]` and the mode is `Board`.
  3. `ts5_2_cancel_in_board_pick_returns`: given the picker, when `esc` is pressed, then the mode is `Board` and there is no effect.
  4. `fr24_pick_starts_on_current_board`: given an app prepared on `home`, when `b` is pressed, then `sel` is 1.
  5. `fr24_switched_board_opens_with_initial_selection`: given a runtime on work and the home folder holding H-1 (Todo, created 09:00) and H-2 (Doing, a note at 11:00), when `b`, `j` and `enter` are dispatched, then `app().board.name` is `home` and `sel` is column 1 with card H-2.
  6. `ts7_4_polling_follows_switched_board`: given test 5's runtime with `env.log` set, when `H-3.md` is written in the home folder and `tick` runs, then H-3 is on the board, and when a work card file changes and `tick` runs, then no further `reload` line is logged.
  7. `fr24_switch_failure_keeps_current_board`: given the home folder with mode 000 (skipped as root), when `b`, `j` and `enter` are dispatched, then `status` starts with `board home: ` and the board is still `work`.
  8. `fr24_board_pick_snapshots`: given the picker on `work`, when rendered at 80x24 and 120x40, then `insta::assert_snapshot!` of each rendered frame with the temp root replaced through `str::replace` pins both and `assert_no_colour` passes.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Snapshots reviewed with `cargo insta review` and committed (12.2 item 3).

## E4-F5 Editor

- Depends on: E4-F1, E1-F6, E2-F3
- Covers: FR-31
- Spec: TECHSPEC 4.8 (R6), 5.2 (`editor`, `$VISUAL`, `$EDITOR`, `vi`; split on ASCII whitespace, copy path last), 6.1 (`BRAIN_SWAP_FAILPOINT`), 7.1 (`edit` emits `Edit`; `Edited` reloads and selects; `Failed` for a non-zero editor exit), 7.5 (around the editor), 7.6, 10 (no lock across an editor; only the TUI's `e` merges), 12.1 (TUI runtime test with a scripted editor, process-level `edit:after_tmp` crash); PRD 7.7 (FR-31), NFR-08 (the TUI editor), A-09, AS-8
- Scope: `e` on the board or in the detail view copies the card's bytes `C0` to `$XDG_STATE_HOME/brain-swap/edit/<board>-<ID>.md`, leaves the terminal, runs the configured editor on the copy and re-enters the terminal. A zero exit hands `C0` and the edited bytes to `core::store::merge_edit`, which keeps notes parked meanwhile, and deletes the copy; a non-zero exit writes nothing and keeps the copy, which the next `e` overwrites. No lock is held while the editor runs. A scripted editor that parks through the real CLI before it saves proves the merge end to end, AS-8 runs with the TUI open, and a re-run test binary crashes the merge at `edit:after_tmp`.
- Provides: `tui::editor::Launcher` (`launch(&mut self, argv: &[String]) -> io::Result<ExitStatus>`); `tui::editor::ProcessLauncher`; `tui::editor::editor_argv(config_editor: &str, env: &Env, copy: &Path) -> Vec<String>`; `tui::editor::copy_path(env: &Env, board: &str, id: CardId) -> PathBuf`; `tui::editor::edit_card(env: &Env, board: &Board, id: CardId, config_editor: &str, launcher: &mut dyn Launcher, guard: &mut dyn TerminalGuard) -> Outcome`; field `tui::runtime::Parts.launcher: Box<dyn Launcher>`; the runtime's `Effect::Edit` arm; test helpers in `tests/common/tui.rs` (`FakeLauncher`, `scripted_editor`, `TuiFixture::set_editor`); `tests/tui_editor.rs`, `tests/tui_crash.rs`
- Requires: E4-F1 (`Runtime`, `Parts`, `TerminalGuard`, `FakeGuard`, `FakeEvents`, `tui::gate`, `tui::prepare`, `tests/common/tui.rs`), E1-F6 (`core::store::merge_edit`, `Merged`, `core::store::lock_board`, `core::store::park` for in-process parks), E1-F6-T9 (`tests/common/sound.rs` `assert_board_sound`), E1-F1 (`Env::edit_dir`, `Env.visual`, `Env.editor`, `core::failpoint`, `Env::from_vars`, `tests/common/spawn.rs` `Scenario`, `command`, `brain_swap`), E1-F1-T9 (`Scenario::vars()` in `tests/common/spawn.rs`, used by `scripted_editor`, A-E4-21), E1-F3 (`Config.editor`), E2-F2 (`brain-swap move <ID> <column>`), E2-F3 (`brain-swap park --card <ID> --session <id>` with the note on stdin, printing `parked to <ID>: <title>`)
- Feature DoD:
  - [ ] `e` copies the card, runs the editor on the copy, merges a zero-exit save and deletes the copy (E4-F5-T1).
  - [ ] The editor follows `editor`, `$VISUAL`, `$EDITOR`, `vi` and the 5.2 splitting rule (E4-F5-T2).
  - [ ] A non-zero editor exit writes nothing, keeps the copy and sets the status line; the next `e` overwrites the copy (E4-F5-T3).
  - [ ] The terminal is left before and re-entered after the editor, and no lock is held meanwhile (E4-F5-T4).
  - [ ] A note parked mid-edit survives (E4-F5-T5).
  - [ ] AS-8 with the TUI editor passes (E4-F5-T6).
  - [ ] The `edit:after_tmp` crash test passes (E4-F5-T7).

### E4-F5-T1 Edit a card

- Status: todo
- Depends on: E4-F1-T6, E1-F6-T7, E1-F1-T6
- Covers: FR-31; TECHSPEC 7.1 (Board and Detail `edit` emit `Edit`; `Edited` reloads and selects the card, no Message), 7.6 (copy `C0` to the edit folder, zero exit with changes merged through the lock, copy deleted after a successful merge), 4.8 (R6)
- Size: M
- Scope: `src/tui/editor.rs`: `Launcher`; `copy_path(env, board, id)` = `<env.edit_dir()>/<board>-<ID>.md`; `edit_card(..)`: read `C0` from `<board.path>/<ID>.md` on disk (missing: `Failed` with the text `<ID> not found`), create the edit folder, write `C0` to the copy (replacing a kept one), `guard.leave()`, `launcher.launch(argv)`, `guard.enter()`, and on a zero exit read the copy `E`, call `core::store::merge_edit(env, board, id, &C0, &E)`, delete the copy on `Ok` and return `Edited(id)`. Until E4-F5-T2 the argv is `config.editor` split on whitespace, else `vi`, plus the copy. Board and Detail `edit` with a card emit `Edit(id)`. Runtime `Effect::Edit`: `edit_card` with `parts.launcher` and `parts.guard`, then on `Edited` reload, `Reloaded` and `Done(Edited(id))` (A-E4-05); `update` on `Done(Edited(id))` selects the card and keeps the mode (A-E4-13). Test helper `FakeLauncher` (records each argv and the copy's bytes at launch, runs a test closure on the copy path, returns a given `ExitStatus` or error); `TuiFixture::runtime` gets a launcher that fails the test when called.
- Not in scope: the resolution order and splitting rule (E4-F5-T2); non-zero exits and failures (E4-F5-T3); the terminal and lock checks (E4-F5-T4); the merge rules themselves (E1-F6-T7).
- Tests first:
  1. `fr31_edit_key_emits_edit_for_selected_card`: given the `work_4_9` app with W-12 selected, when `e` is pressed on the board and, separately, in Detail on W-12, then each gives `[Edit(W-12)]`, and with no card selected `e` gives none.
  2. `ts7_6_card_bytes_are_copied_to_edit_folder`: given a runtime on a `work_4_9` copy with a `FakeLauncher` that exits 0 without changes, when `e` is dispatched on W-12, then the launcher's last argument is `<home>/.local/state/brain-swap/edit/work-W-12.md` and the copy's bytes at launch equal `W-12.md`.
  3. `fr31_saved_edit_is_written_to_card`: given a `FakeLauncher` that writes `E` (the 4.9 `W-12.md` with the title `migrate invoices to v14`) and exits 0, when `e` is dispatched, then `W-12.md` equals `E`, the app's W-12 has the new title and is selected, `status` is `None` and the mode is `Board`.
  4. `ts7_6_copy_deleted_after_successful_merge`: given test 3's run, when it returns, then `work-W-12.md` no longer exists in the edit folder.
  5. `r6_note_parked_in_process_during_edit_is_kept`: given a `FakeLauncher` whose closure parks a note on W-12 through `core::store::park` and then writes `E`, when `e` is dispatched, then `W-12.md` is `E` with that note's block at the end of its timeline.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `edit_card` never calls `lock_board`; only `merge_edit` locks (review).

### E4-F5-T2 Editor command

- Status: todo
- Depends on: E4-F5-T1, E1-F3-T1
- Covers: FR-31; TECHSPEC 5.2 (`editor`, else `$VISUAL`, else `$EDITOR`, else `vi`; split on ASCII whitespace with no quoting and no shell; the copy's path appended last)
- Size: S
- Scope: `editor_argv(config_editor, env, copy)`: the first of `config_editor`, `env.visual` and `env.editor` that is not blank after trimming (A-E4-19), else `vi`; split with `split_ascii_whitespace`; the copy's path appended. `ProcessLauncher` runs `argv[0]` with the rest as arguments through `std::process::Command` with inherited stdio, never through a shell; `edit_card` now builds its argv with `editor_argv`.
- Not in scope: exit status handling (E4-F5-T3); running a real editor (E4-F5-T5).
- Tests first:
  1. `fr31_config_editor_wins`: given `config_editor` `nvim`, `env.visual` `code` and `env.editor` `nano`, when `editor_argv` runs with copy `/s/c.md`, then it returns `["nvim", "/s/c.md"]`.
  2. `fr31_visual_used_when_config_empty`: given `config_editor` empty and `env.visual` `code --wait`, when `editor_argv` runs, then it returns `["code", "--wait", "/s/c.md"]`.
  3. `fr31_editor_variable_used_when_visual_unset`: given `config_editor` empty, no `visual` and `env.editor` `nano`, when `editor_argv` runs, then it returns `["nano", "/s/c.md"]`.
  4. `fr31_vi_when_nothing_set`: given nothing set, when `editor_argv` runs, then it returns `["vi", "/s/c.md"]`.
  5. `ts5_2_blank_values_fall_through`: given `config_editor` of two spaces and `env.visual` empty, when `editor_argv` runs with `env.editor` `nano`, then it returns `["nano", "/s/c.md"]`.
  6. `ts5_2_editor_string_split_without_quoting`: given `config_editor` `"my editor" -f` followed by a tab and `-x`, when `editor_argv` runs, then it returns `["\"my", "editor\"", "-f", "-x", "/s/c.md"]`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `ProcessLauncher` passes `argv[0]` to `Command::new` and the rest to `args`, with no `sh -c` anywhere in `src/tui/` (review and grep for `"-c"`).

### E4-F5-T3 Exit and failure rules

- Status: todo
- Depends on: E4-F5-T1
- Covers: FR-31; TECHSPEC 7.1 (`Failed` for a non-zero editor exit sets the status line), 7.6 (a non-zero exit writes nothing and keeps the copy; a kept copy is overwritten by the next `e`)
- Size: S
- Scope: In `edit_card`: a non-zero exit writes nothing, keeps the copy and returns `Failed("editor: <exit status>")`; a launch error returns `Failed("editor: <os error>")` and keeps the copy; a zero exit with `E` equal to `C0` writes nothing and deletes the copy; a `merge_edit` error returns `Failed(<its Display>)` and keeps the copy (A-E4-19).
- Not in scope: the merge's own error cases (E1-F6-T7).
- Tests first:
  1. `ts7_6_nonzero_exit_writes_nothing_and_keeps_copy`: given a `FakeLauncher` that writes `E` into the copy and exits with status 1, when `e` is dispatched on W-12, then `W-12.md` is byte for byte unchanged, the copy holds `E` and `status` is `editor: exit status: 1`.
  2. `ts7_6_kept_copy_is_overwritten_by_next_edit`: given test 1's kept copy, when `e` is dispatched again, then the copy's bytes at launch equal the current `W-12.md`, not `E`.
  3. `ts7_6_unchanged_copy_writes_nothing`: given a `FakeLauncher` that exits 0 without touching the copy, when `e` is dispatched, then `W-12.md`'s bytes and mtime are unchanged, the copy is deleted and `status` is `None`.
  4. `ts7_6_launch_error_sets_status_and_keeps_copy`: given a `FakeLauncher` that returns `io::ErrorKind::NotFound`, when `e` is dispatched, then `status` starts with `editor: `, `W-12.md` is unchanged and the copy exists.
  5. `ts7_6_failed_merge_keeps_copy`: given a `FakeLauncher` that parks a note on W-12 through `core::store::park` and writes an `E` holding a `<<<<<<<` line, when `e` is dispatched, then `status` is the `Unreadable` error's text, `W-12.md` equals the file with the parked note, and the copy holds `E`.
  6. `ts7_6_missing_card_sets_status_without_launch`: given `W-12.md` deleted after the board was loaded, when `e` is dispatched on W-12, then `status` is `W-12 not found` and the launcher was not called.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] Every path out of `edit_card` either deletes the copy after a successful merge or leaves it in place, never half-written (review).

### E4-F5-T4 Terminal and lock around the editor

- Status: todo
- Depends on: E4-F5-T1
- Covers: FR-31; TECHSPEC 7.5 (raw mode and the alternate screen restored around the editor), 10 (the lock is held for one read-modify-write, never across an editor)
- Size: S
- Scope: `edit_card` calls `guard.leave()` before the launch and `guard.enter()` after it on every path; the runtime clears the terminal after an edit so the next frame repaints fully. A test shares one log between `FakeGuard` and `FakeLauncher`. `Edited` from Detail keeps the Detail mode with `note` clamped (A-E4-13).
- Not in scope: the guard itself (E4-F1-T6); the lock's own rules (E1-F6-T1).
- Tests first:
  1. `ts7_5_terminal_left_before_editor_and_reentered_after`: given a shared log, when `e` is dispatched with a launcher that exits 0, then the log's last three entries are `leave`, `launch`, `enter`.
  2. `ts7_5_terminal_reentered_after_failed_editor`: given a launcher that exits 1 and, separately, one that fails to launch, when `e` is dispatched, then each log ends with `leave` followed by `enter` (with `launch` between them for the first).
  3. `ts10_no_board_lock_held_while_editor_runs`: given a launcher whose closure calls `core::store::lock_board(&env, &board_dir)`, when `e` is dispatched, then the lock is acquired within 100 ms and released before the closure returns.
  4. `fr31_edit_from_detail_returns_to_detail`: given Detail on W-12 at `note 2/2` and a launcher that writes an `E` without the older note, when `e` is dispatched, then the mode is `Detail { card: W-12, note: 0, .. }`.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `Terminal::clear` follows `guard.enter()` in the Edit arm (review).

### E4-F5-T5 Scripted editor: a note parked mid-edit survives

- Status: todo
- Depends on: E4-F5-T2, E4-F5-T4, E2-F3-T1, E2-F2-T6
- Covers: FR-31; TECHSPEC 4.8 (R6), 7.6, 10 (only the TUI's `e` merges), 12.1 (runtime test with a scripted editor that parks on the card before exiting, started with `env -i` and the scenario's variables, passing `--session test`)
- Size: M
- Scope: `tests/common/tui.rs` `scripted_editor(scenario: &Scenario, dir: &Path, steps: &[&str], edited: &Path, code: i32) -> String`: writes `<dir>/scripted_editor.sh` (each step a shell line, `@BS@` replaced by `env!("CARGO_BIN_EXE_brain-swap")`, then `cat <edited> > "$1"` and `exit <code>`) and returns the config `editor` value `/usr/bin/env -i <the scenario's variables as NAME=value> /bin/sh <dir>/scripted_editor.sh` (A-E4-21). `TuiFixture::set_editor(&str)` writes the config's `editor` line and re-prepares. The runtime uses `ProcessLauncher` and `FakeGuard`. Script and edited files live in the test's temp dir.
- Not in scope: parallel parks and the rest of AS-8 (E4-F5-T6); the crash (E4-F5-T7); the CLI's own park tests (E2-F3).
- Tests first:
  1. `fr31_note_parked_mid_edit_survives`: given the scripted editor with the step `@BS@ park --card W-12 --session test` fed `Doing: parked mid-edit`, `Next: check the merge` and `Watch out: nothing` on a quoted heredoc, and `edited` = the 4.9 `W-12.md` titled `migrate invoices to v14`, when `e` is dispatched on W-12, then `W-12.md` holds the v14 title and three notes with the parked one last, every byte outside that note's block equals `edited`, the copy is deleted, and the app shows W-12 selected with three notes.
  2. `r6_file_column_change_mid_edit_is_kept`: given the step `@BS@ move W-12 Done` and an `edited` with a changed body and the column untouched, when `e` is dispatched, then `W-12.md` has `column: Done` and the body change.
  3. `r6_user_column_change_wins_over_file`: given the step `@BS@ move W-12 Done` and an `edited` with `column: Todo`, when `e` is dispatched, then `W-12.md` has `column: Todo`.
  4. `ts12_1_scripted_editor_is_hermetic`: given the step `env > <dir>/env.txt`, when `e` is dispatched, then `env.txt` names exactly the scenario's variables and no `HERDR_` or `CLAUDE_` variable.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] No file of the scripted editor is written inside the repository (review).

### E4-F5-T6 AS-8 with the TUI

- Status: todo
- Depends on: E4-F5-T5
- Covers: FR-31; TECHSPEC 7.4 (both parks show after one poll), 7.6; AS-8 (TUI half)
- Size: S
- Scope: AS-8 with the TUI open: parallel `brain-swap park` processes started through `spawn::brain_swap(&scenario)` with the same `BRAIN_SWAP_NOW`, and a scripted editor whose step starts two parks in the background and waits for them before saving. W-12 and W-11 of `work_4_9` stand in for AS-8's W-12 and W-7.
- Not in scope: the CLI half of AS-8 (E2-F3-T12); the store's thread variant (E1-F6-T10).
- Tests first:
  1. `as8_parks_on_two_cards_in_one_second_show_after_one_tick`: given a runtime on a `work_4_9` copy with W-12 selected, when parks on W-12 (`--session a`) and W-11 (`--session b`) run in parallel, then both exit 0 printing `parked to W-12: migrate invoices to v13` and `parked to W-11: invoice PDF shows the wrong VAT`, and after one `tick` W-12 shows 3 notes, W-11 shows 1, each file differs from before only by its inserted block, `W-9.md` and `board.md` are unchanged and W-12 is still selected.
  2. `as8_two_parks_on_one_card_both_show`: given the same setup, when two parks on W-12 run in parallel, then after one `tick` W-12 shows 4 notes and both new blocks are intact.
  3. `as8_parks_while_tui_editor_open_survive_save`: given a scripted editor whose step runs one park on W-12 and one on W-11 in the background and `wait`s, and an `edited` W-12 with a new title, when `e` is dispatched on W-12 and then `tick` runs, then `W-12.md` is `edited` plus its new note, `W-11.md` has its new note, and the app shows both.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] The tests passed 10 consecutive local runs, stated in the commit message.

### E4-F5-T7 Process-level crash at edit:after_tmp

- Status: todo
- Depends on: E4-F5-T5, E1-F6-T9, E4-F1-T8
- Covers: FR-31; TECHSPEC 3 (I1 to I4), 4.6 (crash safety), 6.1 (`BRAIN_SWAP_FAILPOINT` read into `Env`), 12.1 (crash, process level, `edit:after_tmp` through the editor merge)
- Size: M
- Scope: `tests/tui_crash.rs` with `#![cfg(feature = "failpoints")]`. The child test `tui_crash_child` is `#[ignore]` and returns at once unless `BRAIN_SWAP_FAILPOINT` is set; it builds `Env` from its own process environment as `main` does (`Env::from_vars`), runs `tui::gate(&env, None, true)` and `tui::prepare`, and drives a `Runtime` with `FakeGuard`, `FakeEvents` `[e, q]` and `ProcessLauncher`; the config's editor is a scripted editor, built from the scenario without `BRAIN_SWAP_FAILPOINT`, that parks on W-12 and saves a new title. The parent spawns `std::env::current_exe()` through `spawn::command` with `--ignored --exact tui_crash_child --test-threads=1`, the scenario's variables and `BRAIN_SWAP_FAILPOINT=edit:after_tmp` (A-E4-20), then checks the board with `assert_board_sound`.
- Not in scope: the in-process `edit:after_tmp` test (E1-F6-T9); `create:*` and `park:*` through the CLI (E2-F3-T11).
- Tests first:
  1. `ts12_1_edit_after_tmp_crash_leaves_card_sound`: given the child spawned with `BRAIN_SWAP_FAILPOINT=edit:after_tmp`, when it ends, then its exit is not success, `W-12.md` equals the 4.9 bytes plus the parked note and parses, `assert_board_sound` holds, a `.W-12.md.bs-tmp-<pid>` file holds the merged bytes and is not loaded as a card, and the copy in the edit folder still exists.
  2. `ts12_1_edit_after_crash_succeeds_without_failpoint`: given test 1's leftovers, when an in-process runtime dispatches `e` on W-12 with a `FakeLauncher` that saves a new title, then `W-12.md` holds the title and the parked note and the leftover temp is still ignored.
- DoD:
  - [ ] The tests above pass; the test for the first item was committed red before the implementation turned it green (12.2 item 1).
  - [ ] `cargo test` without features compiles `tests/tui_crash.rs` to no tests, and `cargo test -- --ignored` without the variable runs `tui_crash_child` as a no-op.

## Assumptions

- A-E4-02 Does `App` carry, beyond the 7.1 fields, `warnings` (start-up warnings for Help), `templates` (name and final hotkey) and `boards` (config order), with `Selection { column, card: Option<CardId> }` indexing `Board::column_list()`? Affects E4-F1-T1, E4-F4-T1, E4-F4-T4.
- A-E4-03 Are `Effect::Jump`, `Outcome::Jumped`, `Outcome::NotJumped` and `Outcome::FocusFailed`, the Board and Detail `jump` arms of `update` and the runtime's jump arm all added by E5-F3 (they need `adapters::herdr::PaneStatus`), while E4-F1 defines `Mode::Message` with its any-key return and its screen? Affects E4-F1-T1, E4-F1-T4.
- A-E4-04 In ConfigError does every key emit `Quit` and the runtime return exit 1 when it quits from ConfigError (0 otherwise), and does an I/O error from the terminal end the TUI with exit 1 after restoring it? Affects E4-F1-T5, E4-F1-T6.
- A-E4-05 After its own `Move`, `Create` and `Edit` does the runtime reload the board and dispatch `Reloaded` before `Done`, and is a `Reloaded` whose board name differs from the current one a board switch that resets the selection to the A-27 rule? Affects E4-F1-T7, E4-F2-T5, E4-F4-T3, E4-F4-T4, E4-F5-T1.
- A-E4-06 Do `left` and `right` select the card at the same row of the target column, clamped to its last card, with no per-column memory, unfocused columns shown from their top and the focused column scrolled just enough to show its selection? Affects E4-F2-T2.
- A-E4-07 After a reload, does the focus follow the selected card to its new column, and does a vanished card give the card at its old row in the same column (clamped), or no card in an empty column? Affects E4-F1-T7.
- A-E4-08 Does the status line show at start the first-run notice, else the first warning in the order config loader, `BRAIN_SWAP_LOG` guard, key config, templates, board load, and is it cleared by the next key press? Affects E4-F1-T3, E4-F1-T8, E4-F4-T1.
- A-E4-09 Board layout: a one-character left margin on every row, equal column widths each ending in a two-character gap, a minimum column width of 20 with a window of columns centred on the focused one when they do not fit, two rows per card with no blank row between cards, text cut at the column width without an ellipsis, and the header path with `HOME` shown as `~` (the panel's cwd as recorded)? Affects E4-F2-T1, E4-F2-T2, E4-F2-T4.
- A-E4-10 Does a card whose latest note has an empty Next and Doing show only the age, does the panel of a card without notes read `<ID> no note yet`, does a focused empty column leave only the panel's separator, and is age `now` printed without `ago`? Affects E4-F2-T1, E4-F2-T4.
- A-E4-11 Detail formats: header `<ID> <title> (<column>, <board>)`; note line `note <n>/<N>, <stamp>, <age> ago[, auto]` with a bad stamp shown as the heading text after `### `; parts with their continuation lines, wrapped; `Where: <cwd> (pane <pane>)`, `Where: <cwd>` or `Where: none`; `no note yet` for a card without notes; the body cut, not wrapped? Affects E4-F3-T1, E4-F3-T3, E4-F3-T4.
- A-E4-12 Do `scroll_down` and `scroll_up` move the body by 10 lines, clamped to the first and the last body line, and does changing the note keep the scroll? Affects E4-F3-T3.
- A-E4-13 In Detail, does a reload keep the shown note index (clamped) and return to Board when the card is gone, and does `Edited` keep the Detail mode with the index clamped? Affects E4-F3-T4, E4-F5-T1, E4-F5-T4.
- A-E4-14 Does Help list the 17 actions of 5.2 in 5.2 order (template hotkeys appear in the template picker, not in Help), then the warnings, cut at the screen height without scrolling, since any key leaves Help? Affects E4-F1-T4.
- A-E4-15 Do the Message and ConfigError screens show only their text, with no hint line? Affects E4-F1-T4, E4-F1-T5.
- A-E4-16 When an extra (unknown) column is focused, does a card created in the TUI enter the first configured column, since no write targets an extra column (4.7)? Affects E4-F4-T2.
- A-E4-17 In the pickers, is the selected row marked `>` as well as drawn in reverse video, does BoardPick start on the current board and TemplatePick on the first template, and are the titles `new card in <column>: pick a template`, `new <template> card in <column>` with `title: <buf>`, and `switch board`? Affects E4-F4-T1, E4-F4-T2, E4-F4-T4.
- A-E4-18 Does a board folder that cannot be listed at start make the TUI exit 1 with `brain-swap: <message>` before the alternate screen, while on a poll or a board switch the status line shows the error and the current board stays? Affects E4-F1-T7, E4-F1-T8, E4-F4-T4.
- A-E4-19 Editor: do blank `editor`, `$VISUAL` and `$EDITOR` values count as unset, are the status texts `editor: <exit status>` and `editor: <os error>`, does a zero exit without changes count as a successful merge (copy deleted), and do a failed launch and a failed merge keep the copy? Affects E4-F5-T2, E4-F5-T3.
- A-E4-20 Is re-running the test binary through the spawn helper, with an ignored child test that builds `Env` from its environment as `main` does and drives the runtime with the fake terminal guard, an acceptable process-level layer for `edit:after_tmp`, since the TUI exits 2 without a terminal (6.3) and section 13 has no pty crate? Affects E4-F5-T7.
- A-E4-21 Is the scripted editor configured as `editor = "/usr/bin/env -i <scenario variables> /bin/sh <script>"`? Affects E4-F5-T5.
- A-E4-23 Does the fingerprint cover only `board.md` and the files matching the board letter's card pattern, so other files and dot-prefixed temp files never trigger a reload? Affects E4-F1-T7.
- A-E4-24 Is NFR-02's first frame measured in process, from `tui::prepare` to the first draw on a 120x40 `TestBackend`, by an `#[ignore]` release test, leaving process start-up to E6-F1? Affects E4-F1-T10.
- A-E4-25 Does the TUI path call `log::guard_board_folders` itself (as E1-F1-T8 and A-E1-09 state; the CLI calls it in `cli::cmd::Ctx::load`, E2-F1-T4) and show its warning in Help and the status line? Affects E4-F1-T8.
- A-E4-26 Are key release events ignored, and is `KeyCode::Char(' ')` the `space` key of 5.3? Affects E4-F1-T2.
- A-E4-27 Does the bare-invocation gate belong to E4-F1 as `tui::gate`, checking in this order: config (an error is carried to ConfigError), unknown `--board` (exit 3), then stdin and stdout both terminals (checked by `main.rs` and passed as `terminal`), else exit 2 with `Error::Usage` `not a terminal`? Affects E4-F1-T8.
