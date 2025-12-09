const MIN_NAME_LENGTH: usize = 3;
const MAX_NAME_LENGTH: usize = 20;

const BLOCKED_WORDS: &[&str] = &[
    "merda",
    "puta",
    "caralho",
    "foda",
    "porra",
    "buceta",
    "cu",
    "viado",
    "bicha",
    "puto",
    "filho da puta",
    "fdp",
    "vsf",
    "vai se foder",
    "vtnc",
    "fuck",
    "shit",
    "ass",
    "bitch",
    "cunt",
    "dick",
    "pussy",
    "nigger",
    "nigga",
    "faggot",
    "retard",
    "damn",
    "hell",
];

pub fn is_name_valid(name: &str) -> (bool, Option<String>) {
    let normalized = normalize(name);
    if name.len() < MIN_NAME_LENGTH {
        return (
            false,
            Some(format!(
                "Name must be at least {} characters",
                MIN_NAME_LENGTH
            )),
        );
    }
    if name.len() > MAX_NAME_LENGTH {
        return (
            false,
            Some(format!(
                "Name must be at most {} characters",
                MAX_NAME_LENGTH
            )),
        );
    }

    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != ' ' && ch != '-' && ch != '_' {
            return (
                false,
                Some(
                    "Name can only contain letters, numbers, spaces, hyphens and underscores"
                        .to_string(),
                ),
            );
        }
    }

    let name_lower = normalized.to_lowercase();
    for blocked_word in BLOCKED_WORDS {
        if name_lower.contains(&normalize(blocked_word)) {
            return (
                false,
                Some("Name contains inappropriate content".to_string()),
            );
        }
    }

    (true, None)
}

fn normalize(input: &str) -> String {
    use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
    input
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short() {
        let (ok, msg) = is_name_valid("ab");
        assert!(!ok);
        assert!(msg.unwrap().contains("at least"));
    }

    #[test]
    fn rejects_blocked_normalized() {
        let (ok, _) = is_name_valid("ÁsS_clã");
        assert!(!ok, "normalized blocked word should be rejected");
    }

    #[test]
    fn accepts_valid_with_accents() {
        let (ok, msg) = is_name_valid("João-Pedro_123");
        assert!(ok, "should accept accented and allowed symbols: {:?}", msg);
    }

    #[test]
    fn rejects_bad_chars() {
        let (ok, msg) = is_name_valid("abc$");
        assert!(!ok);
        assert!(msg.unwrap().contains("only contain"));
    }
}
