use std::collections::HashSet;

/// Returns the accented variants for `letter` given the active languages.
/// Characters are deduplicated preserving order across languages.
pub fn accents_for_letter(letter: char, languages: &[String]) -> Vec<char> {
    let lower = letter.to_lowercase().next().unwrap_or(letter);
    let is_upper = letter.is_uppercase();
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for lang in languages {
        if let Some(chars) = chars_for_lang(lower, lang) {
            for &ch in chars {
                let adjusted = if is_upper {
                    ch.to_uppercase().next().unwrap_or(ch)
                } else {
                    ch
                };
                if seen.insert(adjusted) {
                    result.push(adjusted);
                }
            }
        }
    }
    result
}

/// True if the given letter has any accents in the given language set.
pub fn has_accents(letter: char, languages: &[String]) -> bool {
    !accents_for_letter(letter, languages).is_empty()
}

fn chars_for_lang(letter: char, lang: &str) -> Option<&'static [char]> {
    match (lang, letter) {
        // French
        ("fr", 'a') => Some(&['à', 'â', 'æ']),
        ("fr", 'c') => Some(&['ç']),
        ("fr", 'e') => Some(&['é', 'è', 'ê', 'ë']),
        ("fr", 'i') => Some(&['î', 'ï']),
        ("fr", 'o') => Some(&['ô', 'œ']),
        ("fr", 'u') => Some(&['ù', 'û', 'ü']),
        ("fr", 'y') => Some(&['ÿ']),

        // Spanish
        ("es", 'a') => Some(&['á']),
        ("es", 'e') => Some(&['é']),
        ("es", 'i') => Some(&['í']),
        ("es", 'n') => Some(&['ñ']),
        ("es", 'o') => Some(&['ó']),
        ("es", 'u') => Some(&['ú', 'ü']),

        // German
        ("de", 'a') => Some(&['ä']),
        ("de", 'o') => Some(&['ö']),
        ("de", 'u') => Some(&['ü']),
        ("de", 's') => Some(&['ß']),

        // Portuguese
        ("pt", 'a') => Some(&['á', 'â', 'ã', 'à']),
        ("pt", 'c') => Some(&['ç']),
        ("pt", 'e') => Some(&['é', 'ê']),
        ("pt", 'i') => Some(&['í']),
        ("pt", 'o') => Some(&['ó', 'ô', 'õ']),
        ("pt", 'u') => Some(&['ú']),

        // Italian
        ("it", 'a') => Some(&['à']),
        ("it", 'e') => Some(&['è', 'é']),
        ("it", 'i') => Some(&['ì', 'í']),
        ("it", 'o') => Some(&['ò', 'ó']),
        ("it", 'u') => Some(&['ù']),

        // Dutch
        ("nl", 'a') => Some(&['á', 'ä']),
        ("nl", 'e') => Some(&['é', 'ë']),
        ("nl", 'i') => Some(&['í', 'ï']),
        ("nl", 'o') => Some(&['ó', 'ö']),
        ("nl", 'u') => Some(&['ú', 'ü']),

        // Swedish
        ("sv", 'a') => Some(&['å', 'ä']),
        ("sv", 'o') => Some(&['ö']),

        // Norwegian / Danish
        ("no" | "da", 'a') => Some(&['å', 'æ']),
        ("no" | "da", 'o') => Some(&['ø']),

        // Polish
        ("pl", 'a') => Some(&['ą']),
        ("pl", 'c') => Some(&['ć']),
        ("pl", 'e') => Some(&['ę']),
        ("pl", 'l') => Some(&['ł']),
        ("pl", 'n') => Some(&['ń']),
        ("pl", 'o') => Some(&['ó']),
        ("pl", 's') => Some(&['ś']),
        ("pl", 'z') => Some(&['ź', 'ż']),

        // Turkish
        ("tr", 'c') => Some(&['ç']),
        ("tr", 'g') => Some(&['ğ']),
        ("tr", 'i') => Some(&['ı', 'İ']),
        ("tr", 'o') => Some(&['ö']),
        ("tr", 's') => Some(&['ş']),
        ("tr", 'u') => Some(&['ü']),

        // Romanian
        ("ro", 'a') => Some(&['ă', 'â']),
        ("ro", 'i') => Some(&['î']),
        ("ro", 's') => Some(&['ș']),
        ("ro", 't') => Some(&['ț']),

        // Czech
        ("cs", 'a') => Some(&['á']),
        ("cs", 'c') => Some(&['č']),
        ("cs", 'd') => Some(&['ď']),
        ("cs", 'e') => Some(&['é', 'ě']),
        ("cs", 'i') => Some(&['í']),
        ("cs", 'n') => Some(&['ň']),
        ("cs", 'o') => Some(&['ó']),
        ("cs", 'r') => Some(&['ř']),
        ("cs", 's') => Some(&['š']),
        ("cs", 't') => Some(&['ť']),
        ("cs", 'u') => Some(&['ú', 'ů']),
        ("cs", 'y') => Some(&['ý']),
        ("cs", 'z') => Some(&['ž']),

        // Hungarian
        ("hu", 'a') => Some(&['á']),
        ("hu", 'e') => Some(&['é']),
        ("hu", 'i') => Some(&['í']),
        ("hu", 'o') => Some(&['ó', 'ö', 'ő']),
        ("hu", 'u') => Some(&['ú', 'ü', 'ű']),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn french_accents() {
        let langs = vec!["fr".to_string()];
        let accents = accents_for_letter('e', &langs);
        assert_eq!(accents, vec!['é', 'è', 'ê', 'ë']);
    }

    #[test]
    fn uppercase_accents() {
        let langs = vec!["fr".to_string()];
        let accents = accents_for_letter('E', &langs);
        assert_eq!(accents, vec!['É', 'È', 'Ê', 'Ë']);
    }

    #[test]
    fn multi_language_dedup() {
        let langs = vec!["fr".to_string(), "pt".to_string()];
        let accents = accents_for_letter('a', &langs);
        // fr: à, â, æ — pt: á, â, ã, à → merged: à, â, æ, á, ã
        assert_eq!(accents, vec!['à', 'â', 'æ', 'á', 'ã']);
    }

    #[test]
    fn no_accents_for_x() {
        let langs = vec!["fr".to_string()];
        assert!(!has_accents('x', &langs));
    }
}
