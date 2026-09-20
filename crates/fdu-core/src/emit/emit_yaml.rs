use super::{Event, Scalar, Shape, Sink, write_yaml_scalar};

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
pub(crate) struct YamlSink {
    out: String,
    stack: Vec<Frame>,
}

impl YamlSink {
    pub(crate) fn new() -> Self {
        Self { out: String::new(), stack: Vec::new() }
    }

    fn indent(&mut self, depth: usize) {
        self.out.push_str(&"  ".repeat(depth));
    }

    fn newline_at(&mut self, depth: usize) {
        if !self.out.is_empty() {
            self.out.push('\n');
        }
        self.indent(depth);
    }

    fn before_scalar(&mut self) {
        let Some(index) = self.stack.len().checked_sub(1) else { return };
        match (self.stack[index].shape, self.stack[index].kind) {
            (Shape::Block, Kind::Map) => {
                debug_assert!(self.stack[index].pending_key, "a YAML map value must follow a key");
                self.out.push(' ');
                self.stack[index].pending_key = false;
            }
            (Shape::Block, Kind::Seq) => {
                self.newline_at(index);
                self.out.push_str("- ");
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
                    self.out.push_str(", ");
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
                    self.out.push(' ');
                }
                self.stack[index].pending_key = false;
            }
            (Shape::Block, Kind::Seq) => {
                self.newline_at(index);
                self.out.push('-');
                if child_shape == Shape::Inline {
                    self.out.push(' ');
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
                    self.out.push_str(", ");
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
            Scalar::U64(value) => self.out.push_str(&value.to_string()),
            Scalar::I64(value) => self.out.push_str(&value.to_string()),
            Scalar::Bool(value) => self.out.push_str(if value { "true" } else { "false" }),
            Scalar::Null => self.out.push_str("null"),
        }
    }
}

impl Sink for YamlSink {
    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::BeginMap(shape) => {
                let shape = self.collection_shape(shape);
                self.before_collection(shape);
                if shape == Shape::Inline {
                    self.out.push('{');
                }
                self.stack.push(Frame { kind: Kind::Map, shape, first: true, pending_key: false });
            }
            Event::EndMap => {
                let frame = self.stack.pop().expect("balanced YAML map events");
                debug_assert!(matches!(frame.kind, Kind::Map));
                if frame.shape == Shape::Inline {
                    self.out.push('}');
                } else if frame.first {
                    if self.stack.is_empty() {
                        self.out.push_str("{}");
                    } else {
                        self.out.push_str(" {}");
                    }
                }
            }
            Event::BeginSeq(shape) => {
                let shape = self.collection_shape(shape);
                self.before_collection(shape);
                if shape == Shape::Inline {
                    self.out.push('[');
                }
                self.stack.push(Frame { kind: Kind::Seq, shape, first: true, pending_key: false });
            }
            Event::EndSeq => {
                let frame = self.stack.pop().expect("balanced YAML sequence events");
                debug_assert!(matches!(frame.kind, Kind::Seq));
                if frame.shape == Shape::Inline {
                    self.out.push(']');
                } else if frame.first {
                    if self.stack.is_empty() {
                        self.out.push_str("[]");
                    } else {
                        self.out.push_str(" []");
                    }
                }
            }
            Event::Key(key) => {
                let index = self.stack.len().checked_sub(1).expect("key inside YAML map");
                debug_assert!(matches!(self.stack[index].kind, Kind::Map));
                match self.stack[index].shape {
                    Shape::Block => self.newline_at(index),
                    Shape::Inline if !self.stack[index].first => self.out.push_str(", "),
                    Shape::Inline => {}
                }
                write_yaml_scalar(&mut self.out, key);
                self.out.push(':');
                if self.stack[index].shape == Shape::Inline {
                    self.out.push(' ');
                }
                self.stack[index].first = false;
                self.stack[index].pending_key = true;
            }
            Event::Scalar(scalar) => self.scalar(scalar),
        }
    }

    fn finish(mut self) -> String {
        debug_assert!(self.stack.is_empty(), "all YAML collections must close");
        self.out.push('\n');
        self.out
    }
}
