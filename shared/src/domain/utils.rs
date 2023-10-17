const FORBIDDEN_CHARACTER: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

pub fn contains_forbidden_characters(s: &str) -> bool {
    s.chars().any(|g| FORBIDDEN_CHARACTER.contains(&g))
}
