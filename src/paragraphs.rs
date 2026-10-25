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
        "Zany quokkas quietly amaze quiet zebras near azure lagoons, as quirky wizards analyze quaint zigzag mazes. A quiet zealot quickly amazed a zoo with an azure quilt, adoring every quirky zigzag along the quay. Zealous acrobats quietly questioned antique quartz vases, zealously organizing quaint artifacts across hazy plazas.",
    ),
    (
        Finger::LeftPinky,
        "Quaint wizards quickly organized an antique bazaar with azure quilts and quartz artifacts. Zebras and quokkas wandered past hazy plazas, gazing at quirky acrobats who amazed every quiet citizen. Always question hazy assumptions before zealous zealots organize bizarre quizzes across ancient plazas.",
    ),
    (
        Finger::LeftPinky,
        "A quiet zealot quickly amazed a zoo with an azure quilt, adoring every quirky zigzag along the quay.",
    ),
    (
        Finger::LeftRing,
        "Wise walruses swiftly waxed six wooden saws, watching sixty swans swim past waxwork statues on the sand. Six swift wasps waxed a wide wooden wall, swirling sideways with a strange, restless swagger all week. Sweet swallows swept westward over windy waves, seeking warmth while wise foxes watched silently beneath willow shadows.",
    ),
    (
        Finger::LeftRing,
        "Slow whispers swept across the western woods while wild foxes searched for warm shelter. Sixty swans swam swiftly southward as winter winds waxed stronger over snowy swamps. Watchful woodsmen saw wooden wagons sink waist-deep into wet soil without slowing their steady stride.",
    ),
    (
        Finger::LeftRing,
        "Six swift wasps waxed a wide wooden wall, swirling sideways with a strange, restless swagger all week.",
    ),
    (
        Finger::LeftMiddle,
        "Dedicated ducks decided each decade to educate deceived crickets, creating deep, calm, decent codes. Educated coders decided each deed deserved a decisive, dedicated defense against deceptive, careless code. Clever detectives discovered hidden credentials embedded deep inside encrypted directories, declaring complete victory over chaotic defects.",
    ),
    (
        Finger::LeftMiddle,
        "Careful designers create cleaner declarations, deciphering complicated code defects during each development cycle. Decisive directors demanded dedicated efforts to decode ancient documents concealed beneath dark crypts. Every decent decision depended directly upon clear evidence collected by diligent detectives across distant cities.",
    ),
    (
        Finger::LeftMiddle,
        "Educated coders decided each deed deserved a decisive, dedicated defense against deceptive, careless code.",
    ),
    (
        Finger::LeftIndex,
        "Brave frogs bravely gathered fresh berries, forgetting the great, foggy river beyond the vibrant garden. Every great forger brought a firm, vibrant brief, gathering brave friends before the growing thunderstorm. Fierce forest rangers traveled through rugged valleys, finding bright green ferns and vibrant blossoms beneath towering branches.",
    ),
    (
        Finger::LeftIndex,
        "Vibrant travelers found great comfort beside burning fires after trekking through vast riverfront valleys. Brave runners finished first after navigating rugged gravel trails, gathering praise from cheerful spectators. Forgetful foragers brought fragrant berries back to friendly villages before twilight faded into foggy shadows.",
    ),
    (
        Finger::LeftIndex,
        "Every great forger brought a firm, vibrant brief, gathering brave friends before the growing thunderstorm.",
    ),
    (
        Finger::LeftThumb,
        "Go on and do it, one bit at a bit, and do not stop to sit; it is not that hard to go on and do it. Take a good look at what is done, step by step and day by day, so that we can be glad to see how far we went. Good work is born from steady hands and calm minds.",
    ),
    (
        Finger::LeftThumb,
        "Stay on the path and keep your pace steady so that every step brings you closer to your goal. When you run or walk with focus, no hill is too steep and no task is too big to finish in good time.",
    ),
    (
        Finger::RightThumb,
        "Go do it now, go to bed by ten or so, if it is ok to you my dear old pal, we can go at dawn. It is up to us to go on and do it now, no matter how hot or dry it is out there today. Let us make sure that we keep our word and do not turn back.",
    ),
    (
        Finger::RightThumb,
        "We can go as far as we want if we take care to move with ease and rhythm. Let each word and each space fall into place like quiet rain upon dry soil, building speed and skill day after day as we learn.",
    ),
    (
        Finger::RightIndex,
        "Many hungry monkeys jumped joyfully near humming jungles, munching yummy honey under the sunny noon sky. Yesterday my human neighbor joyfully hummed a funny jingle, running through muddy meadows until noon. Young musicians joined joyful reunions, humming rhythmic hymns under moonlight while enjoying yummy mangoes and crunchy nuts.",
    ),
    (
        Finger::RightIndex,
        "Numerous youthful travelers journeyed through mountainous canyons, joining joyful gatherings under humid jungle canopies. Many monks hummed tranquil hymns, honoring ancient traditions while enjoying sunny mornings near muddy riverbanks. Never underestimate how much joy good humor brings to weary human hearts across the nation.",
    ),
    (
        Finger::RightIndex,
        "Yesterday my human neighbor joyfully hummed a funny jingle, running through muddy meadows until noon.",
    ),
    (
        Finger::RightMiddle,
        "I think, I know, I like quiet nights, watching kind kids skip, kick, and giggle, thinking of nice tricks. I picked six kites, thinking I might link nice kids, liking quick tricks, winking, and skipping quietly. Keen kings kept striking silver kettles, seeking kin in distant kingdoms while knitting intricate silk kilts.",
    ),
    (
        Finger::RightMiddle,
        "Knowledgeable knights kindly welcomed inquisitive kids into historic kingdoms, liking bright minds and quick thinking. Inquisitive kings kept linking distant cities through solid brick bridges, thinking about civic stability and lasting kinship. Kind critics praised brisk violin rhythms played in illuminated music halls tonight.",
    ),
    (
        Finger::RightMiddle,
        "I picked six kites, thinking I might link nice kids, liking quick tricks, winking, and skipping quietly.",
    ),
    (
        Finger::RightRing,
        "Lonely owls slowly circled cold, golden fields below, following old moonlit trolls along a long, lonely road. Old sailors slowly followed golden clouds below a cold, lonely moon over a long, hollow valley. Loyal colonels strolled alongside rolling hills, looking for lost soldiers while cool autumnal blossoms floated softly downward.",
    ),
    (
        Finger::RightRing,
        "Colossal colonial vessels sailed along lonely coastlines, following golden constellations toward cold polar islands. Old loyal folks told colorful folklore aloud, looking over peaceful rolling hills below moonlit clouds. Solitary wolves howled softly into cool nocturnal hollows while glowing golden blossoms closed gently.",
    ),
    (
        Finger::RightRing,
        "Old sailors slowly followed golden clouds below a cold, lonely moon over a long, hollow valley.",
    ),
    (
        Finger::RightPinky,
        "Perhaps a puppy quickly hopped upon a purple pipe; papa perhaps hoped to keep the puppy happy. Perhaps upon a proper path, a puppy popped up; people happily hoped to help the puppy up. Proud painters prepared polished paper, picturing picturesque pink palaces, popular parks, and playful puppies pursuing pigeons.",
    ),
    (
        Finger::RightPinky,
        "Patient philosophers pondered puzzling paradoxes, prompting polite pupils to prepare profound replies. Plentiful purple plums ripened atop peaceful orchard slopes, pleasing proud people who picked fresh produce. Proper preparation prevents poor performance, providing predictable power to purposeful professionals everywhere.",
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
/// If `word_count` words are selected, punctuation is cleaned so the final
/// word terminates with appropriate punctuation (`.`, `!`, `?`) without
/// dangling commas or colons.
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
            let mut words: Vec<String> = text
                .split_whitespace()
                .take(word_count)
                .map(|s| s.to_string())
                .collect();
            if let Some(last) = words.last_mut() {
                while last.ends_with(',')
                    || last.ends_with(';')
                    || last.ends_with(':')
                    || last.ends_with('-')
                    || last.ends_with('—')
                {
                    last.pop();
                }
                if !last.ends_with('.') && !last.ends_with('!') && !last.ends_with('?') {
                    last.push('.');
                }
            }
            words.join(" ").chars().collect()
        }
    }
}

/// Picks a random paragraph tagged for `finger` and shapes it for `level`,
/// chaining multiple candidate passages if needed until the requested
/// `word_count` (up to 60 words or more) is met.
pub fn pick_for_finger(finger: Finger, level: Level, word_count: usize) -> Vec<char> {
    let mut rng = rand::thread_rng();
    let mut candidates: Vec<&str> = PARAGRAPHS
        .iter()
        .filter(|(f, _)| *f == finger)
        .map(|(_, text)| *text)
        .collect();
    if candidates.is_empty() {
        candidates.push("the quick brown fox jumps over the lazy dog");
    }
    candidates.shuffle(&mut rng);

    let mut combined = String::new();
    let mut count = 0;
    let mut idx = 0;
    while count < word_count {
        let text = candidates[idx % candidates.len()];
        if !combined.is_empty() {
            combined.push(' ');
        }
        combined.push_str(text);
        count = combined.split_whitespace().count();
        idx += 1;
    }

    shape_for_level(&combined, level, word_count)
}

/// Picks paragraphs at random, ignoring finger tags, and shapes it for
/// `level`, chaining multiple candidate passages if needed until the requested
/// `word_count` (up to 60 words or more) is met.
pub fn pick_random(level: Level, word_count: usize) -> Vec<char> {
    let mut rng = rand::thread_rng();
    let mut all_texts: Vec<&str> = PARAGRAPHS.iter().map(|(_, text)| *text).collect();
    all_texts.shuffle(&mut rng);

    let mut combined = String::new();
    let mut count = 0;
    let mut idx = 0;
    while count < word_count {
        let text = all_texts[idx % all_texts.len()];
        if !combined.is_empty() {
            combined.push(' ');
        }
        combined.push_str(text);
        count = combined.split_whitespace().count();
        idx += 1;
    }

    shape_for_level(&combined, level, word_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_for_level_word_counts() {
        let text = "One two three four five six seven eight nine ten eleven twelve.";
        for count in [5, 10, 15, 20, 30, 40, 50, 60] {
            let beginner = shape_for_level(text, Level::Beginner, count);
            let s: String = beginner.into_iter().collect();
            let words: Vec<&str> = s.split_whitespace().collect();
            assert_eq!(words.len(), count.min(12));

            let intermediate = shape_for_level(text, Level::Intermediate, count);
            let s_inter: String = intermediate.into_iter().collect();
            let words_inter: Vec<&str> = s_inter.split_whitespace().collect();
            assert_eq!(words_inter.len(), count.min(12));
            assert!(s_inter.ends_with('.'));
        }
    }

    #[test]
    fn test_pick_for_finger_reaches_60_words() {
        for finger in Finger::ALL {
            let res = pick_for_finger(finger, Level::Intermediate, 60);
            let s: String = res.into_iter().collect();
            let words: Vec<&str> = s.split_whitespace().collect();
            assert_eq!(words.len(), 60, "Finger {finger} did not yield 60 words");
            assert!(s.ends_with('.'), "Finger {finger} text should end with period");
        }
    }

    #[test]
    fn test_pick_random_reaches_various_lengths() {
        for count in [5, 10, 15, 20, 30, 40, 50, 60] {
            let res = pick_random(Level::Advanced, count);
            let s: String = res.into_iter().collect();
            let words: Vec<&str> = s.split_whitespace().collect();
            assert_eq!(words.len(), count, "Expected {count} words, got {}", words.len());
            assert!(s.ends_with('.') || s.ends_with('!') || s.ends_with('?'));
        }
    }

    #[test]
    fn test_punctuation_cleanup_on_trailing_comma() {
        let text = "First word, second word, third word,";
        let res = shape_for_level(text, Level::Intermediate, 6);
        let s: String = res.into_iter().collect();
        assert_eq!(s, "First word, second word, third word.");

        let res2 = shape_for_level(text, Level::Intermediate, 2);
        let s2: String = res2.into_iter().collect();
        assert_eq!(s2, "First word.");
    }
}
