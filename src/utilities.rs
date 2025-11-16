pub(crate) fn word_wrap(text: impl Into<String>, max_len: usize) -> Vec<String> {
    let text: String = text.into();
    let mut lines = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + word.len() + 1 > max_len {
            lines.push(line.trim().to_string());
            line.clear();
        }

        if !line.is_empty() {
            line.push(' ');
        }

        line.push_str(word);
    }

    if !line.is_empty() {
        lines.push(line.trim().to_string());
    }

    lines
}

pub(crate) trait Conversion {
    fn mm_to_pt(&self, mm: f32) -> f32 {
        mm * 2.83465
    }
}
