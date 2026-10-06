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
//
// [JSON_SEP] is the same separator inside a JSON string, and the artifact is recorded on
// Linux, where that separator is `/` as well. It reached a failing block for the first
// time when the stale-answer warning (fdu-mdop) made the one JSONL tree session with
// nested paths differ on stderr, which puts its whole stdout block in the diff.
const sameSeparator = (line) => line.replace(/\[(?:JSON_)?SEP\]/g, '/');

// tryscript expands named patterns into concrete values on the Python side of a diff.
// After classification checks those values, keep the artifact stable across runs.
// Only '+' lines are observed output; '-' golden expectations remain untouched.
export function normalisePortableValues(text) {
    const lines = text.split('\n');
    let removed = [];
    let added = [];
    const flush = () => {
      if (removed.length === added.length) {
        for (let i = 0; i < removed.length; i += 1) {
          let value = added[i].value;
          for (const [field, named, marker] of [
            ['age_reference_ns', 'AGE_NS', 'AGE_NS_VALUE'],
            ['observed_at_ns', 'MTIME_NS', 'MTIME_NS_VALUE'],
            ['allocated', 'ALLOCATED', 'ALLOCATED_VALUE'],
          ]) {
            const fieldValues = (line) =>
              [...line.matchAll(new RegExp(`("${field}":\\s*)(\\[[A-Z_]+\\]|-?\\d+)`, 'g'))];
            const expected = fieldValues(removed[i]);
            const actual = fieldValues(value);
            if (expected.length !== actual.length) continue;
            let occurrence = 0;
            value = value.replace(
              new RegExp(`("${field}":\\s*)(-?\\d+)`, 'g'),
              (match, prefix) =>
                expected[occurrence++]?.[2] === `[${named}]` ? `${prefix}[${marker}]` : match,
            );
          }
          lines[added[i].index] = `+${value}`;
        }
      }
      removed = [];
      added = [];
    };
    for (let i = 0; i < lines.length; i += 1) {
      const line = lines[i];
      if (/^(?:FAIL |PASS |  ✗ |  ✓ )/.test(line)) {
        flush();
      } else if (line.startsWith('-')) {
        removed.push(line.slice(1));
      } else if (line.startsWith('+')) {
        added.push({ index: i, value: line.slice(1) });
      }
    }
    flush();
    return lines.join('\n');
}

// The golden stores a portable scan-root pattern, while the Python replay prints its
// concrete sandbox root. Match only the fixture root for that session: accepting any
// [SANDBOX] path would conceal a Python request that scanned a different directory.
const fixtureRoot = (file = '') => {
  if (file.endsWith('/cli-content.tryscript.md')) return 'content-project';
  if (/\/(?:cli-axes|cli-cache|cli-json)\.tryscript\.md$/.test(file)) return 'project';
  return null;
};
const portablePatternMatches = (line, actual, file) => {
  let value = sameSeparator(line);
  const fixture = fixtureRoot(file);
  if (fixture) {
    value = value
      .replaceAll('"root": "[SCAN_PATH]"', `"root": "[SANDBOX]/${fixture}"`)
      .replace(/^root: \[SCAN_PATH\]$/, `root: [SANDBOX]/${fixture}`);
  }
  // tryscript reports a whole changed line. A JSONL envelope therefore also contains
  // the golden's numeric platform patterns when only its root path exposed the line.
  // Honor only the patterns already named by the golden, with their original numeric
  // shapes; every other field must still compare byte-for-byte.
  const escaped = value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const pattern = escaped
    .replaceAll('\\[AGE_NS\\]', '(?:\\[AGE_NS_VALUE\\]|-?\\d+)')
    // normalise() already masks `newest_mtime_ns`, but leaves `observed_at_ns`
    // numeric. The same named golden pattern appears in both fields.
    .replaceAll('\\[MTIME_NS\\]', '(?:\\[MTIME_NS\\]|\\[MTIME_NS_VALUE\\]|-?\\d+)')
    .replaceAll('\\[ALLOCATED\\]', '(?:\\[ALLOCATED_VALUE\\]|\\d+)');
  return new RegExp(`^${pattern}$`).test(sameSeparator(actual));
};

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
  /--gitignore-budget|--gitignore-line-limit|--ignored=exclude|--ignored=only|--no-gitignore|--scan-depth|--one-filesystem|--modified-since|--include|--depth|--stale-ok|--cache|--watch|cache policy|stale_ok|ignored=exclude|ignored=only|control_budget|control_line_limit|read_controls|max_depth|one_filesystem|modified_since|include|depth|watch/g;
const withoutKnobs = (line) => sameSeparator(line).replace(KNOBS, '<knob>');

// A report's one bound suggestion lifts every bound that hid something, and names each
// setter as its surface does: flags joined as one command line, keyword arguments as one
// call's arguments. Only these four setters, with these values, in the renderer's fixed
// order and each at most once, are accepted, so no other tip can borrow this exception
// merely because it contains a familiar word.
const BOUND_TIP = 'tip: show more: ';
const BOUND_SETTERS = [
  ['--min-share=0%', 'min_share=0%'],
  ['--depth=all', 'depth=all'],
  ['--breadth=all', 'breadth=all'],
  ['--limit=all', 'limit=all'],
];
const sameBoundTip = (removed, added) => {
  const prefix = (removed.startsWith('! ') ? '! ' : '') + BOUND_TIP;
  if (!removed.startsWith(prefix) || !added.startsWith(prefix)) return false;
  const flags = removed.slice(prefix.length).split(' ');
  const fields = added.slice(prefix.length).split(', ');
  if (flags.length !== fields.length) return false;
  let previous = -1;
  return flags.every((flag, i) => {
    const index = BOUND_SETTERS.findIndex(([cli, api]) => cli === flag && api === fields[i]);
    if (index <= previous) return false;
    previous = index;
    return true;
  });
};
// `full` names the analyzer value that includes the views it skipped. Pinned whole, like
// the bound tip, so no other tip can borrow it.
const sameAnalysisTip = (removed, added) => {
  const marker = removed.startsWith('! ') ? '! ' : '';
  return (
    removed === `${marker}tip: include them: --analyze all` &&
    added === `${marker}tip: include them: analyze all`
  );
};
// The engine's stale-answer warning names each surface's own option for a fresh answer
// (fdu-mdop). Pinned whole, like the tips above, so no other warning can borrow it.
const staleWarning = (option) =>
  `warn: stale answer: served from the snapshot without filesystem verification; drop ${option} for a fresh answer`;
const sameStaleWarning = (removed, added) => {
  const marker = removed.startsWith('! ') ? '! ' : '';
  return removed === marker + staleWarning('--stale-ok') && added === marker + staleWarning('stale_ok');
};
const usesBoundTip = (line) =>
  /^(! )?tip: /.test(line) &&
  /(?:--min-share|min_share|--depth|depth|--breadth|breadth|--limit|limit)=/.test(line);

// A class that no longer explains anything is removed, not kept "just in case". Its
// matcher would still match, so it would quietly absorb a real regression: `execution-tier`
// covered eight sessions until fdu.report exposed the one-shot contract, and its matcher
// keyed on "source" and cache-emptiness -- exactly what a cache regression would look like.
export const CLASSES = [
  {
    id: 'portable-golden-pattern',
    title: 'Portable golden spelling of the same fixture path',
    why: [
      'The CLI golden uses [SCAN_PATH] for the known fixture root and [SEP] for a',
      'platform separator. The Python replay prints the sandbox root and a literal',
      'separator. Only the exact fixture root and otherwise identical lines match;',
      'exact bound and omitted-view tip translations, and the stale-answer warning',
      'naming each surface\'s option, can accompany those lines.',
    ],
    matches: ({ file, removed, added }) =>
      removed.length > 0 &&
      removed.length === added.length &&
      removed.every((line, i) =>
        portablePatternMatches(line, added[i], file) ||
        sameBoundTip(line, added[i]) ||
        sameAnalysisTip(line, added[i]) ||
        sameStaleWarning(line, added[i]),
      ) &&
      removed.some((line, i) =>
        line !== added[i] && portablePatternMatches(line, added[i], file),
      ),
  },
  {
    id: 'bound-tip-vocabulary',
    title: 'Bound suggestions name the same setter on each surface',
    why: [
      'The command line names bound flags and Python names corresponding fields.',
      'Only the four setter and value pairs the shared report renderer emits, in its',
      'fixed order, are accepted; every other line remains identical.',
    ],
    matches: ({ removed, added }) =>
      removed.length > 0 &&
      removed.length === added.length &&
      removed.every((line, i) =>
        sameSeparator(line) === sameSeparator(added[i]) || sameBoundTip(line, added[i]),
      ) &&
      removed.some((line, i) => sameBoundTip(line, added[i])),
  },
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
      !removed.some(usesBoundTip) &&
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
      !removed.some(usesBoundTip) &&
      removed.every((line, i) => withoutKnobs(line) === withoutKnobs(added[i])) &&
      removed.some((line, i) => line !== added[i]),
  },
  {
    id: 'discovery-surface',
    title: 'A discovery surface the package does not carry',
    why: [
      '--docs is a static document that lives in the binary, so the package does not carry',
      'it and the session is recorded here rather than skipped. The skip list is a separate',
      'set -- clap help and usage errors, --skill, and --install-skill -- and lives in',
      'DECLINED in run-parity.mjs (fdu-2b53).',
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
