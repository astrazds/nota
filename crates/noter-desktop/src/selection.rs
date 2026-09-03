use noter_core::markdown_editing::ByteSelection;

pub fn gtk_character_range_to_byte_selection(
    content: &str,
    start_character: usize,
    end_character: usize,
) -> Option<ByteSelection> {
    let start = character_index_to_byte(content, start_character)?;
    let end = character_index_to_byte(content, end_character)?;
    ByteSelection::new(content, start, end)
}

fn character_index_to_byte(content: &str, character_index: usize) -> Option<usize> {
    if character_index == content.chars().count() {
        return Some(content.len());
    }
    content
        .char_indices()
        .nth(character_index)
        .map(|(byte_index, _)| byte_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtk_character_offsets_become_validated_utf8_byte_ranges() {
        let selection = gtk_character_range_to_byte_selection("A😀B", 1, 2).unwrap();
        assert_eq!(selection.ordered(), (1, 5));
        assert!(gtk_character_range_to_byte_selection("A😀B", 1, 99).is_none());
    }
}
