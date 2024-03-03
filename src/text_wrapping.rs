pub fn truncate_ellipse(string: &str, len: usize) -> String {
    if string.len() <= len {
        string.to_string()
    } else {
        string
            .chars()
            .enumerate()
            .filter_map(|(i, c)| {
                if i < len - 3 {
                    Some(c)
                } else if i >= len - 3 && i < len {
                    Some('.')
                } else {
                    None
                }
            })
            .collect()
    }
}

pub fn wrap_string(string: &str, len: usize) -> String {
    string
        .chars()
        .collect::<Vec<char>>()
        .chunks(len)
        .map(|chunk| chunk.iter().collect())
        .collect::<Vec<String>>()
        .join("\n")
}
