use super::{Event, Scalar, Shape, Sink, write_json_string};

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
pub(crate) struct JsonSink {
    out: String,
    stack: Vec<Frame>,
    pretty: bool,
}

impl JsonSink {
    pub(crate) fn pretty() -> Self {
        Self { out: String::new(), stack: Vec::new(), pretty: true }
    }

    pub(crate) fn line() -> Self {
        Self { out: String::new(), stack: Vec::new(), pretty: false }
    }

    fn indent(&mut self, depth: usize) {
        self.out.push_str(&"  ".repeat(depth));
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
                    self.out.push(',');
                }
                let block = self.pretty && self.stack[index].shape == Shape::Block;
                if block {
                    self.out.push('\n');
                    let depth = self.stack.len();
                    self.indent(depth);
                } else if !first {
                    self.out.push(' ');
                }
                self.stack[index].first = false;
            }
        }
    }

    fn scalar(&mut self, scalar: Scalar<'_>) {
        self.before_value();
        match scalar {
            Scalar::Str(value) => write_json_string(&mut self.out, value),
            Scalar::U64(value) => self.out.push_str(&value.to_string()),
            Scalar::I64(value) => self.out.push_str(&value.to_string()),
            Scalar::Bool(value) => self.out.push_str(if value { "true" } else { "false" }),
            Scalar::Null => self.out.push_str("null"),
        }
    }
}

impl Sink for JsonSink {
    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::BeginMap(shape) => {
                self.before_value();
                self.out.push('{');
                self.stack.push(Frame { kind: Kind::Map, shape, first: true, has_key: false });
            }
            Event::EndMap => {
                let frame = self.stack.pop().expect("balanced map events");
                debug_assert!(matches!(frame.kind, Kind::Map));
                if !frame.first && self.pretty && frame.shape == Shape::Block {
                    self.out.push('\n');
                    self.indent(self.stack.len());
                }
                self.out.push('}');
            }
            Event::BeginSeq(shape) => {
                self.before_value();
                self.out.push('[');
                self.stack.push(Frame { kind: Kind::Seq, shape, first: true, has_key: false });
            }
            Event::EndSeq => {
                let frame = self.stack.pop().expect("balanced sequence events");
                debug_assert!(matches!(frame.kind, Kind::Seq));
                if !frame.first && self.pretty && frame.shape == Shape::Block {
                    self.out.push('\n');
                    self.indent(self.stack.len());
                }
                self.out.push(']');
            }
            Event::Key(key) => {
                let index = self.stack.len().checked_sub(1).expect("key inside a map");
                debug_assert!(matches!(self.stack[index].kind, Kind::Map));
                let first = self.stack[index].first;
                if !first {
                    self.out.push(',');
                }
                let block = self.pretty && self.stack[index].shape == Shape::Block;
                if block {
                    self.out.push('\n');
                    let depth = self.stack.len();
                    self.indent(depth);
                } else if !first {
                    self.out.push(' ');
                }
                write_json_string(&mut self.out, key);
                self.out.push(':');
                self.out.push(' ');
                self.stack[index].first = false;
                self.stack[index].has_key = true;
            }
            Event::Scalar(scalar) => self.scalar(scalar),
        }
    }

    fn finish(mut self) -> String {
        debug_assert!(self.stack.is_empty(), "all JSON collections must close");
        if self.pretty {
            self.out.push('\n');
        }
        self.out
    }
}
