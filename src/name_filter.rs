const MIN_NAME_LENGTH: usize = 3;
const MAX_NAME_LENGTH: usize = 20;

const BLOCKED_WORDS: &[&str] = &[
    "merda", "puta", "caralho", "foda", "porra", "buceta", "cu", "viado", "bicha",
    "puto", "filho da puta", "fdp", "vsf", "vai se foder", "vtnc",
    "fuck", "shit", "ass", "bitch", "cunt", "dick", "pussy", "nigger", "nigga",
    "faggot", "retard", "damn", "hell",
];

pub fn is_name_valid(name: &str) -> (bool, Option<String>) {
    if name.len() < MIN_NAME_LENGTH {
        return (false, Some(format!("O nome deve ter pelo menos {} caracteres", MIN_NAME_LENGTH)));
    }
    if name.len() > MAX_NAME_LENGTH {
        return (false, Some(format!("O nome deve ter no máximo {} caracteres", MAX_NAME_LENGTH)));
    }

    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != ' ' && ch != '-' && ch != '_' {
            return (false, Some("O nome pode conter apenas letras, números, espaços, hífens e underscores".to_string()));
        }
    }

    let name_lower = name.to_lowercase();
    for blocked_word in BLOCKED_WORDS {
        if name_lower.contains(blocked_word) {
            return (false, Some("O nome contém conteúdo inadequado".to_string()));
        }
    }

    (true, None)
}
