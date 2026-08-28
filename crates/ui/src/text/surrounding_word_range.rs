use std::ops::Range;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Charkind {
    Whitespace,
    Punctuation,
    Word,
}

fn char_kind(c: char) -> Charkind {
    if c.is_alphanumeric() {
        Charkind::Word
    } else if c.is_whitespace() {
        Charkind::Whitespace
    } else {
        Charkind::Punctuation
    }
}

/// Byte range of the word touching `offset` in `content`.
///
/// ```text
/// "hello world"
///         |
///     offset = 8  ->  6..11 ("world")
///
/// "hello world"
///       |
///  offset = 5  ->  0..5 ("hello", not the space)
/// ```
pub(crate) fn surrounding_word_range(content: &str, offset: usize) -> Range<usize> {
    let content: &str = content;

    let prev_kind = content[..offset].chars().next_back().map(char_kind);
    let next_kind = content[offset..].chars().next().map(char_kind);

    let Some(kind) = prev_kind.max(next_kind) else {
        return offset..offset;
    };

    let start = content[..offset]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| char_kind(c) == kind)
        .last()
        .map_or(offset, |(i, _)| i);

    let end = content[offset..]
        .char_indices()
        .take_while(|&(_, c)| char_kind(c) == kind)
        .last()
        .map_or(offset, |(i, c)| offset + i + c.len_utf8());

    start..end
}

#[cfg(test)]
mod tests {
    use crate::text::surrounding_word_range::surrounding_word_range;

    #[test]
    fn should_return_surrounding_word_range() {
        let content = "hello there";

        // Cursor is at 1 -> h|ello
        let result = surrounding_word_range(content, 1);
        assert_eq!(result, 0..5)
    }

    #[test]
    fn should_return_surrounding_word_range_until_punctuation() {
        let content = "who am, i?";

        // Cursor is at 5 -> who a|m, i?
        let result = surrounding_word_range(content, 5);
        assert_eq!(result, 4..6)
    }

    #[test]
    fn should_return_surrounding_word_range_until_punctuation_japanese() {
        let content = "俺って、誰？";

        // Cursor is at 6 -> 俺っ|て、誰？
        let result = surrounding_word_range(content, 6);
        assert_eq!(result, 0..9)
    }

    #[test]
    fn should_select_preceding_word_when_offset_lands_on_whitespace() {
        let content = "hi there";

        // Cursor is at 2 -> hi| there
        let result = surrounding_word_range(content, 2);
        assert_eq!(result, 0..2)
    }

    #[test]
    fn should_select_last_word_when_offset_is_past_end_of_content() {
        let content = "hello";

        // Cursor is at 5 -> hello|
        let result = surrounding_word_range(content, 5);
        assert_eq!(result, 0..5)
    }

    #[test]
    fn should_collapse_to_offset_when_content_is_empty() {
        let content = "";

        // Cursor is at 0 -> |
        let result = surrounding_word_range(content, 0);
        assert_eq!(result, 0..0)
    }

    #[test]
    fn should_return_word_range_touching_end_of_content() {
        let content = "hello world";

        // Cursor is at 8 -> hello wo|rld
        let result = surrounding_word_range(content, 8);
        assert_eq!(result, 6..11)
    }

    #[test]
    fn should_prefer_word_over_boundary_whitespace() {
        let content = "hello me";

        // Cursor is at 5 -> hello| me
        let result = surrounding_word_range(content, 5);
        assert_eq!(result, 0..5)
    }
}
