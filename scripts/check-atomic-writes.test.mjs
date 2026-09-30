import assert from "node:assert/strict";
import test from "node:test";

import {
  auditAtomicWrites,
  auditNode,
  auditPython,
  auditRust,
  pythonStructure,
  scriptStructure,
} from "./check-atomic-writes.mjs";

const NO_POLICY = {
  helpers: new Map(),
  inputs: new Map(),
  exceptions: new Map(),
  mirrors: new Map(),
};

const kinds = (writes) => writes.map((write) => write.kind);

test("finds every raw Rust write outside test code", () => {
  const source = [
    "fn save(path: &Path) {",
    "    fs::write(path, b\"x\").unwrap();",
    "    std::fs::write(path, b\"x\").unwrap();",
    "    let file = File::create(path)?;",
    "    let file = File::create_new(path)?;",
    "    fs::copy(from, path)?;",
    "    OpenOptions::new().append(true).open(path)?;",
    "    OpenOptions::new()",
    "        .write(true)",
    "        .open(path)?;",
    "}",
  ].join("\n");
  assert.deepEqual(kinds(auditRust(source)), [
    "fs::write()",
    "fs::write()",
    "File::create()",
    "File::create()",
    "fs::copy()",
    "a write-mode OpenOptions",
    "a write-mode OpenOptions",
  ]);
});

test("finds a std::fs writer imported by use and called bare", () => {
  const imports = [
    "use std::fs::write;",
    "use std::fs::{self, write};",
    "use std::fs::{File, copy};",
    "use std::{fs::{write, copy}, io};",
    "use std::fs::write as save;",
    "use std::fs::*;",
  ];
  const calls = ["write(path, b\"x\")?;", "copy(from, path)?;", "save(path, b\"x\")?;"];
  const expected = [
    ["fs::write() imported as write()"],
    ["fs::write() imported as write()"],
    ["fs::copy() imported as copy()"],
    ["fs::write() imported as write()", "fs::copy() imported as copy()"],
    ["fs::write() imported as save()"],
    ["fs::write() imported as write()", "fs::copy() imported as copy()"],
  ];
  imports.forEach((line, index) => {
    const body = calls.map((call) => `    ${call}\n`).join("");
    const source = `${line}\nfn store(path: &Path) -> io::Result<()> {\n${body}}\n`;
    assert.deepEqual(kinds(auditRust(source)), expected[index], line);
  });
});

test("does not count a bare write that is not std::fs's", () => {
  const source = [
    "use std::io::Write;",
    "use std::fs::File;",
    "fn write(buffer: &mut Vec<u8>) {}",
    "fn store(out: &mut impl Write) {",
    "    write(&mut buffer);",
    "    write!(out, \"x\")?;",
    "    out.write(b\"x\")?;",
    "    io::copy(&mut from, out)?;",
    "}",
    "#[cfg(test)]",
    "mod tests {",
    "    use std::fs::write;",
    "    fn plant() { write(p, b\"x\").unwrap(); }",
    "}",
  ].join("\n");
  assert.deepEqual(auditRust(source), []);
});

test("finds an OpenOptions whose write, append, or create flag is computed", () => {
  const source = [
    "fn open(path: &Path, append: bool) -> io::Result<File> {",
    "    OpenOptions::new().read(true).append(append).open(path)?;",
    "    std::fs::OpenOptions::new()",
    "        .write(!read_only)",
    "        .create(settings.create)",
    "        .open(path)?;",
    "    File::options().create_new(fresh).open(path)?;",
    "    let mut options = OpenOptions::new();",
    "    options.read(true);",
    "    options.write(writable);",
    "    options.open(path)",
    "}",
  ].join("\n");
  assert.deepEqual(kinds(auditRust(source)), [
    "OpenOptions .append() with a computed flag",
    "OpenOptions .write() with a computed flag",
    "OpenOptions .create() with a computed flag",
    "OpenOptions .create_new() with a computed flag",
    "OpenOptions .write() with a computed flag",
  ]);
});

test("does not count a read-only OpenOptions or another type's write call", () => {
  const source = [
    "fn read(path: &Path) -> io::Result<()> {",
    "    OpenOptions::new().read(true).write(false).open(path)?;",
    "    fs::OpenOptions::new().read(true).custom_flags(flags).open(path)?;",
    "    let options = OpenOptions::new();",
    "    let file = options.read(true).open(path)?;",
    "    buffer.truncate(length);",
    "    lines.append(&mut other);",
    "    output.write(&bytes)?;",
    "}",
    "fn other(options: Settings) { options.write(flag); }",
  ].join("\n");
  assert.deepEqual(auditRust(source), []);
});

test("does not count a read-open or a Rust write named in a comment or string", () => {
  const source = [
    "// fs::write(path, bytes) would tear",
    "/* File::create(path) */",
    'const HINT: &str = "fs::write(path)";',
    'let raw = r#"OpenOptions::new().write(true)"#;',
    "let file = OpenOptions::new().read(true).open(path)?;",
    "let quote = '\"'; let other = fs::read(path)?;",
  ].join("\n");
  assert.deepEqual(auditRust(source), []);
});

test("skips Rust items compiled only for tests", () => {
  const source = [
    "#[cfg(test)]",
    "mod tests {",
    "    fn fixture() { std::fs::write(p, b\"x\").unwrap(); }",
    "}",
    "#[cfg(all(test, feature = \"watch\"))]",
    "#[allow(dead_code)]",
    "fn helper() { std::fs::write(p, b\"x\").unwrap(); }",
    "#[cfg(test)]",
    "static SEED: Lazy<()> = Lazy::new(|| { std::fs::write(p, b\"x\").unwrap(); });",
    "fn production() {}",
  ].join("\n");
  assert.deepEqual(auditRust(source), []);
});

test("audits Rust items that also compile outside tests", () => {
  const attributes = ["#[cfg(not(test))]", "#[cfg(any(test, unix))]", '#[cfg(feature = "watch")]'];
  for (const attribute of attributes) {
    const source = `${attribute}\nfn save() { std::fs::write(p, b"x").unwrap(); }\n`;
    assert.deepEqual(kinds(auditRust(source)), ["fs::write()"], attribute);
  }
});

test("treats a module declared under cfg(test) as test code", () => {
  const sources = new Map([
    ["crates/x/src/lib.rs", "#[cfg(test)]\nmod test_support;\nmod store;\n"],
    ["crates/x/src/test_support.rs", 'pub fn plant() { std::fs::write(p, b"x").unwrap(); }'],
    ["crates/x/src/store.rs", "#[cfg(all(test, unix))]\nmod golden;\n"],
    ["crates/x/src/store/golden.rs", 'fn plant() { std::fs::write(p, b"x").unwrap(); }'],
  ]);
  assert.deepEqual(auditAtomicWrites(sources, NO_POLICY).problems, []);
});

test("finds every raw Python write", () => {
  const source = [
    'path.write_text("x", encoding="utf-8")',
    'path.write_bytes(b"x")',
    'with open(path, "w") as handle: pass',
    'with open(path, mode="a") as handle: pass',
    'with open(path, "r+b") as handle: pass',
    'with path.open("x") as handle: pass',
    'with gzip.open(path, "wt") as handle: pass',
    'with tarfile.open(path, "w:gz") as archive: pass',
    'with os.fdopen(descriptor, "w") as handle: pass',
    "json.dump(value, handle)",
    "shutil.copy2(source, destination)",
    "descriptor = os.open(path, os.O_WRONLY | os.O_CREAT)",
    "with open(path, mode) as handle: pass",
  ].join("\n");
  assert.deepEqual(kinds(auditPython(source)), [
    ".write_text()",
    ".write_bytes()",
    "json.dump() to a file",
    "shutil.copy2()",
    'open() with mode "w"',
    'open() with mode "a"',
    'open() with mode "r+b"',
    'path.open() with mode "x"',
    'gzip.open() with mode "wt"',
    'tarfile.open() with mode "w:gz"',
    'os.fdopen() with mode "w"',
    "os.open() for writing",
    "open() with a computed mode",
  ]);
});

test("does not count Python reads, streams, definitions, comments, or strings", () => {
  const source = [
    "with open(path) as handle: pass",
    'with open(path, "rb") as handle: pass',
    'with path.open(encoding="utf-8") as handle: pass',
    'with tarfile.open(path, "r:gz") as archive: pass',
    "descriptor = os.open(path, os.O_RDONLY)",
    'with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as out: pass',
    "json.dump(value, sys.stdout, indent=2)",
    "webbrowser.open(url)",
    "def open(root, *, cache=None): pass",
    "    def open(cls, root, options=None): pass",
    '# path.write_text("x") would tear',
    '"""Call path.write_text(text) or open(path, "w")."""',
    "hint = 'json.dump(value, handle)'",
  ].join("\n");
  assert.deepEqual(auditPython(source), []);
});

test("reads builtins.open like the builtin", () => {
  const source = [
    'with builtins.open(path, "w") as handle: pass',
    'with builtins.open("x", "w") as handle: pass',
    "with builtins.open(path, mode) as handle: pass",
    "with builtins.open(path) as handle: pass",
    'with builtins.open("x", "rb") as handle: pass',
  ].join("\n");
  assert.deepEqual(kinds(auditPython(source)), [
    'builtins.open() with mode "w"',
    'builtins.open() with mode "w"',
    "builtins.open() with a computed mode",
  ]);
});

test("finds Python opens whose flags or mode are computed", () => {
  const source = [
    "descriptor = os.open(path, flags)",
    "descriptor = os.open(path, os.O_RDONLY | extra)",
    "descriptor = os.open(path, flags=os.O_WRONLY)",
    "descriptor = os.open(path, os.O_WRONLY | extra)",
    "with path.open(mode) as handle: pass",
    'with path.with_suffix(".x").open(mode, encoding="utf-8") as handle: pass',
  ].join("\n");
  assert.deepEqual(kinds(auditPython(source)), [
    "os.open() with computed flags",
    "os.open() with computed flags",
    "os.open() for writing",
    "os.open() for writing",
    "path.open() with a computed mode",
    ".open() with a computed mode",
  ]);
});

test("does not count Python opens whose flags are literal reads", () => {
  const source = [
    "descriptor = os.open(path, os.O_RDONLY)",
    "descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)",
    "descriptor = os.open(path, (O_RDONLY | O_DIRECTORY), dir_fd=parent)",
    "descriptor = os.open(path, 0)",
    "with fdu.open(root, options) as opened: pass",
    "webbrowser.open(url, new)",
  ].join("\n");
  assert.deepEqual(auditPython(source), []);
});

test("does not count a dump into a handle open_atomic bound", () => {
  const source = [
    'with open_atomic(path, "w", encoding="utf-8") as output:',
    "    json.dump(value, output, indent=2)",
    'with open_atomic(first) as a, atomic_write.open_atomic(second, "wb") as b:',
    "    pickle.dump(value, b)",
    "    json.dump(value, handle)",
  ].join("\n");
  assert.deepEqual(kinds(auditPython(source)), ["json.dump() to a file"]);
});

test("keeps Python structure aligned with the source", () => {
  const source = 'a = rb"\\"x"  # c\nb = f"""{x}\n"""\nc = \'open(p, "w")\'\n';
  const { code, literals } = pythonStructure(source);
  assert.equal(code.length, source.length);
  assert.equal(code.split("\n").length, source.split("\n").length);
  assert.deepEqual(
    literals.map((literal) => literal.value),
    ['\\"x', "{x}\n", 'open(p, "w")'],
  );
});

test("finds every raw Node write", () => {
  const source = [
    'import { writeFileSync, createWriteStream } from "node:fs";',
    "writeFileSync(path, text);",
    "fs.writeFileSync(path, text);",
    "await writeFile(path, text);",
    "appendFileSync(path, line);",
    "const stream = createWriteStream(path);",
    "copyFileSync(from, path);",
    'const fd = openSync(path, "w");',
    "const handle = await open(path, 'a+');",
    "const fd2 = openSync(path, flags);",
  ].join("\n");
  assert.deepEqual(kinds(auditNode(source)), [
    "writeFileSync()",
    "writeFileSync()",
    "writeFile()",
    "appendFileSync()",
    "createWriteStream()",
    "copyFileSync()",
    'openSync() with flags "w"',
    'open() with flags "a+"',
    "openSync() with computed flags",
  ]);
});

test("finds a Node writer called under an alias", () => {
  const source = [
    'import { readFileSync, writeFileSync as save, createWriteStream as stream } from "node:fs";',
    'import fs, { writeFile as put } from "node:fs/promises";',
    'const { appendFileSync: log, readFileSync: read } = require("node:fs");',
    "const {",
    "  appendFile: logLater,",
    '} = await import("node:fs/promises");',
    "save(path, text);",
    "const out = stream(path);",
    "await put(path, text);",
    "log(path, line);",
    "await logLater(path, line);",
    "read(path);",
  ].join("\n");
  assert.deepEqual(kinds(auditNode(source)), [
    "writeFileSync() imported as save()",
    "createWriteStream() imported as stream()",
    "writeFile() imported as put()",
    "appendFileSync() imported as log()",
    "appendFile() imported as logLater()",
  ]);
});

test("does not count an alias that is only defined or named", () => {
  const source = [
    'import { writeFileSync as save } from "node:fs";',
    "function save(path) { return path; }",
    "object.save(path);",
    "const hint = 'save(path)';",
    "const options = { writeFileSync: fake };",
    "fake(path);",
  ].join("\n");
  assert.deepEqual(auditNode(source), []);
});

test("does not count Node reads, comments, strings, templates, or regular expressions", () => {
  const source = [
    'const fd = openSync(path, "r");',
    "// writeFileSync(path, text) would tear",
    "/* appendFileSync(path) */",
    "const hint = \"writeFileSync(path, text)\";",
    "const command = `node -e \"writeFileSync('x')\"`;",
    "const pattern = /writeFileSync\\(/g;",
    "const ratio = total / count; const other = a / b;",
  ].join("\n");
  assert.deepEqual(auditNode(source), []);
});

test("sees code inside template holes and after regular expressions", () => {
  const source = [
    "const message = `saved ${writeFileSync(path, `${text}`)} bytes`;",
    'const quote = /"/; writeFileSync(path, text);',
  ].join("\n");
  assert.deepEqual(kinds(auditNode(source)), ["writeFileSync()", "writeFileSync()"]);
  const { code } = scriptStructure(source);
  assert.equal(code.length, source.length);
});

test("exempts helpers, test code, and input writers, and reports the rest", () => {
  const write = 'writeFileSync(path, "x");';
  const sources = new Map([
    ["scripts/atomic-write.mjs", write],
    ["scripts/tool.test.mjs", write],
    ["tests/release/test_tool.py", 'path.write_text("x")'],
    ["crates/fdu/tests/cli.rs", 'fn t() { std::fs::write(p, b"x").unwrap(); }'],
    ["tests/golden/fixtures/code-project/main.py", 'open("out", "w")'],
    ["explorations/benchmarks/tests/fixtures/fake.py", 'open("out", "w")'],
    ["tests/golden/bin/plant.mjs", write],
    ["scripts/tool.mjs", `const a = 1;\n${write}`],
  ]);
  const policy = {
    ...NO_POLICY,
    helpers: new Map([["scripts/atomic-write.mjs", "the helper"]]),
    inputs: new Map([["tests/golden/bin/plant.mjs", "plants inputs"]]),
  };
  const { problems } = auditAtomicWrites(sources, policy);
  assert.equal(problems.length, 1);
  assert.match(problems[0], /^scripts\/tool\.mjs:2: writeFileSync\(\) writes in place/);
});

test("an exception excuses only its own site, and a stale one fails", () => {
  const sources = new Map([
    ["scripts/tool.py", 'with path.open("a") as out:\n    pass\npath.write_text("x")\n'],
  ]);
  const exceptions = new Map([
    [
      "scripts/tool.py",
      [
        { site: 'path.open("a")', reason: "append protocol" },
        { site: "gone.write_text(", reason: "no longer here" },
      ],
    ],
  ]);
  const { problems } = auditAtomicWrites(sources, { ...NO_POLICY, exceptions });
  assert.equal(problems.length, 2);
  assert.match(problems[0], /scripts\/tool\.py:3: \.write_text\(\) writes in place/);
  assert.match(problems[1], /exception for `gone\.write_text\(` matches no write/);
});

test("a listed file must exist and a helper copy must match its original", () => {
  const sources = new Map([
    ["scripts/atomic_write.py", "def write(): ...\n"],
    ["explorations/benchmarks/atomic_write.py", "def write(): ...  # edited\n"],
  ]);
  const policy = {
    ...NO_POLICY,
    helpers: new Map([
      ["scripts/atomic_write.py", "the helper"],
      ["scripts/gone.py", "removed"],
    ]),
    mirrors: new Map([["explorations/benchmarks/atomic_write.py", "scripts/atomic_write.py"]]),
  };
  const { problems } = auditAtomicWrites(sources, policy);
  assert.deepEqual(problems, [
    "scripts/gone.py: listed in the atomic-write policy but missing",
    "explorations/benchmarks/atomic_write.py: differs from scripts/atomic_write.py; " +
      "copy the helper again",
  ]);
});
