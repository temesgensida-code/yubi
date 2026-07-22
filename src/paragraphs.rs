//! Finger-tagged paragraph bank.
//!
//! Each entry is a `(Finger, text)` pair where the text was written to lean
//! heavily on the letters that finger types (see `Finger`'s keymap in
//! `finger.rs`). English doesn't allow a perfectly exclusive set of
//! letters-per-finger, so treat "loaded" as "noticeably weighted toward",
//! not "only contains".
//!
//! Note: `LeftThumb` never actually receives keystrokes in this engine -
//! the space bar is attributed entirely to `RightThumb` (see the comment
//! on `TypingEngine`'s keymap). Its paragraph entry below is still provided
//! for completeness/future use, but in practice `LeftThumb` accuracy will
//! always read 100% since it has zero recorded keystrokes.

use crate::app::Level;
use crate::finger::Finger;
use rand::seq::SliceRandom;

const PARAGRAPHS: &[(Finger, &str)] = &[
    (
        Finger::LeftPinky,
        "Zany quokkas quietly amaze quiet zebras near azure lagoons, as quirky wizards analyze quaint zigzag mazes.",
    ),
    (
        Finger::LeftPinky,
        "A quiet zealot quickly amazed a zoo with an azure quilt, adoring every quirky zigzag along the quay.",
    ),
    (
        Finger::LeftRing,
        "Wise walruses swiftly waxed six wooden saws, watching sixty swans swim past waxwork statues on the sand.",
    ),
    (
        Finger::LeftRing,
        "Six swift wasps waxed a wide wooden wall, swirling sideways with a strange, restless swagger all week.",
    ),
    (
        Finger::LeftMiddle,
        "Dedicated ducks decided each decade to educate deceived crickets, creating deep, calm, decent codes.",
    ),
    (
        Finger::LeftMiddle,
        "Educated coders decided each deed deserved a decisive, dedicated defense against deceptive, careless code.",
    ),
    (
        Finger::LeftIndex,
        "Brave frogs bravely gathered fresh berries, forgetting the great, foggy river beyond the vibrant garden.",
    ),
    (
        Finger::LeftIndex,
        "Every great forger brought a firm, vibrant brief, gathering brave friends before the growing thunderstorm.",
    ),
    (
        Finger::LeftThumb,
        "Go on and do it, one bit at a bit, and do not stop to sit; it is not that hard to go on and do it.",
    ),
    (
        Finger::RightThumb,
        "Go do it now, go to bed by ten or so, if it is ok to you my dear old pal, we can go at dawn.",
    ),
    (
        Finger::RightThumb,
        "It is up to us to go on and do it now, no matter how hot or dry it is out there today.",
    ),
    (
        Finger::RightIndex,
        "Many hungry monkeys jumped joyfully near humming jungles, munching yummy honey under the sunny noon sky.",
    ),
    (
        Finger::RightIndex,
        "Yesterday my human neighbor joyfully hummed a funny jingle, running through muddy meadows until noon.",
    ),
    (
        Finger::RightMiddle,
        "I think, I know, I like quiet nights, watching kind kids skip, kick, and giggle, thinking of nice tricks.",
    ),
    (
        Finger::RightMiddle,
        "I picked six kites, thinking I might link nice kids, liking quick tricks, winking, and skipping quietly.",
    ),
    (
        Finger::RightRing,
        "Lonely owls slowly circled cold, golden fields below, following old moonlit trolls along a long, lonely road.",
    ),
    (
        Finger::RightRing,
        "Old sailors slowly followed golden clouds below a cold, lonely moon over a long, hollow valley.",
    ),
    (
        Finger::RightPinky,
        "Perhaps a puppy quickly hopped upon a purple pipe; papa perhaps hoped to keep the puppy happy.",
    ),
    (
        Finger::RightPinky,
        "Perhaps upon a proper path, a puppy popped up; people happily hoped to help the puppy up.",
    ),
];

/// Applies difficulty-specific shaping to a paragraph and returns it as a
/// char vector ready to drop straight into `App::target`.
///
/// - `Beginner`: lowercase, punctuation stripped - closer to a warm-up
///   drill than a real paragraph.
/// - `Intermediate`/`Advanced`: original casing/punctuation kept.
///
/// In every case the result is capped to `word_count` words (from the
/// `RoundLength` setting), so length is chosen independently of `level`.
/// If `word_count` is at least as long as the source paragraph, the whole
/// thing is used as-is.
pub fn shape_for_level(text: &str, level: Level, word_count: usize) -> Vec<char> {
    match level {
        Level::Beginner => {
            let cleaned: String = text
                .chars()
                .map(|c| if c.is_alphabetic() || c.is_whitespace() { c } else { ' ' })
                .collect();
            let words: Vec<&str> = cleaned.split_whitespace().take(word_count).collect();
            words.join(" ").to_lowercase().chars().collect()
        }
        Level::Intermediate | Level::Advanced => {
            let words: Vec<&str> = text.split_whitespace().take(word_count).collect();
            words.join(" ").chars().collect()
        }
    }
}

/// Picks a random paragraph tagged for `finger` and shapes it for `level`,
/// capped to `word_count` words. Falls back to a generic pangram if
/// (somehow) no paragraph is tagged for that finger.
pub fn pick_for_finger(finger: Finger, level: Level, word_count: usize) -> Vec<char> {
    let mut rng = rand::thread_rng();
    let candidates: Vec<&str> = PARAGRAPHS
        .iter()
        .filter(|(f, _)| *f == finger)
        .map(|(_, text)| *text)
        .collect();
    let text = candidates
        .choose(&mut rng)
        .copied()
        .unwrap_or("the quick brown fox jumps over the lazy dog");
    shape_for_level(text, level, word_count)
}

/// Picks any paragraph at random, ignoring finger tags, and shapes it for
/// `level`, capped to `word_count` words. Used for the "Random" training
/// mode.
pub fn pick_random(level: Level, word_count: usize) -> Vec<char> {
    let mut rng = rand::thread_rng();
    let (_, text) = PARAGRAPHS.choose(&mut rng).copied().unwrap_or((
        Finger::RightIndex,
        "the quick brown fox jumps over the lazy dog",
    ));
    shape_for_level(text, level, word_count)
}
