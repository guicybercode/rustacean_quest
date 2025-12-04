// Filtro de conteúdo para nomes de jogadores

const MIN_NAME_LENGTH: usize = 3;
const MAX_NAME_LENGTH: usize = 20;

// Lista básica de palavras bloqueadas (case-insensitive)
const BLOCKED_WORDS: &[&str] = &[
    // Português
    "merda", "puta", "caralho", "foda", "porra", "buceta", "cu", "viado", "bicha",
    "puto", "filho da puta", "fdp", "vsf", "vai se foder", "vtnc",
    // Inglês
    "fuck", "shit", "ass", "bitch", "cunt", "dick", "pussy", "nigger", "nigga",
    "faggot", "retard", "damn", "hell",
];

pub fn is_name_valid(name: &str) -> (bool, Option<String>) {
    // Verificar comprimento
    if name.len() < MIN_NAME_LENGTH {
        return (false, Some(format!("O nome deve ter pelo menos {} caracteres", MIN_NAME_LENGTH)));
    }
    if name.len() > MAX_NAME_LENGTH {
        return (false, Some(format!("O nome deve ter no máximo {} caracteres", MAX_NAME_LENGTH)));
    }

    // Verificar caracteres permitidos (alfanuméricos, espaço, hífen, underscore)
    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != ' ' && ch != '-' && ch != '_' {
            return (false, Some("O nome pode conter apenas letras, números, espaços, hífens e underscores".to_string()));
        }
    }

    // Verificar palavras bloqueadas (case-insensitive)
    let name_lower = name.to_lowercase();
    for blocked_word in BLOCKED_WORDS {
        if name_lower.contains(blocked_word) {
            return (false, Some("O nome contém conteúdo inadequado".to_string()));
        }
    }

    // Nome válido
    (true, None)
}


