//! Bigram Hesitation Ladder Drill Generator (Milestone 5.1).
//!
//! Generates 3-tier progressive exercises targeting the typist's slowest
//! transitions measured by Inter-Key Interval (IKI):
//! - Tier 1 (Beginner): Isolation Repeats (`qu qu qui que squ qua qu`)
//! - Tier 2 (Intermediate): Targeted Words (`quick queen equal square liquid`)
//! - Tier 3 (Advanced): Context Sentences (`The quick queen required quiet squares.`)

use crate::app::{Level, RoundLength};

pub const DEFAULT_AWKWARD_BIGRAMS: &[(char, char)] = &[
    ('q', 'u'),
    ('p', 'r'),
    ('s', 'w'),
    ('b', 'r'),
    ('t', 'r'),
    ('s', 't'),
    ('c', 'r'),
    ('g', 'r'),
    ('f', 'l'),
    ('p', 'l'),
];

/// Curated dictionary of words containing common English bigrams.
pub const BIGRAM_WORD_BANK: &[(&str, &[&str])] = &[
    ("qu", &["quick", "queen", "quiet", "quite", "quote", "equal", "liquid", "require", "acquire", "square", "squad", "quench"]),
    ("pr", &["practice", "prepare", "program", "prompt", "proper", "protect", "provide", "problem", "project", "primary", "private"]),
    ("sw", &["swift", "switch", "sweet", "swing", "sweep", "sword", "swarm", "swim", "swear", "swallow", "swiftly"]),
    ("br", &["brave", "bridge", "bright", "broad", "break", "branch", "breeze", "brother", "brown", "bring", "brief"]),
    ("tr", &["train", "trail", "track", "trace", "travel", "trust", "truth", "trend", "trigger", "transform", "transmit"]),
    ("st", &["start", "state", "step", "still", "stop", "store", "strong", "stream", "structure", "standard", "status"]),
    ("cr", &["create", "craft", "crisp", "cross", "crown", "credit", "critical", "crypto", "crash", "crowd"]),
    ("gr", &["great", "grace", "grand", "grasp", "green", "ground", "growth", "group", "graphic", "grid"]),
    ("fl", &["flash", "flame", "flow", "floor", "fleet", "flight", "fluid", "flexible", "floating", "flavor"]),
    ("pl", &["plan", "plane", "plate", "play", "plus", "platform", "plugin", "pleasant", "plenty", "plasma"]),
    ("th", &["think", "thing", "thread", "three", "through", "theory", "theme", "thorough", "thermal", "throttle"]),
    ("ch", &["check", "chain", "chance", "change", "chart", "choice", "chunk", "channel", "charge", "character"]),
    ("sh", &["sharp", "shift", "share", "shell", "shine", "shock", "short", "shadow", "shield", "showcase"]),
    ("wh", &["where", "which", "while", "white", "whole", "whisper", "wheel", "whether", "whenever", "whistle"]),
    ("ph", &["phase", "phone", "photo", "phrase", "physics", "phantom", "physical", "sphere", "graph", "glyph"]),
];

/// Curated context sentences for popular bigrams.
pub const BIGRAM_SENTENCES: &[(&str, &[&str])] = &[
    ("qu", &[
        "The quick queen required quiet squares for equal liquid tests.",
        "A quaint squirrel quietly requested quick answers from the squad.",
        "Frequent questions require quite unique and quiet inquiries.",
    ]),
    ("pr", &[
        "Proper programming practices provide prompt and precise progress.",
        "Primary projects produce practical solutions for protected systems.",
        "Private procedures prevent problems and promote professional pride.",
    ]),
    ("sw", &[
        "Swift software switches safely between sweet and swift sweeps.",
        "The swift swimmer swallowed sweet sips as the swings swayed.",
        "Smart switches swiftly sweep signals without awkward delays.",
    ]),
    ("br", &[
        "Brave brothers build broad bridges across the bright brook.",
        "A brief breeze brought bright branches into the broad brook.",
        "Bring brave brethren to break through the brittle bronze barrier.",
    ]),
    ("tr", &[
        "Train drivers track true trends through trusted transport trails.",
        "Tremendous trust transforms troubled trails into true triumphs.",
        "Tracking transport traffic triggers true trends across terminals.",
    ]),
    ("st", &[
        "Start storing standard structures to strengthen stable systems.",
        "Steady steps steer standard streaming strategies without stress.",
        "Static structures stand strong against steady stream stresses.",
    ]),
];

/// Selects the target bigram from the user's recorded slowest transitions,
/// or falls back to an awkward default.
pub fn select_target_bigram(slowest: &[((char, char), f64)]) -> (char, char) {
    if let Some(&((c1, c2), _)) = slowest.first() {
        if c1.is_alphanumeric() && c2.is_alphanumeric() {
            return (c1.to_ascii_lowercase(), c2.to_ascii_lowercase());
        }
    }
    DEFAULT_AWKWARD_BIGRAMS[0]
}

/// Generates a targeted Bigram Ladder practice string.
pub fn generate_bigram_drill(
    slowest: &[((char, char), f64)],
    level: Level,
    length: RoundLength,
) -> Vec<char> {
    let (c1, c2) = select_target_bigram(slowest);
    let target_words = length.word_count();
    let bigram_str = format!("{c1}{c2}");

    match level {
        Level::Beginner => generate_tier1_isolation(c1, c2, target_words),
        Level::Intermediate => generate_tier2_words(&bigram_str, c1, c2, target_words),
        Level::Advanced => generate_tier3_sentences(&bigram_str, c1, c2, target_words),
    }
}

fn generate_tier1_isolation(c1: char, c2: char, count: usize) -> Vec<char> {
    let vowels = ['a', 'e', 'i', 'o', 'u'];
    let mut words = Vec::new();

    let bg = format!("{c1}{c2}");
    words.push(bg.clone());
    words.push(bg.clone());

    for v in vowels {
        words.push(format!("{bg}{v}"));
        words.push(format!("{v}{bg}"));
        if words.len() >= count {
            break;
        }
    }

    while words.len() < count {
        words.push(bg.clone());
        words.push(format!("{bg}a"));
        words.push(format!("s{bg}"));
    }

    words.truncate(count);
    words.join(" ").chars().collect()
}

fn generate_tier2_words(bigram: &str, c1: char, c2: char, count: usize) -> Vec<char> {
    let word_list = BIGRAM_WORD_BANK
        .iter()
        .find(|(bg, _)| *bg == bigram)
        .map(|(_, words)| *words);

    let mut words = Vec::new();
    if let Some(list) = word_list {
        let mut idx = 0;
        while words.len() < count {
            words.push(list[idx % list.len()].to_string());
            idx += 1;
        }
    } else {
        // Dynamic synthetic fallback if bigram has no curated entry
        let synthetic = [
            format!("{c1}{c2}ed"),
            format!("{c1}{c2}ing"),
            format!("re{c1}{c2}"),
            format!("un{c1}{c2}"),
            format!("{c1}{c2}er"),
            format!("in{c1}{c2}"),
            format!("{c1}{c2}ly"),
            format!("pro{c1}{c2}"),
        ];
        let mut idx = 0;
        while words.len() < count {
            words.push(synthetic[idx % synthetic.len()].clone());
            idx += 1;
        }
    }

    words.truncate(count);
    words.join(" ").chars().collect()
}

fn generate_tier3_sentences(bigram: &str, c1: char, c2: char, count: usize) -> Vec<char> {
    let sentence_list = BIGRAM_SENTENCES
        .iter()
        .find(|(bg, _)| *bg == bigram)
        .map(|(_, s)| *s);

    if let Some(sentences) = sentence_list {
        let mut text = String::new();
        let mut current_words = 0;
        let mut idx = 0;
        while current_words < count {
            let sentence = sentences[idx % sentences.len()];
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(sentence);
            current_words += sentence.split_whitespace().count();
            idx += 1;
        }
        text.chars().collect()
    } else {
        // Fall back to Tier 2 if no predefined sentences exist for this awkward bigram
        generate_tier2_words(bigram, c1, c2, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_target_bigram_fallback() {
        let empty: [((char, char), f64); 0] = [];
        let bg = select_target_bigram(&empty);
        assert_eq!(bg, ('q', 'u'));
    }

    #[test]
    fn test_select_target_bigram_from_slowest() {
        let slowest = [(('s', 'w'), 320.0), (('p', 'r'), 280.0)];
        let bg = select_target_bigram(&slowest);
        assert_eq!(bg, ('s', 'w'));
    }

    #[test]
    fn test_tier1_isolation_drill_length() {
        let drill = generate_bigram_drill(&[(('q', 'u'), 300.0)], Level::Beginner, RoundLength::Words10);
        let text: String = drill.into_iter().collect();
        let words: Vec<&str> = text.split_whitespace().collect();
        assert_eq!(words.len(), 10);
        assert!(words.contains(&"qu"));
    }

    #[test]
    fn test_tier2_words_drill() {
        let drill = generate_bigram_drill(&[(('q', 'u'), 300.0)], Level::Intermediate, RoundLength::Burst5);
        let text: String = drill.into_iter().collect();
        let words: Vec<&str> = text.split_whitespace().collect();
        assert_eq!(words.len(), 5);
        for w in words {
            assert!(w.contains("qu"));
        }
    }

    #[test]
    fn test_tier3_sentences_drill() {
        let drill = generate_bigram_drill(&[(('s', 'w'), 300.0)], Level::Advanced, RoundLength::Words15);
        let text: String = drill.into_iter().collect();
        assert!(text.contains("Swift") || text.contains("swift"));
    }
}
