<p align="center">
  <img src="docs/_.svg" alt="Yubi Logo" width="140" />
</p>

<h1 align="center">Yubi</h1>

<p align="center">
  <strong>Biomechanical Touch-Typing Trainer & Sensory Biofeedback Engine for Terminals</strong>
</p>

<p align="center">
  Built with 100% Rust and Ratatui. Pure local execution, zero web dependencies, sub-millisecond in-memory rendering.
</p>

---

## 📺 Overview

Yubi is an interactive terminal typing trainer designed to accelerate touch-typing velocity, eliminate cognitive hesitations, and balance finger workloads through real-time sensory biofeedback and native in-terminal diagnostics.

```
+----------------------------------------------------------------------------------------------------------------------+
| Yubi — Progress Dashboard                                                                                            |
| Sessions: 42 • Avg: 68.2 WPM • Peak: 94.1 WPM • Acc: 96.4% • Strokes: 12,450 (320 errs) • Streak (≥90%): 8          |
| Filters [f]: Time: All Time [t] | Level: All [l] | Mode: All [m]   (Showing 42 of 42 sessions)                       |
+----------------------------------------------------------------------------------------------------------------------+
| [1] Trends & Progress  |  [2] Keyboard Heatmap  |  [3] Finger Load  |  [4] Level Comparison  |  [5] Session History  |
+----------------------------------------------------------------------------------------------------------------------+
```

---

## 🚀 Key Features

### 1. Biomechanical Speed Acceleration
- **Ghost Pacer & Real-time Delta**: Visual velocity cursor gliding at target or personal-best WPM, accompanied by an instant dynamic pace badge (`▲ +6.4 WPM` / `▼ -3.2 WPM`).
- **Burst / Sprint Engine**: Timed countdown rounds (15s, 30s, 60s) with infinite streaming word replenishment, plus 5-word micro-bursts to push neuromuscular speed ceilings.
- **Auto-Stepping Burst Ladder**: Automatically steps target pacing by `+5 WPM` when completing rounds with $\ge 96\%$ accuracy.
- **Inter-Key Interval (IKI) Tracking**: Measures millisecond latency between consecutive keystrokes ($\text{IKI}_n = t_n - t_{n-1}$) and classifies transitions (Alternating Hands, Same-Hand Rolls, Same-Finger Leaps, and Weak-Finger Hesitations).

### 2. Live Sensory Biofeedback
- **Live On-Screen Keyboard**: Real-time virtual layout with active key highlighting, opposite-hand Shift chording indicators, and tactile home-row anchor points (`F` and `J`).
- **Dynamic Thermal Error Glow**: Keys heat up upon mistakes ($H_k \leftarrow \min(1.0, H_k + 0.4)$) and decay gradually ($\lambda = 0.15\text{ s}^{-1}$) as clean typing resumes.
- **Cadence & Rhythm Metronome**: Live consistency gauge calculating rolling coefficient of variation ($CV$) to guide typists into sustained flow-state rhythm (`Flow State`, `Staccato`, `Erratic`).
- **Instant Replay Debrief**: Immediate post-round scorecard with Braille velocity curves, discrete error markers, top mistake substitution pairs, and slowest bigram hesitation metrics.

### 3. Native 5-Tab TUI Dashboard
- **[1] Trends & Progression**: Dual Braille continuous curves tracking WPM and accuracy over time, with dynamic target benchmarks.
- **[2] Keyboard Heatmap & Inspector**: Interactive QWERTY matrix with three visualization modes (`Finger Accuracy`, `Key Errors`, `Keystroke Load`) and an arrow-key-navigated Key Inspector.
- **[3] Finger Load & Biomechanical Balance**: Comparative horizontal progress bars analyzing Workload Share %, Latency Index, and Accuracy Precision % across all 10 fingers, accompanied by Left vs. Right hand symmetry metrics and ergonomic diagnostics.
- **[4] Level & Mode Comparison**: Comparative performance matrices segmented across difficulty levels (Beginner, Intermediate, Advanced) and training modes.
- **[5] Session History Table**: Scrollable chronological data log with date, duration, speed, accuracy, stroke/error counts, average IKI, and rhythm consistency.
- **In-TUI Slicing & Filter Engine**: Seamlessly filter all analytics by Time (`All`, `Last 7 Days`, `Last 30 Days`), Level, and Mode.

### 4. Advanced Training Content Banks
- **Rust Programming Syntax Practice**: Dedicated programming practice domain featuring 8 core syntactic topics across 3 graduated difficulty levels (Beginner, Intermediate, Advanced):
  1. *Ownership, Borrowing & Lifetimes*: References (`&T`, `&mut T`), raw pointers (`*ptr`), explicit lifetimes (`'a`, `'static`, `struct Buffer<'a>`), and `move` closures.
  2. *Smart Pointers & Interior Mutability*: `Box::new()`, `Rc::clone(&item)`, thread-safe `Arc::new()`, `RefCell`, `Mutex`, `RwLock`, `Weak<T>`, and `Cow<'a, str>`.
  3. *Data Structures & Collections*: `Vec::with_capacity()`, `HashMap::entry().or_insert()`, `BTreeMap`, `HashSet` intersections, `VecDeque`, turbofish syntax (`::<T>`), and custom structs/enums.
  4. *Pattern Matching & Control Flow*: Exhaustive match blocks with guards (`Some(x) if x > 10 =>`), concise `if let Some(val) = opt`, `let ... else` early returns, and slice destructuring.
  5. *Error Handling (Result & Option)*: The `?` propagation operator, chained combinators (`.map()`, `.and_then()`, `.unwrap_or_else()`, `.ok_or_else()`), and custom error formatting.
  6. *Traits, Generics & Bounds*: Generic bounds (`<T: Clone + Debug>`), complex `where` clauses, dynamic dispatch (`Box<dyn Trait>` vs `impl Trait`), and derive attributes (`#[derive(Debug, Clone, PartialEq)]`).
  7. *Iterators & Closures*: Pipe syntax (`|x| x * 2`, `move |a, b|`), iterator chains (`.iter().filter().map().collect::<Vec<_>>()`), and adapters (`.enumerate()`, `.zip()`, `.fold()`, `.flatten()`).
  8. *Async, Concurrency & Macros*: `async fn` / `.await`, `tokio::spawn`, thread channels (`mpsc::channel()`, `tx.send()`, `rx.recv()`), and declarative macros (`println!`, `vec!`, `macro_rules!`).
  - *Generous Syntax Concatenation*: Full code statements are concatenated generously up to your chosen word count without severing statements mid-token.
  - *All Topics Mixed*: Drill across all 8 topics concurrently for comprehensive Rust motor fluency.
- **Bigram Hesitation Ladder**: Automatically identifies the user's slowest transitions and generates 3-tier progressive drills:
  - *Tier 1*: Isolation Repeats (`qu qu qui que squ qua qu`)
  - *Tier 2*: Targeted Words (`quick queen equal square liquid`)
  - *Tier 3*: Context Sentences (`The quick queen required quiet squares.`)
- **Developer Syntax & Code Mode**: Curated code banks across TypeScript, Python, and synthetic symbol sprints (`{}`, `[]`, `<>`, `->`, `=>`, `::`, `!=`, `&&`, `||`).

### 5. Ergonomics & Customization
- **Multi-Layout Support**: Native key attribution for **QWERTY**, **Colemak**, **Colemak-DH**, and **Dvorak**.
- **Thumb Preference**: Spacebar thumb selection for **Right Thumb**, **Left Thumb**, or **Auto-Alternating** based on hand opposition.
- **Color Themes**: Centralized theme palette switchable at runtime with `<F2>`:
  - *Catppuccin Mocha*
  - *Tokyo Night*
  - *Nord*
  - *Gruvbox Dark*
  - *Yubi Classic*
- **Configurable Paragraph Length**: Choose practice lengths up to 60 words (`Burst 5w`, `Quick 10w`, `Short 15w`, `Standard 20w`, `Medium 30w`, `Extended 40w`, `Long 50w`, `Full 60w`), backed by rich multi-sentence paragraph banks with automatic chaining and punctuation integrity.
- **Editor Ergonomics**: Quick word deletion with `Ctrl+W` / `Ctrl+Backspace` and instant round retry with `Ctrl+R` (or `r` in debrief).

---

## ⌨️ Keybindings Reference

### Setup Screen
| Key | Action |
| :--- | :--- |
| `↑` / `↓` / `Tab` / `Shift+Tab` | Switch settings field (Domain, Topic/Mode, Level, Length, Layout, Thumb, Pacer) |
| `←` / `→` | Cycle option values |
| `Enter` | Launch typing round |
| `F2` | Cycle color theme |
| `Esc` | Quit Yubi |

### Practice Round (`Mode::Typing`)
| Key | Action |
| :--- | :--- |
| Any Printable Character | Type expected character |
| `Backspace` | Correct last mistake |
| `Ctrl+W` / `Ctrl+Backspace` | Fast delete previous word |
| `Ctrl+R` | Instant round restart with new text |
| `F2` | Cycle color theme |
| `Esc` | Return to Setup / Cancel round |

### Instant Replay Debrief (`Mode::Debrief`)
| Key | Action |
| :--- | :--- |
| `r` | Retry same text immediately (reinforce motor memory) |
| `w` | Launch targeted round drilling weakest finger |
| `Enter` | Next round with same settings |
| `Tab` | Open native TUI Dashboard |
| `Esc` | Return to Setup screen |

### Performance Dashboard (`Mode::Dashboard`)
| Key | Action |
| :--- | :--- |
| `1` – `5` | Switch directly to Tab 1–5 |
| `Tab` / `Shift+Tab` | Cycle next / previous dashboard tab |
| `t` / `T` | Cycle Time Filter (`All Time` $\to$ `Last 7 Days` $\to$ `Last 30 Days`) |
| `l` / `L` | Cycle Level Filter (`All` $\to$ `Beginner` $\to$ `Intermediate` $\to$ `Advanced`) |
| `m` / `M` | Cycle Mode Filter (`All` $\to$ `Targeted` $\to$ `Random`) |
| `h` / `H` | Cycle Heatmap Mode (*Tab 2 only*: `Accuracy` $\to$ `Errors` $\to$ `Load`) |
| `↑` / `↓` / `←` / `→` | Navigate keys on virtual keyboard (*Tab 2*) |
| `↑` / `↓` / `PgUp` / `PgDn` | Scroll session log rows and pages (*Tab 5*) |
| `Home` / `End` | Jump to first / last session (*Tab 5*) |
| `r` | Return to Setup screen |
| `Esc` / `q` | Quit Yubi |

---

## 📦 Installation & Building

### Prerequisites
- A working Rust toolchain (Rust 1.75+ recommended).

### single run settup

```bash
curl -fsSL https://raw.githubusercontent.com/temesgensida-code/yubi/main/install.sh | bash
```
 
### Building from Source
```bash
git clone https://github.com/temesgensida/yubi.git
cd yubi
cargo build --release
```

Run directly:
```bash
cargo run --release
```

Install binary locally:
```bash
cargo install --path .
```

---

## 🧪 Testing & Verification

Run the full automated test suite:
```bash
cargo test
```

Check all targets and documentation:
```bash
cargo check --all-targets
```

---

## 📄 License
MIT License. Open source and free for all typists and developers.
