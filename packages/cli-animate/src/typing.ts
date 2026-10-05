/**
 * Keystroke timing for a fast, skilled human typist, seeded so a take reproduces.
 *
 * Each key waits an inter-key interval (IKI) after the previous one:
 *
 *   IKI = base × digraph factor × token factor × noise (+ a stall or a hesitation)
 *
 * - The base comes from the target speed in words per minute, calibrated so the reference
 *   text below averages that speed.
 * - The digraph factor depends on the pair of keys, relative to a hand alternation, with
 *   values measured on the fastest typists (103–130 WPM) of the Behmer and Crump
 *   copy-typing data: same hand 1.10 (1.20 across the top and bottom rows), same finger
 *   1.70, repeated key 1.50, frequent English digraph 0.85, first key of a word 1.20.
 *   Digits are 1.7 (CMU keystroke data), Shift 1.5 with 1.25 on the key after it, and
 *   little-finger punctuation 1.15.
 * - Shell syntax splits a word into sub-tokens (`/`, `-`, `=`, `.` and similar); the first
 *   key of a sub-token is 1.10.
 * - Whole tokens run faster or slower together (log-normal σ 0.12). That word-level
 *   factor, not stroke-to-stroke correlation, is what the data shows.
 * - Per-key noise is mean-preserving log-normal, σ 0.30, as measured. Its right tail is
 *   too light, so 1.5% of keys inside a word stall at 2–4×, and 5% of words start after a
 *   hesitation (log-normal excess, median 170 ms, capped at 600 ms).
 *
 * The research brief (docs/project/research/research-2026-10-04-terminal-demo-recordings.md)
 * records the sources and which values are measured and which extrapolated.
 */

/** A small, fast, seedable generator (sfc32, seeded through splitmix32). */
export class Rng {
  private a: number;
  private b: number;
  private c: number;
  private d: number;

  constructor(seed: number) {
    let s = seed >>> 0;
    const next = (): number => {
      s = (s + 0x9e3779b9) >>> 0;
      let z = s;
      z = Math.imul(z ^ (z >>> 16), 0x85ebca6b);
      z = Math.imul(z ^ (z >>> 13), 0xc2b2ae35);
      return (z ^ (z >>> 16)) >>> 0;
    };
    this.a = next();
    this.b = next();
    this.c = next();
    this.d = next();
  }

  /** Uniform in [0, 1). */
  random(): number {
    const t = (((this.a + this.b) >>> 0) + this.d) >>> 0;
    this.d = (this.d + 1) >>> 0;
    this.a = this.b ^ (this.b >>> 9);
    this.b = (this.c + (this.c << 3)) >>> 0;
    this.c = (this.c << 21) | (this.c >>> 11);
    this.c = (this.c + t) >>> 0;
    return t / 4294967296;
  }

  /** Standard normal, by Box–Muller; always consumes two draws. */
  gauss(): number {
    const u = 1 - this.random();
    const v = this.random();
    return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
  }
}

type Hand = 'L' | 'R' | 'T';
interface Key {
  hand: Hand;
  /** 1 index, 2 middle, 3 ring, 4 little; 0 thumb. */
  finger: number;
  /** 0 number row, 1 top, 2 home, 3 bottom, 4 space. */
  row: number;
  shifted: boolean;
}

// US QWERTY, standard touch-typing fingers. The letter rows sit half a key right of the
// number row, so a letter row's column n is struck by the finger of number-row column n+1.
const ROWS: [string, number][] = [
  ['`1234567890-=', 0],
  ['qwertyuiop[]\\', 1],
  ["asdfghjkl;'", 2],
  ['zxcvbnm,./', 3],
];
const FINGER_BY_COLUMN: [Hand, number][] = [
  ['L', 4], ['L', 4], ['L', 3], ['L', 2], ['L', 1], ['L', 1],
  ['R', 1], ['R', 1], ['R', 2], ['R', 3], ['R', 4], ['R', 4], ['R', 4], ['R', 4],
];
const SHIFTED_FROM = '~!@#$%^&*()_+{}|:"<>?';
const SHIFTED_TO = "`1234567890-=[]\\;',./";

const KEYS = new Map<string, Omit<Key, 'shifted'>>();
for (const [chars, row] of ROWS) {
  const offset = row === 0 ? 0 : 1;
  [...chars].forEach((ch, column) => {
    const [hand, finger] = FINGER_BY_COLUMN[Math.min(column + offset, FINGER_BY_COLUMN.length - 1)]!;
    KEYS.set(ch, { hand, finger, row });
  });
}
KEYS.set(' ', { hand: 'T', finger: 0, row: 4 });

const AVERAGE_KEY: Key = { hand: 'R', finger: 1, row: 2, shifted: false };

export function keyOf(ch: string): Key {
  const lower = ch.toLowerCase();
  const plain = KEYS.get(lower);
  if (plain) return { ...plain, shifted: ch !== lower };
  const index = SHIFTED_FROM.indexOf(ch);
  if (index >= 0) return { ...KEYS.get(SHIFTED_TO[index]!)!, shifted: true };
  return AVERAGE_KEY;
}

/** Frequent English letter digraphs, typed faster than their geometry predicts. */
export const FREQUENT_DIGRAPHS: ReadonlySet<string> = new Set(
  (
    'th he in er an re on at en nd ti es or te of ed is it al ar st to nt ng se ha as ou io le ' +
    've co me de hi ri ro ic ne ea ra ce li ch ll be ma si om ur'
  ).split(' '),
);

/** Characters that split a shell word into sub-tokens. */
const SUBTOKEN_SEPARATORS = new Set(['/', '-', '=', '.', '|', '&', ':', ',', '~', '_']);

/** Ordinary English plus typical shell commands: the text the speed target refers to. */
export const REFERENCE_TEXT =
  'the quick brown fox jumps over the lazy dog while the typist works through a long' +
  ' morning of notes and commands; find the largest files in the home directory, then' +
  ' show the code in the source tree by language and check what changed since noon.' +
  ' git status && cargo build --release -p fdu && fdu ~/src --depth 2 --limit 10';

export interface TypingProfile {
  /** Words per minute (five characters a word) on REFERENCE_TEXT. */
  wpm: number;
  alternateHand: number;
  sameHand: number;
  rowJump: number;
  sameFinger: number;
  repeatKey: number;
  frequentDigraph: number;
  wordInitial: number;
  subtokenInitial: number;
  space: number;
  digit: number;
  numberRowSymbol: number;
  pinkyPunctuation: number;
  shift: number;
  afterShift: number;
  sigma: number;
  tokenSigma: number;
  stallRate: number;
  stallRange: [number, number];
  hesitationRate: number;
  hesitationMedianS: number;
  hesitationSigma: number;
  hesitationCapS: number;
  floorS: number;
}

export const DEFAULT_PROFILE: Readonly<TypingProfile> = {
  wpm: 160,
  alternateHand: 1.0,
  sameHand: 1.1,
  rowJump: 1.1,
  sameFinger: 1.7,
  repeatKey: 1.5,
  frequentDigraph: 0.85,
  wordInitial: 1.2,
  subtokenInitial: 1.1,
  space: 1.0,
  digit: 1.7,
  numberRowSymbol: 1.4,
  pinkyPunctuation: 1.15,
  shift: 1.5,
  afterShift: 1.25,
  sigma: 0.3,
  tokenSigma: 0.12,
  stallRate: 0.015,
  stallRange: [2, 4],
  hesitationRate: 0.05,
  hesitationMedianS: 0.17,
  hesitationSigma: 0.5,
  hesitationCapS: 0.6,
  floorS: 0.03,
};

const isLetter = (ch: string): boolean => /^[a-z]$/i.test(ch);
const isDigit = (ch: string): boolean => /^[0-9]$/.test(ch);
const startsToken = (prev: string | undefined): boolean => prev === undefined || prev === ' ';

/** The interval before `ch` relative to a hand alternation, given the previous character. */
export function digraphFactor(prev: string | undefined, ch: string, p: TypingProfile = DEFAULT_PROFILE): number {
  const key = keyOf(ch);
  if (ch === ' ') return p.space;
  let f: number;
  if (startsToken(prev)) {
    f = p.wordInitial;
  } else {
    const previous = keyOf(prev!);
    if (prev!.toLowerCase() === ch.toLowerCase()) f = p.repeatKey;
    else if (previous.hand !== key.hand) f = p.alternateHand;
    else if (previous.finger === key.finger) f = p.sameFinger;
    else f = p.sameHand * (Math.abs(previous.row - key.row) >= 2 ? p.rowJump : 1);
    if (FREQUENT_DIGRAPHS.has((prev! + ch).toLowerCase())) f *= p.frequentDigraph;
    if (SUBTOKEN_SEPARATORS.has(prev!) && !SUBTOKEN_SEPARATORS.has(ch)) f *= p.subtokenInitial;
    if (previous.shifted) f *= p.afterShift;
    if (key.row === 0 && previous.row !== 0) f *= isDigit(ch) ? p.digit : p.numberRowSymbol;
  }
  if (key.finger === 4 && !isLetter(ch) && !isDigit(ch)) f *= p.pinkyPunctuation;
  if (key.shifted) f *= p.shift;
  return f;
}

/** Mean of exp(N(mu, sigma)) is 1 when mu = -sigma^2 / 2. */
const lognormal = (z: number, sigma: number): number => Math.exp(sigma * z - (sigma * sigma) / 2);

function expectedHesitationS(p: TypingProfile): number {
  const mean = p.hesitationMedianS * Math.exp((p.hesitationSigma * p.hesitationSigma) / 2);
  return p.hesitationRate * Math.min(mean, p.hesitationCapS);
}

/** The base interval that makes REFERENCE_TEXT average the profile's speed. */
export function baseInterval(p: TypingProfile = DEFAULT_PROFILE): number {
  const target = 60 / (p.wpm * 5);
  const stall = 1 + p.stallRate * ((p.stallRange[0] + p.stallRange[1]) / 2 - 1);
  let factors = 0;
  let tokenStarts = 0;
  let prev: string | undefined;
  for (const ch of REFERENCE_TEXT) {
    const inWord = !startsToken(prev) && ch !== ' ';
    factors += digraphFactor(prev, ch, p) * (inWord ? stall : 1);
    if (startsToken(prev) && ch !== ' ') tokenStarts += 1;
    prev = ch;
  }
  const n = REFERENCE_TEXT.length;
  const hesitation = (expectedHesitationS(p) * tokenStarts) / n;
  return Math.max(target - hesitation, 0) / (factors / n);
}

/**
 * Seconds to wait before each character of `text`. Every character consumes the same
 * nine random draws, whichever branches apply, so a seed reproduces a take.
 */
export function keyIntervals(text: string, rng: Rng, p: TypingProfile = DEFAULT_PROFILE): number[] {
  const base = baseInterval(p);
  const out: number[] = [];
  let tokenFactor = 1;
  let prev: string | undefined;
  for (const ch of text) {
    const noise = lognormal(rng.gauss(), p.sigma);
    const tokenDraw = rng.gauss();
    const stallDraw = rng.random();
    const stallSize = rng.random();
    const hesitateDraw = rng.random();
    const hesitationDraw = rng.gauss();

    const tokenStart = startsToken(prev) && ch !== ' ';
    if (tokenStart) tokenFactor = lognormal(tokenDraw, p.tokenSigma);
    let delay = base * digraphFactor(prev, ch, p) * (ch === ' ' ? 1 : tokenFactor) * noise;
    if (!startsToken(prev) && ch !== ' ' && stallDraw < p.stallRate) {
      const [low, high] = p.stallRange;
      delay *= low + (high - low) * stallSize;
    }
    if (tokenStart && prev !== undefined && hesitateDraw < p.hesitationRate) {
      const excess = p.hesitationMedianS * Math.exp(p.hesitationSigma * hesitationDraw);
      delay += Math.min(excess, p.hesitationCapS);
    }
    out.push(Math.max(delay, p.floorS));
    prev = ch;
  }
  return out;
}

/** The pause between the last typed key and Enter: log-normal, median 250 ms. */
export function enterPause(rng: Rng): number {
  return 0.25 * Math.exp(0.3 * rng.gauss());
}
