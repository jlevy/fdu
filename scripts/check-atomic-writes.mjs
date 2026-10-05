#!/usr/bin/env node

// Every file written for something else to read later is written whole: to a temporary
// in the same directory, flushed and synced, then renamed over the target, with the
// temporary removed on any failure. A reader -- a later run, another tool, CI, a
// reviewer -- then sees the old file or the new one, never a torn one under the final
// name, even after a crash or a container restart mid-write. An append-only file (a log,
// a JSONL stream) cannot be renamed into place a record at a time, so its reader must
// detect and drop a torn last record instead. The rule is stated in
// docs/project/architecture/fdu-design-principles.md; this check keeps new code to it.
//
// A raw write is allowed in only three places: an approved helper, which is the atomic
// implementation; test code and the other input writers listed below, whose writes are
// the input a test or an experiment observes; and a listed exception that says why the
// write needs no helper. Everything else fails, so a new writer either goes through a
// helper or is argued for here.
//
// Inputs are exempt because nothing reads them once the process that wrote them exits:
// a crash fails the test or the trial that owned them, and an experiment's write is often
// the very change under study, which a rename would replace with a different one. How a
// freshly written fixture settles before a test measures it is a separate question,
// settled by fdu-tq70.

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, posix } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { rustStructure } from "./check-admission-sites.mjs";

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));

// The atomic implementations. Raw writes inside them are the mechanism.
export const HELPERS = new Map([
  [
    "crates/fdu-core/src/snapshot.rs",
    "write_atomically and replace_atomically: a sibling temporary created exclusively, " +
      "fsync, rename, and stale-temporary reaping; touch() only moves an mtime",
  ],
  [
    "crates/fdu/src/skill_install.rs",
    "stages the skill file beside its target, syncs it, and renames it into place",
  ],
  ["scripts/atomic_write.py", "the shared Python helper for the repository's tooling"],
  ["scripts/atomic-write.mjs", "the shared Node helper for the repository's tooling"],
  [
    "packages/cli-animate/src/fsutil.ts",
    "cli-animate's own whole-file writer: a sibling temporary, fsync, rename; the package " +
      "is self-contained so it can be extracted",
  ],
]);

// Test code, recognised by name or place. Its writes are the inputs of the test.
const TEST_FILE = /(?:^|\/)(?:test_[^/]*\.py|[^/]*_test\.py|conftest\.py|[^/]*\.test\.m?[jt]s)$/;
const TEST_DIRECTORIES = [
  // Rust integration tests, and the Python binding's tests and smoke scripts.
  /^crates\/[^/]+\/(?:tests|benches)\//,
  // Trees the golden corpus scans. They are data, not code that runs.
  /^tests\/golden\/fixtures\//,
  // Programs a test runs in place of the real one.
  /\/tests\/fixtures\//,
];

// Research workloads, whose writes are the filesystem changes an experiment observes.
const CATALOG = "each write is a change the change-source catalog observes";
const DIRSTATS = "each write is a change whose directory statistics the study verifies";
const REPLAY = "each write is a change FSEvents replay must report, in the probe or its tests";

// Test support and experiment workloads outside those names: every raw write left in
// these files builds or changes the input a test or experiment observes. Their results go
// through a helper.
export const INPUT_WRITERS = new Map([
  ["scripts/check-yaml.mjs", "builds the tree the YAML self-check scans"],
  [
    "explorations/benchmarks/corpus_cache.py",
    "materializes a trial's scan tree from a verified base, in a run directory the " +
      "trial removes",
  ],
  ["explorations/benchmarks/spikes/gen_tree.py", "generates a benchmark scan tree"],
  ["tests/correctness/build_tree.py", "builds the correctness runbook's scan tree"],
  ["tests/path_independence/fixture.py", "builds and copies the matrix's scan tree"],
  [
    "tests/path_independence/matrix.py",
    "mutates the scan tree; an in-place rewrite is the change under test",
  ],
  ["tests/golden/bin/cache-plant.mjs", "plants damaged cache files for a golden to reject"],
  ["tests/golden/bin/directory-builds.cjs", "builds the tree a golden scans"],
  ["tests/golden/bin/watch-capture.mjs", "each write is a change the watch golden observes"],
  [
    "tests/golden/bin/watch-repaint-capture.mjs",
    "each write is a change the watch golden observes",
  ],
  ["explorations/change-sources/catalog/src/complete.py", CATALOG],
  ["explorations/change-sources/catalog/src/dirstats.py", CATALOG],
  ["explorations/change-sources/catalog/src/dirstats2.py", CATALOG],
  ["explorations/change-sources/catalog/src/nested_wa.py", CATALOG],
  ["explorations/change-sources/catalog/src/wa_bench.py", CATALOG],
  ["explorations/change-sources/dirstats-verify/src/accounting.py", DIRSTATS],
  ["explorations/change-sources/dirstats-verify/src/bench_wa.py", DIRSTATS],
  ["explorations/change-sources/dirstats-verify/src/dmg_tests.py", DIRSTATS],
  ["explorations/change-sources/dirstats-verify/src/flag_unset.py", DIRSTATS],
  ["explorations/change-sources/dirstats-verify/src/hardlink_primary.py", DIRSTATS],
  ["explorations/change-sources/dirstats-verify/src/ow_lag.py", DIRSTATS],
  ["explorations/change-sources/dirstats-verify/src/refresh.py", DIRSTATS],
  ["explorations/change-sources/dirstats-verify/src/scale_build.py", "builds the scale tree"],
  ["explorations/change-sources/dirstats-verify/src/semantics.py", DIRSTATS],
  [
    "explorations/change-sources/harness-audit/close_order.py",
    "each write is a change whose close-order events the audit observes",
  ],
  ["explorations/fsevents-replay/fixture_workload.py", REPLAY],
  ["explorations/fsevents-replay/open_writer.py", REPLAY],
  ["explorations/fsevents-replay/real_tree.py", REPLAY],
  ["explorations/fsevents-replay/run.py", REPLAY],
]);

// A child's output captured in scratch, read back once it exits, and removed.
const CHILD_CAPTURE =
  "captures a child's output in a scratch directory; this process reads it after the " +
  "child exits, and the directory is removed";

// A kernel control, not a file anything reads back.
const DROP_CACHES = "writes the kernel's /proc/sys/vm/drop_caches control";

// An append-only record stream, whose readers drop a torn last record.
const JSONL =
  "appends one JSON record per line; its readers use complete_lines, which drops a torn " +
  "last record";

// The Actions runner creates $GITHUB_OUTPUT, and reads it once the step ends.
const GITHUB_OUTPUT =
  "appends to $GITHUB_OUTPUT, the runner's file; a torn write fails its step, and no " +
  "release.yml step runs after a failure to read it";

// Individual writes that need no helper, each matched by the text of its line. An entry
// that no longer matches a write fails, so the list cannot outlive the code it excuses.
export const EXCEPTIONS = new Map([
  [
    "crates/fdu-core/build.rs",
    [
      {
        site: "fs::write(output.join(GENERATED_NAME), generated)",
        reason:
          "Cargo reruns a build script that did not finish, so OUT_DIR is never read " +
          "after a torn write",
      },
    ],
  ],
  [
    "explorations/change-sources/replay-cost/src/drive.py",
    [{ site: 'with open(out, "a") as f:', reason: JSONL }],
  ],
  [
    "explorations/change-sources/writer-coverage/src/ops.py",
    [
      { site: "os.open(", reason: "each writer is a change the coverage study observes" },
      { site: 'with open(args.out, "a") as out:', reason: JSONL },
    ],
  ],
  [
    "explorations/benchmarks/corpus.py",
    [
      { site: 'with path.open("xb") as output:', reason: "creates a file of the scan tree" },
      {
        site: 'with path.open("r+b") as output:',
        reason: "rewrites a corpus file in place; the in-place change is the mutation",
      },
    ],
  ],
  [
    "explorations/benchmarks/runner.py",
    [
      {
        site: 'snapshot_path.write_bytes(b"fdu-invalid-snapshot',
        reason: "plants the corrupt snapshot a scenario declares as its input",
      },
      { site: 'stderr_path.open("xb")', reason: CHILD_CAPTURE },
      { site: 'stdout_path.open("xb")', reason: CHILD_CAPTURE },
    ],
  ],
  [
    "explorations/benchmarks/realtree/floor.py",
    [{ site: 'with out_path.open("xb") as out, err_path.open("xb")', reason: CHILD_CAPTURE }],
  ],
  [
    "explorations/benchmarks/realtree/measure.py",
    [
      { site: 'with out_path.open("xb") as out, err_path.open("xb")', reason: CHILD_CAPTURE },
      { site: 'drop.write_text("3\\n", encoding="ascii")', reason: DROP_CACHES },
    ],
  ],
  [
    "explorations/benchmarks/realtree/profile.py",
    [{ site: 'with stdout_path.open("xb") as stdout_file, stderr_path', reason: CHILD_CAPTURE }],
  ],
  [
    "explorations/benchmarks/spikes/code_analysis_pair.py",
    [{ site: "path.write_bytes(body)", reason: "builds the spike's scan tree" }],
  ],
  [
    "explorations/benchmarks/spikes/paired_runner.py",
    [{ site: 'open("/proc/sys/vm/drop_caches", "w")', reason: DROP_CACHES }],
  ],
  [
    "scripts/qa/pty_probe.py",
    [
      {
        site: "os.dup2(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC), fd)",
        reason: CHILD_CAPTURE,
      },
    ],
  ],
  [
    "scripts/qa_peer_agreement.py",
    [{ site: "(probe / ", reason: "builds the self-test's scan tree" }],
  ],
  [
    "scripts/run_installed_cli_qa.py",
    [
      {
        site: "(watch_root / ",
        reason: "the watched tree; each write is a change the watch run must report",
      },
      {
        site: 'with out_path.open("wb") as out, err_path.open("wb") as err:',
        reason:
          "a child's live stdout and stderr, incremental on purpose so a long run can be " +
          "followed; only this run reads them, after the child exits",
      },
    ],
  ],
  [
    "tests/path_independence/runner.py",
    [{ site: "shutil.copytree(warmed, xdg)", reason: "copies a warmed cache in as a case input" }],
  ],
  [
    "scripts/release/publish_gate.py",
    [{ site: 'with path.open("a", encoding="utf-8") as output:', reason: GITHUB_OUTPUT }],
  ],
  [
    "scripts/release/resolve_plan.py",
    [{ site: 'with args.github_output.open("a", encoding="utf-8")', reason: GITHUB_OUTPUT }],
  ],
]);

// Copies of a helper for a project that cannot import the original, each required to
// stay byte-identical to it.
export const MIRRORS = new Map([
  ["explorations/benchmarks/atomic_write.py", "scripts/atomic_write.py"],
]);

// ---------------------------------------------------------------------------------------
// Lexing. Each structure keeps the source's length and newlines, blanks comments, and
// blanks the contents of string literals, so a write named in a comment or a string is
// not a write, and offsets still map to lines.

// Python: comments, and strings with any prefix, single or triple quoted. Literal
// contents are kept aside, because an open mode is a string.
export function pythonStructure(source) {
  let code = "";
  const literals = [];
  let index = 0;
  while (index < source.length) {
    const character = source[index];
    if (character === "#") {
      while (index < source.length && source[index] !== "\n") {
        code += " ";
        index += 1;
      }
      continue;
    }
    if (character === '"' || character === "'") {
      const triple = source.startsWith(character.repeat(3), index);
      const quote = triple ? character.repeat(3) : character;
      let cursor = index + quote.length;
      while (cursor < source.length) {
        if (source[cursor] === "\\") {
          cursor += 2;
          continue;
        }
        if (source.startsWith(quote, cursor)) break;
        if (!triple && source[cursor] === "\n") break;
        cursor += 1;
      }
      cursor = Math.min(cursor, source.length);
      const closed = source.startsWith(quote, cursor);
      const end = closed ? cursor + quote.length : cursor;
      const value = source.slice(index + quote.length, cursor);
      literals.push({ start: index, end, value });
      code += quote + blank(value) + (closed ? quote : "");
      index = end;
      continue;
    }
    code += character;
    index += 1;
  }
  return { code, literals };
}

// A slash opens a regular expression, not a division, after an operator, an opening
// bracket, or a keyword that takes an expression.
const REGEX_KEYWORDS =
  "return|typeof|case|do|else|in|of|new|delete|void|throw|yield|await|instanceof";
const REGEX_FOLLOWS = new RegExp(`(?:^|[(,=:[!&|?{};+\\-*%<>~^]|\\b(?:${REGEX_KEYWORDS}))\\s*$`);

// JavaScript: line and block comments, quoted strings, template literals (whose `${}`
// holes are code), and regular-expression literals.
export function scriptStructure(source) {
  let code = "";
  const literals = [];
  const holes = [];
  let index = 0;

  const scanTemplate = (start) => {
    // `start` is just past a backtick or a closing hole brace.
    let cursor = start;
    while (cursor < source.length) {
      if (source[cursor] === "\\") {
        cursor += 2;
        continue;
      }
      if (source[cursor] === "`") {
        literals.push({ start: start - 1, end: cursor + 1, value: source.slice(start, cursor) });
        code += blank(source.slice(start, cursor)) + "`";
        return cursor + 1;
      }
      if (source.startsWith("${", cursor)) {
        code += blank(source.slice(start, cursor)) + "${";
        holes.push(0);
        return cursor + 2;
      }
      cursor += 1;
    }
    code += blank(source.slice(start));
    return source.length;
  };

  while (index < source.length) {
    const character = source[index];
    const next = source[index + 1];
    if (character === "/" && next === "/") {
      const end = source.indexOf("\n", index);
      const stop = end === -1 ? source.length : end;
      code += blank(source.slice(index, stop));
      index = stop;
      continue;
    }
    if (character === "/" && next === "*") {
      const end = source.indexOf("*/", index + 2);
      const stop = end === -1 ? source.length : end + 2;
      code += blank(source.slice(index, stop));
      index = stop;
      continue;
    }
    if (character === '"' || character === "'") {
      let cursor = index + 1;
      while (cursor < source.length && source[cursor] !== character && source[cursor] !== "\n") {
        cursor += source[cursor] === "\\" ? 2 : 1;
      }
      cursor = Math.min(cursor, source.length);
      const closed = source[cursor] === character;
      const value = source.slice(index + 1, cursor);
      literals.push({ start: index, end: closed ? cursor + 1 : cursor, value });
      code += character + blank(value) + (closed ? character : "");
      index = closed ? cursor + 1 : cursor;
      continue;
    }
    if (character === "`") {
      code += "`";
      index = scanTemplate(index + 1);
      continue;
    }
    if (character === "{" && holes.length > 0) {
      holes[holes.length - 1] += 1;
    }
    if (character === "}" && holes.length > 0) {
      if (holes[holes.length - 1] === 0) {
        holes.pop();
        code += "}";
        index = scanTemplate(index + 1);
        continue;
      }
      holes[holes.length - 1] -= 1;
    }
    if (character === "/" && REGEX_FOLLOWS.test(code.slice(-40))) {
      const end = regexEnd(source, index);
      if (end !== -1) {
        code += "/" + blank(source.slice(index + 1, end - 1)) + "/";
        index = end;
        continue;
      }
    }
    code += character;
    index += 1;
  }
  return { code, literals };
}

// The index just past the closing slash of a regular expression opening at `start`, or
// -1 when the line ends first and the slash was division after all.
function regexEnd(source, start) {
  let inClass = false;
  for (let cursor = start + 1; cursor < source.length; cursor += 1) {
    const character = source[cursor];
    if (character === "\n") return -1;
    if (character === "\\") {
      cursor += 1;
    } else if (character === "[") {
      inClass = true;
    } else if (character === "]") {
      inClass = false;
    } else if (character === "/" && !inClass) {
      return cursor + 1;
    }
  }
  return -1;
}

function blank(text) {
  return text.replace(/[^\n]/g, " ");
}

// ---------------------------------------------------------------------------------------
// Calls.

const CLOSERS = { "(": ")", "[": "]", "{": "}" };

// The index of the bracket that closes the one at `open` in structural `code`, or -1.
function closingIndex(code, open) {
  let depth = 0;
  for (let cursor = open; cursor < code.length; cursor += 1) {
    if (CLOSERS[code[cursor]]) depth += 1;
    else if (code[cursor] === ")" || code[cursor] === "]" || code[cursor] === "}") {
      depth -= 1;
      if (depth === 0) return cursor;
    }
  }
  return -1;
}

// The arguments of the call whose opening parenthesis is at `open` in structural `code`,
// each as a span, split on top-level commas.
function callArguments(code, open) {
  const args = [];
  const stack = [];
  let start = open + 1;
  for (let cursor = open; cursor < code.length; cursor += 1) {
    const character = code[cursor];
    if (CLOSERS[character]) {
      stack.push(CLOSERS[character]);
    } else if (character === ")" || character === "]" || character === "}") {
      stack.pop();
      if (stack.length === 0) {
        if (code.slice(start, cursor).trim() !== "") args.push({ start, end: cursor });
        return args;
      }
    } else if (character === "," && stack.length === 1) {
      args.push({ start, end: cursor });
      start = cursor + 1;
    }
  }
  return args;
}

// The literal an argument consists of, if it is exactly one string literal.
function literalValue(structure, argument) {
  const text = structure.code.slice(argument.start, argument.end).trim();
  if (!/^[A-Za-z]{0,2}("""|'''|["'`])\s*\1$/.test(text)) return undefined;
  const literal = structure.literals.find(
    (candidate) => candidate.start >= argument.start && candidate.end <= argument.end,
  );
  return literal?.value;
}

function keywordArgument(structure, args, name) {
  for (const argument of args) {
    const text = structure.code.slice(argument.start, argument.end);
    const match = text.match(new RegExp(`^\\s*${name}\\s*=(?!=)`));
    if (match) return { start: argument.start + match[0].length, end: argument.end };
  }
  return undefined;
}

function positional(structure, args) {
  return args.filter(
    (argument) =>
      !/^\s*\*{0,2}\w+\s*=(?!=)/.test(structure.code.slice(argument.start, argument.end)),
  );
}

// ---------------------------------------------------------------------------------------
// Detection. Each auditor returns the raw writes in one file as { offset, kind }.

const PYTHON_MODE = /^([rwaxbtU+]{1,4})(?:[:|][\w*]*)?$/;
const PYTHON_MODULE_OPENERS = new Set([
  "builtins",
  "gzip",
  "bz2",
  "lzma",
  "io",
  "codecs",
  "tarfile",
  "os",
]);
// Modules whose `open` opens no file: the package's own `fdu.open(root)` serves a tree,
// and `webbrowser.open(url)` shows a page.
const PYTHON_NON_FILE_OPENERS = new Set(["fdu", "webbrowser"]);
const PYTHON_ARCHIVES = new Set(["GzipFile", "BZ2File", "LZMAFile", "ZipFile", "TarFile"]);
const OS_WRITE_FLAGS = /\bO_(?:WRONLY|RDWR|CREAT|APPEND|TRUNC)\b/;
// Flags spelled out as `os.O_*` names or numbers, joined by `|`, which can be read here.
const OS_LITERAL_FLAG = /^(?:(?:os\s*\.\s*)?O_[A-Z0-9_]+|\d+)$/;
// A call of `open`, `fdopen`, or an archive class, with the name before any dot.
const PYTHON_OPENERS = new RegExp(
  `(?:(\\b\\w+)\\s*\\.\\s*)?(?<![\\w])(open|fdopen|${[...PYTHON_ARCHIVES].join("|")})\\s*\\(`,
  "g",
);

function pythonModeWrites(value) {
  const match = value.match(PYTHON_MODE);
  return match !== null && /[wax+]/.test(match[1]);
}

function literalFlags(text) {
  return text
    .replace(/[()]/g, "")
    .split("|")
    .every((term) => OS_LITERAL_FLAG.test(term.trim()));
}

// The names `open_atomic(...) as name` binds. A dump into one of them is the helper's
// write, and any other handle of the same name was opened by a call reported on its own.
function atomicHandles(code) {
  const names = new Set();
  for (const match of code.matchAll(/\bopen_atomic\s*\(/g)) {
    const end = closingIndex(code, match.index + match[0].length - 1);
    if (end === -1) continue;
    const bound = code.slice(end + 1).match(/^\s*as\s+(\w+)/);
    if (bound) names.add(bound[1]);
  }
  return names;
}

export function auditPython(source) {
  const structure = pythonStructure(source);
  const { code } = structure;
  const writes = [];

  for (const match of code.matchAll(/\.write_(text|bytes)\s*\(/g)) {
    writes.push({ offset: match.index, kind: `.write_${match[1]}()` });
  }
  const atomic = atomicHandles(code);
  for (const match of code.matchAll(/\b(?:json|pickle)\.dump\s*\(/g)) {
    const open = match.index + match[0].length - 1;
    const handle = positional(structure, callArguments(code, open))[1];
    const target = handle ? code.slice(handle.start, handle.end).trim() : "";
    if (target === "sys.stdout" || target === "sys.stderr" || atomic.has(target)) continue;
    writes.push({ offset: match.index, kind: `${match[0].replace(/\s*\($/, "")}() to a file` });
  }
  for (const match of code.matchAll(/\bshutil\.(copy|copy2|copyfile|copytree)\s*\(/g)) {
    writes.push({ offset: match.index, kind: `shutil.${match[1]}()` });
  }

  for (const match of code.matchAll(PYTHON_OPENERS)) {
    const [, receiver, name] = match;
    // A definition, such as the package's own `def open(root, ...)`, is not a call.
    if (/\bdef\s+$/.test(code.slice(Math.max(0, match.index - 8), match.index))) continue;
    if (PYTHON_NON_FILE_OPENERS.has(receiver)) continue;
    // `x.open(` where x is not a bare name, such as `path.with_suffix(".x").open(`.
    const qualified = receiver !== undefined || code[match.index - 1] === ".";
    const open = match.index + match[0].length - 1;
    const args = callArguments(code, open);
    const label = `${receiver ? `${receiver}.` : qualified ? "." : ""}${name}()`;

    if (receiver === "os" && name === "open") {
      const flags =
        keywordArgument(structure, args, "flags") ?? positional(structure, args)[1];
      if (flags === undefined) continue;
      const text = code.slice(flags.start, flags.end);
      if (OS_WRITE_FLAGS.test(text)) {
        writes.push({ offset: match.index, kind: "os.open() for writing" });
      } else if (!literalFlags(text)) {
        // Flags this check cannot read may hold a write flag.
        writes.push({ offset: match.index, kind: "os.open() with computed flags" });
      }
      continue;
    }
    if (keywordArgument(structure, args, "fileobj")) continue;

    // Where the mode sits: second for the builtin and module-level openers and the archive
    // classes, first for a method such as Path.open.
    const moduleLevel =
      !qualified || PYTHON_MODULE_OPENERS.has(receiver) || PYTHON_ARCHIVES.has(name);
    const keyword = keywordArgument(structure, args, "mode");
    const place = keyword ?? positional(structure, args)[moduleLevel ? 1 : 0];
    if (place === undefined) continue;
    const value = literalValue(structure, place);
    if (value === undefined) {
      // A computed mode cannot be proven to read. A method's first argument is taken as
      // its mode, as Path.open's is; a method whose `open` takes something else is either
      // one of the modules above or a site listed with its reason.
      writes.push({ offset: match.index, kind: `${label} with a computed mode` });
      continue;
    }
    if (pythonModeWrites(value)) {
      writes.push({ offset: match.index, kind: `${label} with mode "${value}"` });
    }
  }
  return writes;
}

// Calls, not mentions: `import { writeFileSync }` has no parenthesis after the name.
const NODE_WRITER_NAMES = [
  "writeFileSync",
  "writeFile",
  "appendFileSync",
  "appendFile",
  "createWriteStream",
  "copyFileSync",
  "copyFile",
  "cpSync",
];
const NODE_WRITERS = new RegExp(`\\b(${NODE_WRITER_NAMES.join("|")})\\s*\\(`, "g");
const NODE_WRITE_FLAG = /^(?:w|wx|w\+|wx\+|a|ax|a\+|ax\+|as|as\+|r\+|rs\+)$/;

// The local names a writer is imported or destructured under, such as
// `import { writeFileSync as save }` or `const { writeFileSync: save } = require(...)`.
function nodeWriterAliases(code) {
  const aliases = new Map();
  const forms = [
    [/\bimport\s*(?:[\w$]+\s*,\s*)?\{([^}]*)\}/g, /^\s*([\w$]+)\s+as\s+([\w$]+)\s*$/],
    [/\b(?:const|let|var)\s*\{([^}]*)\}\s*=/g, /^\s*([\w$]+)\s*:\s*([\w$]+)\s*$/],
  ];
  for (const [group, specifier] of forms) {
    for (const match of code.matchAll(group)) {
      for (const item of match[1].split(",")) {
        const [, name, local] = item.match(specifier) ?? [];
        if (NODE_WRITER_NAMES.includes(name) && local !== name) aliases.set(local, name);
      }
    }
  }
  return aliases;
}

export function auditNode(source) {
  const structure = scriptStructure(source);
  const { code } = structure;
  const writes = [];
  for (const match of code.matchAll(NODE_WRITERS)) {
    writes.push({ offset: match.index, kind: `${match[1]}()` });
  }
  for (const [local, name] of nodeWriterAliases(code)) {
    const call = new RegExp(`(?<![\\w$.])${local.replace(/\$/g, "\\$")}\\s*\\(`, "g");
    for (const match of code.matchAll(call)) {
      if (/\bfunction\s*$/.test(code.slice(Math.max(0, match.index - 10), match.index))) continue;
      writes.push({ offset: match.index, kind: `${name}() imported as ${local}()` });
    }
  }
  for (const match of code.matchAll(/\b(openSync|open)\s*\(/g)) {
    const args = callArguments(code, match.index + match[0].length - 1);
    const flags = args[1];
    if (flags === undefined) continue;
    const value = literalValue(structure, flags);
    if (value === undefined) {
      if (match[1] === "openSync") {
        writes.push({ offset: match.index, kind: "openSync() with computed flags" });
      }
      continue;
    }
    if (NODE_WRITE_FLAG.test(value)) {
      writes.push({ offset: match.index, kind: `${match[1]}() with flags "${value}"` });
    }
  }
  return writes;
}

const RUST_WRITERS = [
  [/\bfs::write\s*\(/g, "fs::write()"],
  [/\bFile::create(?:_new)?\s*\(/g, "File::create()"],
  [/\bfs::copy\s*\(/g, "fs::copy()"],
  [/\.(?:write|append|create|create_new|truncate)\s*\(\s*true\s*\)/g, "a write-mode OpenOptions"],
];
const OPEN_OPTIONS_WRITES = new Set(["write", "append", "create", "create_new", "truncate"]);
const OPEN_OPTIONS_BUILDER = /\b(?:OpenOptions\s*::\s*new|File\s*::\s*options)\s*\(\s*\)/g;
// The std::fs writers a `use` can bring in under a bare name.
const RUST_FS_WRITERS = new Map([
  ["std::fs::write", "fs::write()"],
  ["std::fs::copy", "fs::copy()"],
]);

// Each path a `use` tree names, with the local name it binds: `std::{fs::{self, write as
// save}, io}` gives std::fs as fs, std::fs::write as save, and std::io as io.
function useLeaves(tree, prefix = []) {
  const text = tree.trim();
  const brace = text.indexOf("{");
  if (brace === -1) {
    const [path, alias] = text.split(/\s+as\s+/);
    const segments = [...prefix, ...path.split("::").map((part) => part.trim())].filter(Boolean);
    if (segments.at(-1) === "self") segments.pop();
    return [{ path: segments.join("::"), local: alias?.trim() ?? segments.at(-1) }];
  }
  const base = [...prefix, ...text.slice(0, brace).split("::").map((part) => part.trim())];
  const inner = text.slice(brace + 1, text.lastIndexOf("}"));
  const items = [];
  let depth = 0;
  let item = "";
  for (const character of inner) {
    if (character === "{") depth += 1;
    if (character === "}") depth -= 1;
    if (character === "," && depth === 0) {
      items.push(item);
      item = "";
    } else {
      item += character;
    }
  }
  items.push(item);
  return items
    .filter((entry) => entry.trim() !== "")
    .flatMap((entry) => useLeaves(entry, base.filter(Boolean)));
}

// Bare calls of a std::fs writer imported by `use`, such as `use std::fs::write;` and
// then `write(path, bytes)`.
function rustImportedWriters(code) {
  const writes = [];
  const locals = new Map();
  for (const declaration of code.matchAll(/\buse\s+([^;]+);/g)) {
    for (const { path, local } of useLeaves(declaration[1])) {
      if (RUST_FS_WRITERS.has(path)) locals.set(local, RUST_FS_WRITERS.get(path));
      if (path === "std::fs::*") {
        for (const [writer, kind] of RUST_FS_WRITERS) locals.set(writer.split("::").at(-1), kind);
      }
    }
  }
  for (const [local, kind] of locals) {
    for (const match of code.matchAll(new RegExp(`(?<![\\w:.])${local}\\s*\\(`, "g"))) {
      if (/\bfn\s+$/.test(code.slice(Math.max(0, match.index - 8), match.index))) continue;
      writes.push({ offset: match.index, kind: `${kind} imported as ${local}()` });
    }
  }
  return writes;
}

// The calls chained from `cursor`, each as { name, argument, offset }.
function rustChain(code, cursor) {
  const calls = [];
  for (;;) {
    const call = code.slice(cursor).match(/^\s*\.\s*(\w+)\s*\(/);
    if (!call) return calls;
    const open = cursor + call[0].length - 1;
    const close = closingIndex(code, open);
    if (close === -1) return calls;
    const offset = cursor + call[0].lastIndexOf(call[1]);
    calls.push({ name: call[1], argument: code.slice(open + 1, close).trim(), offset });
    cursor = close + 1;
  }
}

// An OpenOptions whose write, append, create, create_new, or truncate flag is an
// expression rather than `true` or `false`, whether set in the chain that builds it or
// through a binding in the same block. A literal `true` is reported by RUST_WRITERS.
function rustComputedOpenOptions(code) {
  const writes = [];
  const report = (calls) => {
    for (const call of calls) {
      if (!OPEN_OPTIONS_WRITES.has(call.name) || /^(?:true|false)$/.test(call.argument)) {
        continue;
      }
      const kind = `OpenOptions .${call.name}() with a computed flag`;
      writes.push({ offset: call.offset, kind });
    }
  };
  for (const builder of code.matchAll(OPEN_OPTIONS_BUILDER)) {
    const end = builder.index + builder[0].length;
    const chain = rustChain(code, end);
    report(chain);
    // Only `let options = OpenOptions::new();` binds the builder itself.
    if (chain.length > 0 || !/^\s*;/.test(code.slice(end))) continue;
    const before = code.slice(Math.max(0, builder.index - 200), builder.index);
    const binding = before.match(/\blet\s+(?:mut\s+)?(\w+)\s*(?::[^=;]*)?=\s*(?:\w+\s*::\s*)*$/);
    if (!binding) continue;
    // The binding lives until the block that holds it closes.
    let depth = 0;
    let stop = code.length;
    for (let index = end; index < code.length; index += 1) {
      if (code[index] === "{") depth += 1;
      if (code[index] === "}" && --depth < 0) {
        stop = index;
        break;
      }
    }
    const uses = new RegExp(`(?<![\\w.:])${binding[1]}(?=\\s*\\.)`, "g");
    for (const use of code.slice(end, stop).matchAll(uses)) {
      report(rustChain(code, end + use.index + use[0].length));
    }
  }
  return writes;
}

// The attribute names a configuration that exists only when testing: `cfg(test)`, or an
// `all(...)` with `test` among its terms.
function testOnlyAttribute(attribute) {
  const body = attribute.match(/^#\s*\[\s*cfg\s*\(([\s\S]*)\)\s*\]$/);
  if (!body) return false;
  const predicate = body[1].trim();
  if (predicate === "test") return true;
  const all = predicate.match(/^all\s*\(([\s\S]*)\)$/);
  if (!all) return false;
  let depth = 0;
  let term = "";
  const terms = [];
  for (const character of all[1]) {
    if (character === "(") depth += 1;
    if (character === ")") depth -= 1;
    if (character === "," && depth === 0) {
      terms.push(term.trim());
      term = "";
    } else {
      term += character;
    }
  }
  terms.push(term.trim());
  return terms.includes("test");
}

// Rust structure with every test-only item blanked, and the names of test-only modules
// declared in other files (`#[cfg(test)] mod name;`).
export function rustProductionStructure(source) {
  let code = rustStructure(source);
  const testModules = [];
  const attributes = /#\s*\[\s*cfg\s*\((?:[^()]|\((?:[^()]|\([^()]*\))*\))*\)\s*\]/g;
  for (const match of [...code.matchAll(attributes)].reverse()) {
    if (!testOnlyAttribute(match[0])) continue;
    const start = match.index;
    let cursor = start + match[0].length;
    // Skip any further attributes on the same item.
    for (;;) {
      const rest = code.slice(cursor).match(/^\s*#\s*\[/);
      if (!rest) break;
      cursor += rest[0].length;
      let depth = 1;
      while (cursor < code.length && depth > 0) {
        if (code[cursor] === "[") depth += 1;
        if (code[cursor] === "]") depth -= 1;
        cursor += 1;
      }
    }
    const external = code.slice(cursor).match(/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;/);
    if (external) testModules.push(external[1]);
    let depth = 0;
    let end = code.length;
    for (let index = cursor; index < code.length; index += 1) {
      const character = code[index];
      if (character === "(" || character === "[" || character === "{") depth += 1;
      if (character === ")" || character === "]" || character === "}") {
        depth -= 1;
        if (depth < 0) {
          end = index;
          break;
        }
        if (depth === 0 && character === "}") {
          end = index + 1;
          break;
        }
      }
      if (depth === 0 && (character === ";" || character === ",")) {
        end = index + 1;
        break;
      }
    }
    code = code.slice(0, start) + blank(code.slice(start, end)) + code.slice(end);
  }
  return { code, testModules };
}

export function auditRust(source) {
  const { code } = rustProductionStructure(source);
  const writes = [];
  for (const [pattern, kind] of RUST_WRITERS) {
    for (const match of code.matchAll(pattern)) writes.push({ offset: match.index, kind });
  }
  writes.push(...rustImportedWriters(code), ...rustComputedOpenOptions(code));
  return writes;
}

// The files a test-only module declaration in `path` names.
function rustModuleFiles(path, name) {
  const directory = posix.dirname(path);
  const base = posix.basename(path, ".rs");
  const parent = ["lib", "main", "mod"].includes(base) ? directory : posix.join(directory, base);
  return [posix.join(parent, `${name}.rs`), posix.join(parent, name, "mod.rs")];
}

// ---------------------------------------------------------------------------------------
// The audit.

const AUDITORS = [
  [/\.rs$/, auditRust],
  [/\.py$/, auditPython],
  [/\.(?:mjs|cjs|js|ts|mts)$/, auditNode],
];

function testCode(path) {
  return TEST_FILE.test(path) || TEST_DIRECTORIES.some((pattern) => pattern.test(path));
}

function lineOf(source, offset) {
  let line = 1;
  for (let index = 0; index < offset; index += 1) if (source[index] === "\n") line += 1;
  return line;
}

export function auditAtomicWrites(sources, policy = {}) {
  const helpers = policy.helpers ?? HELPERS;
  const inputs = policy.inputs ?? INPUT_WRITERS;
  const exceptions = policy.exceptions ?? EXCEPTIONS;
  const mirrors = policy.mirrors ?? MIRRORS;
  const problems = [];
  const counts = { files: 0, writes: 0, excused: 0 };
  const usedExceptions = new Set();

  // Rust modules compiled only for tests, wherever they are declared.
  const testModules = new Set();
  for (const [path, source] of sources) {
    if (!path.endsWith(".rs")) continue;
    for (const name of rustProductionStructure(source).testModules) {
      for (const file of rustModuleFiles(path, name)) testModules.add(file);
    }
  }

  for (const path of [...helpers.keys(), ...inputs.keys(), ...exceptions.keys()]) {
    if (!sources.has(path)) problems.push(`${path}: listed in the atomic-write policy but missing`);
  }
  for (const [mirror, original] of mirrors) {
    if (!sources.has(mirror)) {
      problems.push(`${mirror}: listed as a copy of ${original} but missing`);
    } else if (sources.get(mirror) !== sources.get(original)) {
      problems.push(`${mirror}: differs from ${original}; copy the helper again`);
    }
  }

  for (const [path, source] of sources) {
    const auditor = AUDITORS.find(([pattern]) => pattern.test(path))?.[1];
    if (auditor === undefined) continue;
    if (helpers.has(path) || mirrors.has(path) || inputs.has(path)) continue;
    if (testCode(path) || testModules.has(path)) continue;
    counts.files += 1;
    const lines = source.split("\n");
    for (const write of auditor(source)) {
      counts.writes += 1;
      const line = lineOf(source, write.offset);
      const text = lines[line - 1].trim();
      const exception = (exceptions.get(path) ?? []).find((entry) => text.includes(entry.site));
      if (exception) {
        usedExceptions.add(exception);
        counts.excused += 1;
        continue;
      }
      problems.push(
        `${path}:${line}: ${write.kind} writes in place; ` +
          "write through the atomic helper, or list the site with its reason",
      );
    }
  }

  for (const [path, entries] of exceptions) {
    for (const entry of entries) {
      if (!usedExceptions.has(entry)) {
        problems.push(`${path}: the exception for \`${entry.site}\` matches no write; remove it`);
      }
    }
  }
  return { problems, counts };
}

function trackedSources() {
  const listing = spawnSync(
    "git",
    ["ls-files", "-z", "--cached", "--others", "--exclude-standard"],
    { cwd: ROOT, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (listing.status !== 0) {
    throw new Error(`git ls-files failed: ${listing.stderr}`);
  }
  const sources = new Map();
  for (const path of listing.stdout.split("\0")) {
    if (!AUDITORS.some(([pattern]) => pattern.test(path))) continue;
    const absolute = `${ROOT}/${path}`;
    if (!existsSync(absolute)) continue;
    sources.set(path, readFileSync(absolute, "utf8"));
  }
  return sources;
}

function main() {
  const { problems, counts } = auditAtomicWrites(trackedSources());
  if (problems.length > 0) {
    console.error("atomic-write check failed:\n");
    for (const problem of problems) console.error(`  ${problem}`);
    console.error(
      "\nSee 'Write Every File Whole' in docs/project/architecture/fdu-design-principles.md.",
    );
    process.exitCode = 1;
    return;
  }
  console.log(
    `atomic-write check passed: ${counts.files} files audited; the ${counts.excused} raw ` +
      "writes outside the helpers and test inputs are each listed with a reason",
  );
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  main();
}
