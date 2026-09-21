use std::fmt;

use super::{Event, IoFmt, Scalar, Shape, Sink, write_json_string};

#[derive(Clone, Copy)]
enum Kind {
    Map,
    Seq,
}

struct Frame {
    kind: Kind,
    shape: Shape,
    first: bool,
    has_key: bool,
}

/// A JSON sink with either declared-layout pretty output or one-line output.
pub(crate) struct JsonSink<W> {
    out: W,
    stack: Vec<Frame>,
    pretty: bool,
}

impl JsonSink<String> {
    pub(crate) fn pretty() -> Self {
        Self { out: String::new(), stack: Vec::new(), pretty: true }
    }

    pub(crate) fn line() -> Self {
        Self { out: String::new(), stack: Vec::new(), pretty: false }
    }
}

impl<'a> JsonSink<IoFmt<'a>> {
    pub(crate) fn pretty_to(writer: &'a mut dyn std::io::Write) -> Self {
        Self { out: IoFmt::new(writer), stack: Vec::new(), pretty: true }
    }

    pub(crate) fn line_to(writer: &'a mut dyn std::io::Write) -> Self {
        Self { out: IoFmt::new(writer), stack: Vec::new(), pretty: false }
    }
}

impl<W: fmt::Write> JsonSink<W> {
    fn push(&mut self, ch: char) {
        let _ = self.out.write_char(ch);
    }

    fn push_str(&mut self, text: &str) {
        let _ = self.out.write_str(text);
    }

    fn indent(&mut self, depth: usize) {
        for _ in 0..depth {
            self.push_str("  ");
        }
    }

    fn before_value(&mut self) {
        let Some(index) = self.stack.len().checked_sub(1) else { return };
        match self.stack[index].kind {
            Kind::Map => {
                debug_assert!(self.stack[index].has_key, "a map value must follow a key");
                self.stack[index].has_key = false;
            }
            Kind::Seq => {
                let first = self.stack[index].first;
                if !first {
                    self.push(',');
                }
                let block = self.pretty && self.stack[index].shape == Shape::Block;
                if block {
                    self.push('\n');
                    let depth = self.stack.len();
                    self.indent(depth);
                } else if !first {
                    self.push(' ');
                }
                self.stack[index].first = false;
            }
        }
    }

    fn scalar(&mut self, scalar: Scalar<'_>) {
        self.before_value();
        match scalar {
            Scalar::Str(value) => write_json_string(&mut self.out, value),
            Scalar::U64(value) => {
                let _ = write!(self.out, "{value}");
            }
            Scalar::I64(value) => {
                let _ = write!(self.out, "{value}");
            }
            Scalar::I128(value) => {
                let _ = write!(self.out, "{value}");
            }
            Scalar::Bool(value) => self.push_str(if value { "true" } else { "false" }),
            Scalar::Null => self.push_str("null"),
        }
    }
}

impl<W: fmt::Write> Sink for JsonSink<W> {
    type Output = W;

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::BeginMap(shape) => {
                self.before_value();
                self.push('{');
                self.stack.push(Frame { kind: Kind::Map, shape, first: true, has_key: false });
            }
            Event::EndMap => {
                let frame = self.stack.pop().expect("balanced map events");
                debug_assert!(matches!(frame.kind, Kind::Map));
                if !frame.first && self.pretty && frame.shape == Shape::Block {
                    self.push('\n');
                    self.indent(self.stack.len());
                }
                self.push('}');
            }
            Event::BeginSeq(shape) => {
                self.before_value();
                self.push('[');
                self.stack.push(Frame { kind: Kind::Seq, shape, first: true, has_key: false });
            }
            Event::EndSeq => {
                let frame = self.stack.pop().expect("balanced sequence events");
                debug_assert!(matches!(frame.kind, Kind::Seq));
                if !frame.first && self.pretty && frame.shape == Shape::Block {
                    self.push('\n');
                    self.indent(self.stack.len());
                }
                self.push(']');
            }
            Event::Key(key) => {
                let index = self.stack.len().checked_sub(1).expect("key inside a map");
                debug_assert!(matches!(self.stack[index].kind, Kind::Map));
                let first = self.stack[index].first;
                if !first {
                    self.push(',');
                }
                let block = self.pretty && self.stack[index].shape == Shape::Block;
                if block {
                    self.push('\n');
                    let depth = self.stack.len();
                    self.indent(depth);
                } else if !first {
                    self.push(' ');
                }
                write_json_string(&mut self.out, key);
                self.push(':');
                self.push(' ');
                self.stack[index].first = false;
                self.stack[index].has_key = true;
            }
            Event::Scalar(scalar) => self.scalar(scalar),
        }
    }

    fn finish(mut self) -> W {
        debug_assert!(self.stack.is_empty(), "all JSON collections must close");
        if self.pretty {
            self.push('\n');
        }
        self.out
    }
}
