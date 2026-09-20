//! Dependency-free machine-output sinks shared by reports, streams, and cache status.

mod emit_json;
mod emit_scalar;
mod emit_yaml;

pub(crate) use emit_json::JsonSink;
pub(crate) use emit_scalar::{write_json_string, write_yaml_scalar};
pub(crate) use emit_yaml::YamlSink;

/// The intended presentation of a collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Shape {
    Block,
    Inline,
}

/// A scalar value accepted by every machine-output sink.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Scalar<'a> {
    Str(&'a str),
    U64(u64),
    I64(i64),
    Bool(bool),
    Null,
}

/// One structural event in a machine document.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Event<'a> {
    BeginMap(Shape),
    EndMap,
    BeginSeq(Shape),
    EndSeq,
    Key(&'static str),
    Scalar(Scalar<'a>),
}

/// A serializer for the closed machine-output value vocabulary.
pub(crate) trait Sink {
    fn event(&mut self, event: Event<'_>);
    fn finish(self) -> String
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(mut sink: impl Sink) -> String {
        for event in [
            Event::BeginMap(Shape::Block),
            Event::Key("items"),
            Event::BeginSeq(Shape::Block),
            Event::BeginMap(Shape::Block),
            Event::Key("name"),
            Event::Scalar(Scalar::Str("a { b [ c")),
            Event::EndMap,
            Event::EndSeq,
            Event::Key("empty"),
            Event::BeginMap(Shape::Inline),
            Event::EndMap,
            Event::EndMap,
        ] {
            sink.event(event);
        }
        sink.finish()
    }

    #[test]
    fn line_json_preserves_braces_and_brackets_inside_strings() {
        assert_eq!(sample(JsonSink::line()), r#"{"items": [{"name": "a { b [ c"}], "empty": {}}"#);
    }

    #[test]
    fn yaml_uses_block_collections_and_inline_hints() {
        assert_eq!(sample(YamlSink::new()), "items:\n  -\n    name: \"a { b [ c\"\nempty: {}\n");
    }

    #[test]
    fn yaml_keeps_nested_collections_inside_a_flow_parent_in_flow_style() {
        let mut sink = YamlSink::new();
        for event in [
            Event::BeginMap(Shape::Inline),
            Event::Key("outer"),
            Event::BeginMap(Shape::Block),
            Event::Key("items"),
            Event::BeginSeq(Shape::Block),
            Event::Scalar(Scalar::Str("one")),
            Event::EndSeq,
            Event::EndMap,
            Event::EndMap,
        ] {
            sink.event(event);
        }
        assert_eq!(sink.finish(), "{outer: {items: [one]}}\n");
    }
}
