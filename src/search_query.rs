use crate::model::Note;
use crate::tag_rules::{fold_case, note_has_active_tag};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    terms: Vec<SearchTerm>,
}

impl SearchQuery {
    pub fn parse(raw: &str) -> Self {
        let mut parser = QueryParser::new(raw);
        parser.parse()
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn matches(&self, note: &Note) -> bool {
        self.terms.iter().all(|term| term.matches(note))
    }

    pub fn title_highlight_terms(&self) -> Vec<&str> {
        self.terms
            .iter()
            .filter_map(|term| match term {
                SearchTerm::AnyText(pattern) | SearchTerm::Title(pattern) => {
                    Some(pattern.original.as_str())
                }
                SearchTerm::Tag(_) | SearchTerm::IsPinned => None,
            })
            .collect()
    }

    pub fn preview_highlight_terms(&self) -> Vec<&str> {
        self.terms
            .iter()
            .filter_map(|term| match term {
                SearchTerm::AnyText(pattern) => Some(pattern.original.as_str()),
                SearchTerm::Title(_) | SearchTerm::Tag(_) | SearchTerm::IsPinned => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SearchTerm {
    AnyText(TextPattern),
    Title(TextPattern),
    Tag(String),
    IsPinned,
}

impl SearchTerm {
    fn matches(&self, note: &Note) -> bool {
        match self {
            SearchTerm::AnyText(pattern) => {
                pattern.contains_in(&note.title)
                    || pattern.contains_in(&note.content)
                    || note.tags.iter().any(|tag| pattern.contains_in(tag))
            }
            SearchTerm::Title(pattern) => pattern.contains_in(&note.title),
            SearchTerm::Tag(tag) => note_has_active_tag(note, tag),
            SearchTerm::IsPinned => note.is_pinned,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextPattern {
    original: String,
    folded: String,
}

impl TextPattern {
    fn new(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }

        Some(Self {
            original: trimmed.to_string(),
            folded: fold_case(trimmed),
        })
    }

    fn contains_in(&self, text: &str) -> bool {
        fold_case(text).contains(&self.folded)
    }
}

struct QueryParser<'a> {
    raw: &'a str,
    cursor: usize,
    terms: Vec<SearchTerm>,
    plain: String,
}

impl<'a> QueryParser<'a> {
    fn new(raw: &'a str) -> Self {
        Self {
            raw,
            cursor: 0,
            terms: Vec::new(),
            plain: String::new(),
        }
    }

    fn parse(&mut self) -> SearchQuery {
        while self.cursor < self.raw.len() {
            if self.starts_scoped_term("title:") {
                self.flush_plain();
                self.cursor += "title:".len();
                if let Some(value) = self.parse_value()
                    && let Some(pattern) = TextPattern::new(&value)
                {
                    self.terms.push(SearchTerm::Title(pattern));
                }
            } else if self.starts_scoped_term("tag:") {
                self.flush_plain();
                self.cursor += "tag:".len();
                if let Some(value) = self.parse_value() {
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        self.terms.push(SearchTerm::Tag(trimmed.to_string()));
                    }
                }
            } else if self.starts_scoped_term("is:") {
                self.flush_plain();
                self.cursor += "is:".len();
                if let Some(value) = self.parse_value()
                    && fold_case(value.trim()) == "pinned"
                {
                    self.terms.push(SearchTerm::IsPinned);
                }
            } else if self.current_char() == Some('"') {
                let quoted = self.parse_quoted_value();
                self.plain.push_str(&quoted);
            } else {
                self.push_current_char();
            }
        }

        self.flush_plain();
        SearchQuery {
            terms: std::mem::take(&mut self.terms),
        }
    }

    fn starts_scoped_term(&self, prefix: &str) -> bool {
        self.is_token_boundary()
            && self.raw[self.cursor..].starts_with(prefix)
            && self.raw[self.cursor + prefix.len()..]
                .chars()
                .next()
                .is_some_and(|ch| !ch.is_whitespace())
    }

    fn is_token_boundary(&self) -> bool {
        if self.cursor == 0 {
            return true;
        }

        self.raw[..self.cursor]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
    }

    fn parse_value(&mut self) -> Option<String> {
        match self.current_char()? {
            '"' => Some(self.parse_quoted_value()),
            _ => {
                let start = self.cursor;
                while self.cursor < self.raw.len() {
                    if self.current_char().is_some_and(char::is_whitespace) {
                        break;
                    }
                    self.advance_current_char();
                }
                Some(self.raw[start..self.cursor].to_string())
            }
        }
    }

    fn parse_quoted_value(&mut self) -> String {
        self.cursor += '"'.len_utf8();
        let start = self.cursor;

        while self.cursor < self.raw.len() {
            if self.current_char() == Some('"') {
                let value = self.raw[start..self.cursor].to_string();
                self.cursor += '"'.len_utf8();
                return value;
            }
            self.advance_current_char();
        }

        self.raw[start..].to_string()
    }

    fn flush_plain(&mut self) {
        if let Some(pattern) = TextPattern::new(&self.plain) {
            self.terms.push(SearchTerm::AnyText(pattern));
        }
        self.plain.clear();
    }

    fn current_char(&self) -> Option<char> {
        self.raw[self.cursor..].chars().next()
    }

    fn push_current_char(&mut self) {
        if let Some(ch) = self.current_char() {
            self.plain.push(ch);
            self.cursor += ch.len_utf8();
        }
    }

    fn advance_current_char(&mut self) {
        if let Some(ch) = self.current_char() {
            self.cursor += ch.len_utf8();
        }
    }
}
