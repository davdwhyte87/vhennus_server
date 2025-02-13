

pub fn truncate_string(s: String) -> String {
    let truncated: String = s.chars().take(40).collect();
    if s.chars().count() > 40 {
        format!("{}...", truncated)
    } else {
        truncated
    }
}