// Why each surviving difference between the two surfaces is allowed to survive.
//
// A flat list of failures says nothing about which are understood. Every deviation is
// matched against exactly one of these, and anything that matches none is UNEXPLAINED and
// fails the run -- so "we know why each of these differs" is checked rather than claimed,
// and a new difference cannot hide among the accepted ones.
//
// Each `matches` is mechanical. None of them says "this session is fine"; they say what
// the difference IS, and a session only lands in a class if its hunks actually look like
// that. Adding a class is a claim that needs the same scrutiny as changing behaviour.

// A golden expectation writes the path separator as the named pattern [SEP], because the
// corpus also runs on Windows. Both surfaces print the platform's own separator and
// run-parity's normalise() has already folded \ into /, so for comparison the pattern and
// the literal are the same text. Without this, any session whose block diff contains a
// path cannot be matched line by line, and a real difference elsewhere in that block -- a
// label, a note -- reads as unexplained for a reason that has nothing to do with it.
const sameSeparator = (line) => line.replace(/\[SEP\]/g, '/');

/** Flags and parameters name the same thing: --modified-since is modified_since. */
const sameName = (line) => sameSeparator(line).replace(/--(?=[a-z])/g, '').replace(/[-_]/g, '');

// The library reports a rule as a runtime error and names its kind; the CLI catches the
// same rule up front and reports it as a usage error, which carries no kind and exits 2
// instead of 1. Listed explicitly rather than matched loosely, so adding one is a visible
// decision -- a general "strip any prefix" rule would hide real differences.
const ERROR_KINDS = ['unsupported scan configuration: '];

const withoutKind = (line) => {
  // The kind sits after the program name, which both surfaces print: the shim says
  // `fdu: unsupported scan configuration: ...` where the CLI says `fdu: ...`.
  for (const kind of ERROR_KINDS) {
    if (line.includes(kind)) {
      return line.replace(kind, '');
    }
  }
  return line;
};

// A class that no longer explains anything is removed, not kept "just in case". Its
// matcher would still match, so it would quietly absorb a real regression: `execution-tier`
// covered eight sessions until fdu.report exposed the one-shot contract, and its matcher
// keyed on "source" and cache-emptiness -- exactly what a cache regression would look like.
export const CLASSES = [
  {
    id: 'surface-label',
    title: 'Each surface names its own parameter',
    why: [
      'There is no --view or --analyze in Python, so its diagnostics name the parameter.',
      'Everything after the label is identical, and that is the part encoding behaviour:',
      'the rule the caller hits is the same rule, proven line by line rather than assumed.',
      '',
      'Some entries here also differ by an error-kind prefix: the library reports a scan',
      'configuration it cannot serve as a runtime error and names its kind, while the CLI',
      'catches the same case up front and reports a usage error. Everything after the kind',
      'is compared exactly, so the two are one rule with two framings rather than two',
      'rules.',
    ],
    // Strip the flag dashes and normalise -/_ ; if the lines then match exactly, the
    // label is the whole of the difference. Anything else and this class does not apply.
    matches: ({ removed, added }) =>
      removed.length > 0 &&
      removed.length === added.length &&
      removed.every((line, i) => sameName(line) === sameName(withoutKind(added[i]))) &&
      removed.some((line, i) => line !== added[i]),
  },
  {
    id: 'run-telemetry',
    title: 'Output carrying walk telemetry the report schema excludes',
    why: [
      'A note quoting how many bytes analysis read, or the performance footer itself.',
      'Both are telemetry about the run rather than facts about the report, and the',
      'envelope deliberately carries none, so a Report cannot reproduce them. The omission',
      'note is NOT this: that one is a fact about the report, travels on it, and every',
      'surface states it in its own vocabulary.',
    ],
    matches: ({ removed, added }) =>
      removed.some((line) => /^note:|^Performance:/.test(line)) &&
      removed.filter((line) => !/^note:|^Performance:/.test(line)).length === added.length,
  },
  {
    id: 'process-instrumentation',
    title: 'Counter relations only a binary invocation can report',
    why: [
      'The cost goldens assert relations between the counters FDU_COUNTERS=1 records --',
      'stats against entries, a cache-only answer touching nothing. Those counters include',
      'an allocator tier, and the allocator is global to a process: the `fdu` binary',
      'installs a counting one in its own `main`, and an extension module loaded into a',
      'host interpreter cannot install one for CPython. So the package reports no counters',
      'at all rather than a subset that would read as a smaller number for the same work.',
      'The scripts say so in one line instead of failing, which is what makes this',
      'mechanical: the package emits exactly {"instrumented":false} where the binary emits',
      'a relation object. A different Python answer would not match.',
    ],
    matches: ({ removed, added }) =>
      added.length === 1 &&
      added[0] === '{"instrumented":false}' &&
      removed.length === 1 &&
      removed[0].startsWith('{"'),
  },
  {
    id: 'discovery-surface',
    title: 'A discovery surface the package does not carry',
    why: [
      '--docs is a static document that lives in the binary, so the package does not carry',
      'it and the session is recorded here rather than skipped. The skip list is a separate',
      'set -- clap help and usage errors, and --skill -- and lives in DECLINED in',
      'run-parity.mjs (fdu-2b53).',
      '',
      '--version is the deliberate one, and it is load-bearing: the shim names itself so',
      'it can never be mistaken for the binary, which is what keeps this artifact',
      'non-empty. An empty artifact would mean the shim never ran.',
    ],
    // Not `The Portable Skill`: that session is in run-parity's DECLINED list, so it is
    // filtered out before anything is compared and an alternative for it here could never
    // match. One mechanism per session, and the two lists must not both claim one.
    matches: ({ name }) =>
      /^(Version Is Exact|The Guide Is Reachable Without a Root)/.test(name),
  },
];

/** Split the runner's output into one record per failing session. */
export function parseSessions(text) {
  const sessions = [];
  let current = null;
  let file = '';
  const flush = () => {
    if (current) sessions.push(current);
    current = null;
  };
  for (const line of text.split('\n')) {
    if (line.startsWith('FAIL ') || line.startsWith('PASS ')) {
      flush();
      file = line.slice(5).trim();
    } else if (line.startsWith('  ✗ ')) {
      flush();
      current = { name: line.slice(4), file, removed: [], added: [] };
    } else if (line.startsWith('  ✓ ')) {
      flush();
    } else if (current && line.startsWith('-')) {
      current.removed.push(line.slice(1));
    } else if (current && line.startsWith('+')) {
      current.added.push(line.slice(1));
    }
  }
  flush();
  return sessions;
}

export function classify(session) {
  return CLASSES.find((cls) => cls.matches(session)) ?? null;
}
