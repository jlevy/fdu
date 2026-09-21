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

// The two surfaces have different NAMES for the same knob, not merely different
// punctuation: the flag is --scan-depth and the field is max_depth. Comparing with these
// elided on both sides proves the rest of the sentence is identical without this file
// having to restate the pairing -- the pairing lives in `AxisNames`, where a unit test
// renders every refusal in both vocabularies and pins the result.
// Longest first, and one pass, so `depth` cannot match inside `--scan-depth`.
//
// `cache policy` is the one name that is not a field: the Python parameter is `cache`, and
// its refusals have always said `invalid cache policy`, which `AxisNames::FIELDS` kept
// rather than changing the wording when the rule moved into the request model.
const KNOBS =
  /--gitignore-budget|--gitignore-line-limit|--exclude-ignored|--only-ignored|--no-gitignore|--scan-depth|--one-filesystem|--modified-since|--include|--depth|--cache|--watch|cache policy|ignored=exclude|ignored=only|control_budget|control_line_limit|read_controls|max_depth|one_filesystem|modified_since|include|depth|watch/g;
const withoutKnobs = (line) => sameSeparator(line).replace(KNOBS, '<knob>');

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
    ],
    // Strip the flag dashes and normalise -/_ ; if the lines then match exactly, the
    // label is the whole of the difference. Anything else and this class does not apply.
    matches: ({ removed, added }) =>
      removed.length > 0 &&
      removed.length === added.length &&
      removed.every((line, i) => sameName(line) === sameName(added[i])) &&
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
      'envelope deliberately carries none, so a Report cannot reproduce them. The omission',
      'note is NOT this: that one is a fact about the report, travels on it, and every',
      'surface states it in its own vocabulary.',
    ],
    // Everything that is not telemetry has to be unchanged, line for line. Counting the
    // remainder instead let this class absorb a genuinely different answer: a session
    // whose command-line output lost a `note:` line could also report a different tally
    // and still be explained, because one removed line was matched by one added line
    // whatever the two said. That is the opposite of what a class is for -- the header
    // above requires a class to say what the difference IS.
    matches: ({ removed, added }) => {
      const telemetry = (line) => /^note:|^Performance:/.test(line);
      const rest = removed.filter((line) => !telemetry(line));
      return (
        removed.some(telemetry) &&
        rest.length === added.length &&
        rest.every((line, index) => line === added[index])
      );
    },
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
