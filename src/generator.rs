//! Targeted practice-text generator.
//!
//! The idea: once we know which finger is currently making the most
//! mistakes, we want practice text that is heavily loaded with letters that
//! finger is responsible for, so the user gets focused repetition instead of
//! generic text.

use crate::finger::{Finger, TypingEngine};
use rand::seq::SliceRandom;
use rand::Rng;

/// A small built-in vocabulary. Real applications would load a much larger
/// word list from a file; this is intentionally compact for a boilerplate.
const WORD_BANK: &[&str] = &[
    "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "pack",
    "my", "box", "with", "five", "dozen", "liquor", "jugs", "amazingly",
    "few", "discotheques", "provide", "jukeboxes", "waltz", "bad", "nymph",
    "for", "quick", "jigs", "vex", "sphinx", "of", "black", "quartz",
    "judge", "my", "vow", "few", "black", "taxis", "drive", "up", "major",
    "roads", "on", "icy", "winter", "mornings", "pack", "my", "box",
    "grumpy", "wizards", "make", "toxic", "brew", "for", "the", "evil",
    "queen", "and", "jack", "how", "quickly", "daft", "jumping", "zebras",
    "vex", "crazy", "fredrick", "just", "won", "prize", "cozy", "lummox",
    "gives", "smart", "squid", "eyes", "when", "muzzy", "bikers", "hew",
    "razor", "power", "keyboard", "correct", "typing", "space", "focus",
    "practice", "session", "finger", "accuracy", "matrix", "history",
    "target", "letters", "index", "middle", "ring", "pinky", "thumb",
    "words", "chart", "graph", "trend", "value", "point", "line", "bar",
];

/// Scores a word by how many of its characters are typed by `finger`
/// (higher is more relevant for practicing that finger).
fn score_word(word: &str, engine: &TypingEngine, finger: Finger) -> usize {
    word.chars()
        .filter(|c| engine.finger_for(*c) == Some(finger))
        .count()
}

/// Generates a whitespace-separated practice string of `word_count` words,
/// biased toward whichever finger is currently weakest.
///
/// Strategy:
/// 1. Ask the engine which finger has the worst error rate so far.
/// 2. Rank the word bank by how "loaded" each word is with that finger's
///    characters.
/// 3. Sample mostly from the top-scoring words, with a smaller fraction of
///    random words mixed in so practice text still reads naturally and
///    doesn't feel like a wall of the same three letters.
///
/// If no finger has any recorded keystrokes yet (a fresh session), falls
/// back to a plain random sample from the word bank.
#[allow(dead_code)] // convenience wrapper kept for callers that always want
                     // "whatever's currently weakest", even though the app
                     // itself now goes through `generate_for_finger` directly
                     // so it can distinguish "no data yet" from "user chose Random".
pub fn generate_practice_text(engine: &TypingEngine, word_count: usize) -> String {
    let weak_finger = engine.get_most_faulty_finger().map(|(f, _)| f);
    generate_for_finger(engine, weak_finger, word_count)
}

/// Same as [`generate_practice_text`] but lets the caller force a specific
/// finger (used by "practice this finger" style flows), rather than always
/// picking whatever is currently worst.
pub fn generate_for_finger(
    engine: &TypingEngine,
    finger: Option<Finger>,
    word_count: usize,
) -> String {
    let mut rng = rand::thread_rng();

    let Some(finger) = finger else {
        // No data yet: just shuffle the word bank.
        let mut words: Vec<&str> = WORD_BANK.to_vec();
        words.shuffle(&mut rng);
        return words.into_iter().take(word_count).collect::<Vec<_>>().join(" ");
    };

    let mut scored: Vec<(&str, usize)> = WORD_BANK
        .iter()
        .map(|w| (*w, score_word(w, engine, finger)))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    // Top ~40% of the ranked bank is our "high value" pool; everything else
    // is the "general" pool used to keep the text varied.
    let split = (scored.len() as f64 * 0.4).ceil() as usize;
    let (heavy_pool, general_pool) = scored.split_at(split.max(1).min(scored.len()));

    let mut result = Vec::with_capacity(word_count);
    for _ in 0..word_count {
        // 70% chance: pull from the finger-loaded pool. 30%: general pool,
        // for natural variety.
        let pool = if rng.gen_bool(0.7) && !heavy_pool.is_empty() {
            heavy_pool
        } else if !general_pool.is_empty() {
            general_pool
        } else {
            heavy_pool
        };
        if let Some((word, _)) = pool.choose(&mut rng) {
            result.push(*word);
        }
    }

    result.join(" ")
}
