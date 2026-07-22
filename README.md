# FingerTrack

A terminal typing trainer that tracks *which physical finger* is making
mistakes, optionally saves progress across sessions as local JSON (with
CSV export), and visualizes per-finger accuracy/WPM trends and mistake
patterns with ratatui charts.

## Architecture

```
src/
├── main.rs        Terminal setup/teardown + non-blocking input/mouse event loop
├── app.rs           App state machine: Consent -> Setup -> Typing -> Dashboard
│                     Level, TrainingMode, RoundLength, DashboardTab enums live here
├── finger.rs         Finger enum, QWERTY -> finger keymap, TypingEngine
│                     (stats + mistake matrix + "most faulty finger")
├── generator.rs      Weak-finger-targeted word drills (Beginner level)
├── paragraphs.rs      Finger-tagged paragraph bank (Intermediate/Advanced)
├── persistence.rs     SessionSnapshot / UserHistory + AppSettings; JSON
│                       load/save + CSV export
├── theme.rs           Centralized color palette + shared Block helpers
└── ui.rs              Pure ratatui rendering (consent/setup/typing/dashboard views)
```

## Flow

1. **Consent screen** (first launch only): choose whether FingerTrack may
   save your session history to disk. This is asked once — your answer is
   remembered in `settings.json`, and every later launch skips straight to
   Setup. Rounds work either way; if you decline, nothing is written to
   disk and CSV export stays unavailable until you opt in.
2. **Setup screen**: pick a **difficulty level**, a **training mode**, and a
   **round length**, then press `Enter`. All three choices are saved to
   disk as you change them, so your last-used settings are what you see
   next time you launch FingerTrack.
3. **Typing round**: practice text appears **centered on screen** (not
   pinned to the top), with a progress gauge, a live WPM sparkline, and a
   "next finger" hint above the text.
4. **Dashboard**: appears automatically when a round finishes. Sidebar on
   the left lists all 10 fingers, worst-accuracy-first, each with an inline
   mini-sparkline of its recent trend — click a row (or use `↑`/`↓`) to
   select a finger. The right pane has three tabs: Accuracy Trend, WPM
   Trend, and Mistake Matrix. Press `e` to export saved history to CSV.
5. Press `r` from the dashboard to go back to Setup and start another round
   (same or different level/training mode/round length).

## Levels

| Level        | Source                     | Text style |
|--------------|------------------------------|-------|
| Beginner     | `generator.rs` word drills   | lowercase words, punctuation stripped |
| Intermediate | `paragraphs.rs` bank         | original casing/punctuation |
| Advanced     | `paragraphs.rs` bank         | original casing/punctuation |

## Round length

Independent of `Level`, **Round Length** controls how much text a round
contains: Short (8 words), Medium (16 words), or Long (28 words). It's a
separate Setup field, so e.g. "Advanced" + "Short" gives a brief but
punctuation-intact drill, while "Beginner" + "Long" gives a longer run of
simple lowercase words.

## Training modes

- **Target weakest finger** — looks up whichever finger currently has the
  highest error rate (`TypingEngine::get_most_faulty_finger`) and picks
  word/paragraph content loaded with that finger's letters.
- **Random** — ignores finger weighting entirely; every word/paragraph
  choice is uniform-random.

Both are selectable on the Setup screen and apply to whichever level and
round length are also selected.

## Controls

**Consent screen** (first launch only)
- `y` — save progress to disk (enables history + CSV export).
- `n` — don't save anything to disk.
- `Esc` — quit.

**Setup screen**
- `↑`/`↓` or `Tab` — switch focus between Difficulty, Training Mode, and
  Round Length.
- `←`/`→` — change the focused field's value.
- `Enter` — start the round.
- `Esc` — quit.

**Typing mode**
- Type the centered text (green = correct, red-underline = wrong — briefly
  flashes bright red the instant it happens, then settles to a dimmer red;
  the current word is brighter than not-yet-reached words so your eye
  tracks progress naturally).
- `Backspace` — correct the last character visually (the mistake still
  stays in the analytics — see the doc comment on `handle_backspace`).
- `Esc` — quit.

**Dashboard**
- `↑`/`↓` or **click a row** — select a finger in the sidebar (sorted
  worst-first).
- `Tab` — cycle Accuracy Trend / WPM Trend / Mistake Matrix.
- `e` — export saved history to CSV (only if progress tracking is on and
  at least one session has been recorded).
- `r` — back to Setup to configure and start a new round.
- `Esc`/`q` — quit.

## Running

```bash
cargo run
```

History is stored at your OS config directory, e.g.
`~/.config/fingertrack/history.json` on Linux, falling back to
`./history.json` if that directory can't be resolved. Settings (level,
training mode, round length, and your tracking-consent answer) live next
to it as `settings.json`, and a CSV export lands at
`history_export.csv` in the same directory.

## UI details worth knowing about

- **Centralized theme** (`theme.rs`): every color used in `ui.rs` goes
  through `THEME` constants and the `theme::block()`/`theme::block_focused()`
  helpers (rounded borders everywhere). Restyle the whole app by editing
  one file.
- **Color-blind-safe cues**: wrong characters get an underline modifier in
  addition to red, and weak fingers in the sidebar get a `⚠` marker in
  addition to a color change — none of the signal is color-only.
- **Small-terminal guard**: below 70x20 cells, the whole UI is replaced
  with a "resize your terminal" message instead of rendering a broken
  layout (checked in `ui::draw` before anything else).
- **Mouse capture is on**, solely to support click-to-select in the
  Dashboard's finger sidebar (`App::handle_sidebar_click`); every other
  mode ignores mouse events. This does mean normal terminal text selection
  is unavailable while FingerTrack is running.

## Deliberate simplifications (documented in code, worth revisiting)

- **Shift key is not modeled as its own keystroke.** Uppercase letters and
  shifted symbols are attributed to the same finger as their unshifted
  key, not to the opposite pinky that would press Shift in real touch
  typing. See the doc comment on `build_keymap`.
- **Backspace doesn't retroactively remove a mistake from the stats** —
  the wrong keystroke genuinely happened, so it stays in the analytics;
  only the visible input buffer rewinds.
- **`LeftThumb` never accrues keystrokes** — the space bar is attributed
  entirely to `RightThumb`, so `LeftThumb`'s accuracy always reads 100%.
  Its paragraph-bank entry exists for completeness but won't currently
  affect its stats. See the comment at the top of `paragraphs.rs`.
- **Paragraph bank is a small built-in list** (`paragraphs.rs`, 2 entries
  per finger) rather than a large curated corpus — the letters are
  "loaded" toward each finger, not exclusive (English doesn't allow that).
- **Word bank in `generator.rs`** is likewise a small built-in list.
- **Intermediate and Advanced now shape text identically** (casing/
  punctuation intact); the only thing that used to distinguish them —
  paragraph length — is now controlled by the independent Round Length
  setting instead. They're kept as separate `Level` values for future
  divergence (e.g. sentence complexity, vocabulary tier).

## Extending this

- Grow the paragraph bank, or load it from an external file instead of a
  compiled-in `&[(Finger, &str)]` slice.
- Let `Level` diverge from `RoundLength` in more than styling — e.g. an
  Advanced-only vocabulary tier or punctuation density.
- CSV export currently overwrites a single `history_export.csv`; a
  timestamped-file or append-only mode would suit long-running use better.
