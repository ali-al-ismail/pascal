/// This module provides syntax highlighting for the application.
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

pub struct HighlightedSegment {
    pub content: String,
    pub style: Style,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme: Self::get_theme(),
        }
    }

    fn get_theme() -> Theme {
        let mut theme = ThemeSet::load_defaults().themes["base16-eighties.dark"].clone();
        theme.settings.foreground = Some(syntect::highlighting::Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        });
        theme
    }

    pub fn highlight_line(&self, line: &str, extension: &str) -> Vec<HighlightedSegment> {
        // get syntax from extension
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let highlighted_segment = highlighter
            .highlight_line(line, &self.syntax_set)
            .unwrap_or_else(|_| vec![(Style::default(), line)]);

        // convert to highlighted segment type
        highlighted_segment
            .into_iter()
            .map(|(style, content)| HighlightedSegment {
                content: content.to_string(),
                style,
            })
            .collect()
    }
}
