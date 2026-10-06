#!/usr/bin/env node
// Put named patterns back into goldens that `tryscript run --update` expanded.
//
// The updater writes what it saw, so every `[PERF_TIME]`, `[PERF_RATE]`, or `[..]` in a
// session it rewrites becomes this machine's literal, and the golden then passes only
// here (see check-portability.mjs). After an intentional output change, most rewritten
// lines differ from the committed ones only inside such a pattern.
//
// For each golden that differs from a base revision (HEAD by default), this aligns the
// committed and updated lines and, inside each changed region, keeps the committed line
// wherever its patterns still match the updated one. A line whose literal text changed
// keeps the updated text: that is the intentional change, and its review is the
// author's. Run it after `npm run test:golden:update`, then review `git diff`.
//
//   node scripts/golden-restore-patterns.mjs [--base <rev>] [golden.tryscript.md ...]

import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { parse as parseYaml } from 'yaml';

import { writeFileAtomicSync } from './atomic-write.mjs';

const root = dirname(dirname(fileURLToPath(import.meta.url)));

/** Named patterns from a golden's front matter, as `{ NAME: source }`. */
export function frontMatterPatterns(text) {
  const match = /^---\n([\s\S]*?)\n---\n/.exec(text);
  if (!match) {
    return {};
  }
  return parseYaml(match[1])?.patterns ?? {};
}

const escape = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/**
 * A matcher for one expected line, or null when the line has no pattern to restore.
 *
 * `[..]` matches any text on the line; `[NAME]` uses the front-matter pattern, or any text
 * when the name is one tryscript defines itself. A `...` line elides whole lines and
 * cannot be paired line for line, so it is never restored here.
 */
export function lineMatcher(line, patterns) {
  const tokens = /\[\.\.\]|\[[A-Z][A-Z0-9_]*\]/g;
  if (line.trim() === '...' || !tokens.test(line)) {
    return null;
  }
  tokens.lastIndex = 0;
  let source = '';
  let last = 0;
  for (const token of line.matchAll(tokens)) {
    source += escape(line.slice(last, token.index));
    const name = token[0].slice(1, -1);
    source += name === '..' || !(name in patterns) ? '.*?' : `(?:${patterns[name]})`;
    last = token.index + token[0].length;
  }
  source += escape(line.slice(last));
  return new RegExp(`^${source}$`, 'u');
}

/** Longest-common-subsequence alignment as a list of `[op, line]`, op in ` -+`. */
export function diffLines(before, after) {
  const n = before.length;
  const m = after.length;
  const table = Array.from({ length: n + 1 }, () => new Uint32Array(m + 1));
  for (let i = n - 1; i >= 0; i -= 1) {
    for (let j = m - 1; j >= 0; j -= 1) {
      table[i][j] =
        before[i] === after[j] ? table[i + 1][j + 1] + 1 : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }
  const ops = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (before[i] === after[j]) {
      ops.push([' ', before[i]]);
      i += 1;
      j += 1;
    } else if (table[i + 1][j] >= table[i][j + 1]) {
      ops.push(['-', before[i]]);
      i += 1;
    } else {
      ops.push(['+', after[j]]);
      j += 1;
    }
  }
  for (; i < n; i += 1) ops.push(['-', before[i]]);
  for (; j < m; j += 1) ops.push(['+', after[j]]);
  return ops;
}

/** The updated text with committed pattern lines restored, and how many were. */
export function restorePatterns(committed, updated) {
  const patterns = frontMatterPatterns(committed);
  const ops = diffLines(committed.split('\n'), updated.split('\n'));
  const out = [];
  let restored = 0;
  for (let k = 0; k < ops.length; ) {
    if (ops[k][0] === ' ') {
      out.push(ops[k][1]);
      k += 1;
      continue;
    }
    // One changed region: its removed (committed) and added (updated) lines.
    const removed = [];
    const added = [];
    for (; k < ops.length && ops[k][0] !== ' '; k += 1) {
      (ops[k][0] === '-' ? removed : added).push(ops[k][1]);
    }
    const candidates = removed.map((line) => ({ line, matcher: lineMatcher(line, patterns) }));
    for (const line of added) {
      const index = candidates.findIndex((c) => c.matcher?.test(line));
      if (index === -1) {
        out.push(line);
      } else {
        out.push(candidates[index].line);
        candidates.splice(index, 1);
        restored += 1;
      }
    }
  }
  return { text: out.join('\n'), restored };
}

function main(argv) {
  let base = 'HEAD';
  const files = [];
  for (let k = 0; k < argv.length; k += 1) {
    if (argv[k] === '--base') {
      base = argv[(k += 1)];
    } else {
      files.push(argv[k]);
    }
  }
  // A missing or unknown base would make every file look new and restore nothing,
  // silently; refuse it instead.
  try {
    if (!base) throw new Error('--base needs a revision');
    execFileSync('git', ['rev-parse', '--verify', '--quiet', `${base}^{commit}`], { cwd: root });
  } catch {
    console.error(`golden-restore-patterns: not a revision: ${base ?? '(none)'}`);
    process.exit(2);
  }
  if (files.length === 0) {
    const dir = join(root, 'tests', 'golden');
    files.push(...readdirSync(dir).filter((f) => f.endsWith('.tryscript.md')).map((f) => join(dir, f)));
  }
  let total = 0;
  for (const file of files) {
    const path = relative(root, resolve(file)).split('\\').join('/');
    let committed;
    try {
      committed = execFileSync('git', ['show', `${base}:${path}`], { cwd: root, encoding: 'utf8' });
    } catch {
      continue; // New in this change: nothing committed to restore from.
    }
    const updated = readFileSync(join(root, path), 'utf8');
    if (committed === updated) {
      continue;
    }
    const { text, restored } = restorePatterns(committed, updated);
    if (restored > 0) {
      writeFileAtomicSync(join(root, path), text);
      console.log(`golden-restore-patterns: ${path}: restored ${restored} pattern lines`);
      total += restored;
    }
  }
  console.log(`golden-restore-patterns: ${total} lines restored against ${base}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2));
}
