//! Reader-visible Markdown projection over a bounded UTF-8 buffer.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use super::{BasicAccumulator, MetricValues, TextAdmission};

/// Fold `CommonMark` events directly into visible word statistics without an owned AST.
#[derive(Debug)]
pub(crate) struct MarkdownAccumulator {
    visible: BasicAccumulator,
    paragraphs: u64,
    suppressed_depth: usize,
    html_hidden: bool,
}

impl MarkdownAccumulator {
    fn new() -> Self {
        Self {
            visible: BasicAccumulator::with_logical_metrics(true),
            paragraphs: 0,
            suppressed_depth: 0,
            html_hidden: false,
        }
    }

    fn push_text(&mut self, text: &str) {
        if self.suppressed_depth == 0 {
            self.visible.push(text.as_bytes());
        }
    }

    fn boundary(&mut self) {
        if self.suppressed_depth == 0 {
            self.visible.push(b" ");
        }
    }

    fn start(&mut self, tag: &Tag<'_>) {
        if self.suppressed_depth > 0 {
            self.suppressed_depth += 1;
            return;
        }
        if matches!(tag, Tag::CodeBlock(_) | Tag::FootnoteDefinition(_) | Tag::MetadataBlock(_)) {
            self.boundary();
            self.suppressed_depth = 1;
            return;
        }
        if matches!(tag, Tag::Paragraph | Tag::Heading { .. }) {
            self.paragraphs = self.paragraphs.saturating_add(1);
        }
        if matches!(tag, Tag::Paragraph | Tag::Heading { .. } | Tag::TableCell) {
            self.boundary();
        }
    }

    fn end(&mut self, tag: TagEnd) {
        if self.suppressed_depth > 0 {
            self.suppressed_depth -= 1;
            if self.suppressed_depth == 0 {
                self.boundary();
            }
        } else if matches!(
            tag,
            TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::BlockQuote(_)
                | TagEnd::HtmlBlock
                | TagEnd::List(_)
                | TagEnd::Item
                | TagEnd::DefinitionList
                | TagEnd::DefinitionListTitle
                | TagEnd::DefinitionListDefinition
                | TagEnd::Table
                | TagEnd::TableHead
                | TagEnd::TableRow
                | TagEnd::TableCell
        ) {
            self.boundary();
        }
    }

    fn push_html(&mut self, html: &str) {
        if self.suppressed_depth > 0 {
            return;
        }
        let bytes = html.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'<' {
                let Some(relative_end) = bytes[index..].iter().position(|byte| *byte == b'>')
                else {
                    break;
                };
                let end = index + relative_end + 1;
                let tag = &bytes[index + 1..end - 1];
                let tag = &tag[tag.iter().take_while(|byte| byte.is_ascii_whitespace()).count()..];
                let closing = tag.first() == Some(&b'/');
                let name = &tag[usize::from(closing)..];
                let name =
                    &name[..name.iter().take_while(|byte| byte.is_ascii_alphabetic()).count()];
                if !self.html_hidden {
                    self.boundary();
                }
                if name.eq_ignore_ascii_case(b"script") || name.eq_ignore_ascii_case(b"style") {
                    self.html_hidden = !closing;
                }
                if !self.html_hidden {
                    self.boundary();
                }
                index = end;
                continue;
            }
            let end = bytes[index..]
                .iter()
                .position(|byte| *byte == b'<')
                .map_or(bytes.len(), |offset| index + offset);
            if !self.html_hidden {
                self.push_text(&html[index..end]);
            }
            index = end;
        }
    }

    fn finish(self) -> MetricValues {
        let TextAdmission::Accepted(visible) = self.visible.finish() else {
            unreachable!("Markdown input was already admitted as UTF-8 text")
        };
        MetricValues {
            visible_words: visible.raw_words,
            visible_logical_word_stats: visible.logical_word_stats,
            paragraphs: self.paragraphs,
            ..MetricValues::default()
        }
    }
}

/// Analyze one already-bounded and UTF-8-admitted Markdown source buffer.
pub(crate) fn analyze_markdown(source: &str) -> MetricValues {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
    let mut accumulator = MarkdownAccumulator::new();
    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(tag) => accumulator.start(&tag),
            Event::End(tag) => accumulator.end(tag),
            Event::Text(text) => accumulator.push_text(&text),
            Event::Html(html) | Event::InlineHtml(html) => accumulator.push_html(&html),
            Event::SoftBreak | Event::HardBreak | Event::Rule => accumulator.boundary(),
            Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::FootnoteReference(_)
            | Event::TaskListMarker(_) => {}
        }
    }
    accumulator.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_reader_text_and_excludes_destinations_code_and_hidden_blocks() {
        let metrics = analyze_markdown(
            r#"---
title: Hidden Frontmatter
---
# Visible heading

Read [the label](https://example.test/very/long/url) and ![image alt](image.png).

`inline code should vanish`

```rust
let fenced = "hidden";
```

<div>wrapped words</div><script>hidden script words</script>

[^note]: hidden footnote words
Reference[^note].
"#,
        );
        assert_eq!(metrics.visible_words, 10);
        assert_eq!(metrics.paragraphs, 3);
        assert_eq!(metrics.visible_logical_word_stats.logical_words(), 10);
    }

    #[test]
    fn table_cells_are_visible_and_reference_definitions_are_not() {
        let source = "| Name | Value |\n| --- | --- |\n| alpha | beta |\n\n[docs]: https://example.test\nSee [docs].\n";
        let metrics = analyze_markdown(source);
        assert_eq!(metrics.visible_words, 6);
        assert_eq!(metrics.paragraphs, 1);
    }

    #[test]
    fn malformed_markdown_remains_bounded_and_deterministic() {
        let source = "Text [with](broken and `unterminated\n<style>hidden";
        assert_eq!(analyze_markdown(source), analyze_markdown(source));
    }
}
