use std::fmt;

use super::{Event, IoFmt, Scalar, Shape, Sink, write_yaml_scalar};

#[derive(Clone, Copy)]
enum Kind {
    Map,
    Seq,
}

struct Frame {
    kind: Kind,
    shape: Shape,
    first: bool,
    pending_key: bool,
}

/// An incremental YAML sink with depth-only structural state.
pub(crate) struct YamlSink<W> {
    out: W,
    stack: Vec<Frame>,
    started: bool,
}

impl YamlSink<String> {
    pub(crate) fn new() -> Self {
        Self { out: String::new(), stack: Vec::new(), started: false }
    }
}

impl<'a> YamlSink<IoFmt<'a>> {
    pub(crate) fn to(writer: &'a mut dyn std::io::Write) -> Self {
        Self { out: IoFmt::new(writer), stack: Vec::new(), started: false }
    }
}

impl<W: fmt::Write> YamlSink<W> {
    fn push(&mut self, ch: char) {
        let _ = self.out.write_char(ch);
        self.started = true;
    }

    fn push_str(&mut self, text: &str) {
        let _ = self.out.write_str(text);
        self.started |= !text.is_empty();
    }

    fn indent(&mut self, depth: usize) {
        for _ in 0..depth {
            self.push_str("  ");
        }
    }

    fn newline_at(&mut self, depth: usize) {
        if self.started {
            self.push('\n');
        }
        self.indent(depth);
    }

    fn before_scalar(&mut self) {
        let Some(index) = self.stack.len().checked_sub(1) else { return };
        match (self.stack[index].shape, self.stack[index].kind) {
            (Shape::Block, Kind::Map) => {
                debug_assert!(self.stack[index].pending_key, "a YAML map value must follow a key");
                self.push(' ');
                self.stack[index].pending_key = false;
            }
            (Shape::Block, Kind::Seq) => {
                self.newline_at(index);
                self.push_str("- ");
                self.stack[index].first = false;
            }
            (Shape::Inline, Kind::Map) => {
                debug_assert!(
                    self.stack[index].pending_key,
                    "a YAML flow-map value must follow a key"
                );
                self.stack[index].pending_key = false;
            }
            (Shape::Inline, Kind::Seq) => {
                if !self.stack[index].first {
                    self.push_str(", ");
                }
                self.stack[index].first = false;
            }
        }
    }

    fn before_collection(&mut self, child_shape: Shape) {
        let Some(index) = self.stack.len().checked_sub(1) else { return };
        match (self.stack[index].shape, self.stack[index].kind) {
            (Shape::Block, Kind::Map) => {
                debug_assert!(self.stack[index].pending_key, "a YAML map value must follow a key");
                if child_shape == Shape::Inline {
                    self.push(' ');
                }
                self.stack[index].pending_key = false;
            }
            (Shape::Block, Kind::Seq) => {
                self.newline_at(index);
                self.push('-');
                if child_shape == Shape::Inline {
                    self.push(' ');
                }
                self.stack[index].first = false;
            }
            (Shape::Inline, Kind::Map) => {
                debug_assert!(
                    self.stack[index].pending_key,
                    "a YAML flow-map value must follow a key"
                );
                self.stack[index].pending_key = false;
            }
            (Shape::Inline, Kind::Seq) => {
                if !self.stack[index].first {
                    self.push_str(", ");
                }
                self.stack[index].first = false;
            }
        }
    }

    fn collection_shape(&self, requested: Shape) -> Shape {
        if self.stack.last().is_some_and(|parent| parent.shape == Shape::Inline) {
            Shape::Inline
        } else {
            requested
        }
    }

    fn scalar(&mut self, scalar: Scalar<'_>) {
        self.before_scalar();
        match scalar {
            Scalar::Str(value) => write_yaml_scalar(&mut self.out, value),
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

impl<W: fmt::Write> Sink for YamlSink<W> {
    type Output = W;

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::BeginMap(shape) => {
                let shape = self.collection_shape(shape);
                self.before_collection(shape);
                if shape == Shape::Inline {
                    self.push('{');
                }
                self.stack.push(Frame { kind: Kind::Map, shape, first: true, pending_key: false });
            }
            Event::EndMap => {
                let frame = self.stack.pop().expect("balanced YAML map events");
                debug_assert!(matches!(frame.kind, Kind::Map));
                if frame.shape == Shape::Inline {
                    self.push('}');
                } else if frame.first {
                    if self.stack.is_empty() {
                        self.push_str("{}");
                    } else {
                        self.push_str(" {}");
                    }
                }
            }
            Event::BeginSeq(shape) => {
                let shape = self.collection_shape(shape);
                self.before_collection(shape);
                if shape == Shape::Inline {
                    self.push('[');
                }
                self.stack.push(Frame { kind: Kind::Seq, shape, first: true, pending_key: false });
            }
            Event::EndSeq => {
                let frame = self.stack.pop().expect("balanced YAML sequence events");
                debug_assert!(matches!(frame.kind, Kind::Seq));
                if frame.shape == Shape::Inline {
                    self.push(']');
                } else if frame.first {
                    if self.stack.is_empty() {
                        self.push_str("[]");
                    } else {
                        self.push_str(" []");
                    }
                }
            }
            Event::Key(key) => {
                let index = self.stack.len().checked_sub(1).expect("key inside YAML map");
                debug_assert!(matches!(self.stack[index].kind, Kind::Map));
                match self.stack[index].shape {
                    Shape::Block => self.newline_at(index),
                    Shape::Inline if !self.stack[index].first => self.push_str(", "),
                    Shape::Inline => {}
                }
                write_yaml_scalar(&mut self.out, key);
                self.push(':');
                if self.stack[index].shape == Shape::Inline {
                    self.push(' ');
                }
                self.stack[index].first = false;
                self.stack[index].pending_key = true;
            }
            Event::Scalar(scalar) => self.scalar(scalar),
        }
    }

    fn finish(mut self) -> W {
        debug_assert!(self.stack.is_empty(), "all YAML collections must close");
        self.push('\n');
        self.out
    }
}
