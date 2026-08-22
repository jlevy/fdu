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

/** Flags and parameters name the same thing: --modified-since is modified_since. */
const sameName = (line) => line.replace(/--(?=[a-z])/g, '').replace(/[-_]/g, '');

// The library reports a rule as a runtime error and names its kind; the CLI catches the
// same rule up front and reports it as a usage error, which carries no kind and exits 2
// instead of 1. Listed explicitly rather than matched loosely, so adding one is a visible
// decision -- a general "strip any prefix" rule would hide real differences.
const ERROR_KINDS = ['unsupported scan configuration: '];

// The two surfaces have different NAMES for the same knob, not merely different
// punctuation: the flag is --scan-depth and the field is max_depth. Comparing with these
// elided on both sides proves the rest of the sentence is identical without this file
// having to restate the pairing -- the pairing lives in cli.rs, where a unit test pins it.
// Longest first, and one pass, so `depth` cannot match inside `--scan-depth`.
const KNOBS =
  /--scan-depth|--one-filesystem|--modified-since|--include|--depth|max_depth|one_filesystem|modified_since|include|depth/g;
const withoutKnobs = (line) => withoutKind(line).replace(KNOBS, '<knob>');
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
      'One entry here also differs by an error-kind prefix. The library reports the watch',
      'scope rule as a runtime error and names its kind; the CLI catches it up front and',
      'reports a usage error, which exits 2 rather than 1. The rule text is one constant',
      'either way, with only the knob names substituted, and a unit test pins that.',
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
    id: 'surface-vocabulary',
    title: 'The same rule, in each surface\'s own names for the same knobs',
    why: [
      'The command line calls it --scan-depth and the API calls it max_depth. They are',
      'different words, not different punctuation, so this is checked by eliding the knob',
      'names on both sides and requiring the rest of the sentence to be identical.',
      '',
      'The rule itself is one constant -- scan::WATCH_SCOPE_GUIDANCE -- with the knob names',
      'substituted for the command line in a single whole-word pass. It was two separate',
      'messages that had already drifted: the CLI explained what to do instead, and the',
      'library said "requires event-scope filtering", so a library caller got jargon and a',
      'CLI user got help. A unit test pins the substitution, because doing it sequentially',
      'produced `--scan---depth` the first time (the fdu-7j6z bug, again).',
    ],
    matches: ({ removed, added }) =>
      removed.length > 0 &&
      removed.length === added.length &&
      removed.every((line, i) => withoutKnobs(line) === withoutKnobs(added[i])) &&
      removed.some((line, i) => line !== added[i]),
  },
  {
    id: 'run-telemetry',
    title: 'Output carrying walk telemetry the report schema excludes',
    why: [
      'A note quoting how many bytes analysis read, or the performance footer itself.',
      'Both are telemetry about the run rather than facts about the report, and the',
      'envelope deliberately carries none, so a Report cannot reproduce them (fdu-x8u6).',
    ],
    matches: ({ removed, added }) =>
      removed.some((line) => /^note:|^Performance:/.test(line)) &&
      removed.filter((line) => !/^note:|^Performance:/.test(line)).length === added.length,
  },
  {
    id: 'watch-render',
    title: 'Watch records have no renderer',
    why: [
      'Index.watch yields Change values and nothing turns them into the CLI bytes, so the',
      'shim prints repr(). The cache-status fix is the template, but the watch loop also',
      'owns repaint timing and the stream envelope, so what is a renderer and what is loop',
      'behaviour has to be decided before moving anything (fdu-m66a).',
    ],
    matches: ({ file }) => /cli-watch\.tryscript\.md$/.test(file),
  },
  {
    id: 'discovery-surface',
    title: 'A discovery surface the package does not carry',
    why: [
      '--docs and --skill are static documents that live in the binary, and --help is',
      "clap's own rendering. The whole skip list is these three (fdu-2b53).",
      '',
      '--version is the deliberate one, and it is load-bearing: the shim names itself so',
      'it can never be mistaken for the binary, which is what keeps this artifact',
      'non-empty. An empty artifact would mean the shim never ran.',
    ],
    matches: ({ name }) =>
      /^(Version Is Exact|The Guide Is Reachable Without a Root|The Portable Skill)/.test(name),
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
