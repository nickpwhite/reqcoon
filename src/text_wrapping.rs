use ratatui::text::Text;
use textwrap;

pub fn truncate_ellipse(string: &str, len: usize) -> Text {
    let retval = if string.len() <= len {
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
    };

    Text::raw(retval)
}

pub fn wrap_string(string: &str, len: usize) -> Text {
    let options = textwrap::Options::new(len)
        .word_separator(textwrap::WordSeparator::Custom(find_words_at_char));
    Text::raw(textwrap::fill(string, options))
}

fn find_words_at_char(line: &str) -> Box<dyn Iterator<Item = textwrap::core::Word<'_>> + '_> {
    Box::new(
        line.char_indices()
            .map(|(i, _)| textwrap::core::Word::from(&line[i..i + 1])),
    )
}
