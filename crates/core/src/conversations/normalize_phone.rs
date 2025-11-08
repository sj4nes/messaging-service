/// Normalize phone numbers: keep leading '+' if present, retain digits only.
pub fn normalize_phone(raw: &str) -> String {
    let mut out = String::new();
    for (i, ch) in raw.chars().enumerate() {
        if ch.is_ascii_digit() {
            out.push(ch);
        } else if ch == '+' && i == 0 {
            out.push(ch);
        }
    }
    out
}
