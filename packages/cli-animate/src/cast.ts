/**
 * asciicast reading and writing (https://docs.asciinema.org/manual/asciicast/v3/).
 *
 * Inside this tool every event carries its absolute time; v3 files store intervals. The
 * recorder marks step boundaries in-band with an OSC sequence no terminal acts on, and
 * `extractMarkers` turns those into asciicast `m` events after recording.
 */

export type EventCode = 'o' | 'i' | 'm' | 'r' | 'x';

export interface CastEvent {
  time: number;
  code: EventCode;
  data: string;
}

export interface CastHeader {
  version: 3;
  term: { cols: number; rows: number; type?: string };
  timestamp?: number;
  title?: string;
  env?: Record<string, string>;
  [key: string]: unknown;
}

export interface Cast {
  header: CastHeader;
  events: CastEvent[];
}

/** The in-band step marker the driver writes: ESC ] 1337 ; cli-animate=<label> BEL. */
export const MARKER_PREFIX = '\u001b]1337;cli-animate=';
const MARKER = /\u001b\]1337;cli-animate=([^\u0007]*)\u0007/g;

export function markerSequence(label: string): string {
  return `${MARKER_PREFIX}${label.replace(/[\u0000-\u001f]/g, ' ')}\u0007`;
}

/** Parse an asciicast v2 or v3 file into absolute-time events and a v3 header. */
export function parseCast(text: string): Cast {
  const lines = text.split('\n').filter((line) => line.trim() !== '');
  if (lines.length === 0) throw new Error('empty cast file');
  const raw = JSON.parse(lines[0]!) as Record<string, unknown>;
  const version = raw.version;
  if (version !== 2 && version !== 3) throw new Error(`unsupported asciicast version ${String(version)}`);
  const header: CastHeader =
    version === 3
      ? (raw as unknown as CastHeader)
      : {
          ...raw,
          version: 3,
          term: { cols: Number(raw.width), rows: Number(raw.height) },
        };
  let time = 0;
  const events = lines.slice(1).map((line, index) => {
    const event = JSON.parse(line) as unknown;
    if (!Array.isArray(event) || event.length < 3) throw new Error(`event ${index + 1} is not [time, code, data]`);
    const [t, code, data] = event as [number, EventCode, unknown];
    time = version === 3 ? time + t : t;
    return { time, code, data: typeof data === 'string' ? data : JSON.stringify(data) };
  });
  return { header, events };
}

/** Serialize as asciicast v3, with intervals rounded to microseconds. */
export function formatCast(cast: Cast): string {
  const lines = [JSON.stringify(cast.header)];
  let previous = 0;
  for (const event of cast.events) {
    const interval = Math.round((event.time - previous) * 1e6) / 1e6;
    lines.push(JSON.stringify([interval, event.code, event.data]));
    previous = event.time;
  }
  return `${lines.join('\n')}\n`;
}

/**
 * Replace in-band markers in output events with `m` events at the same instant, and drop
 * output that becomes empty.
 */
export function extractMarkers(events: CastEvent[]): CastEvent[] {
  const out: CastEvent[] = [];
  for (const event of events) {
    if (event.code !== 'o' || !event.data.includes(MARKER_PREFIX)) {
      out.push(event);
      continue;
    }
    let last = 0;
    for (const match of event.data.matchAll(MARKER)) {
      const before = event.data.slice(last, match.index);
      if (before) out.push({ time: event.time, code: 'o', data: before });
      out.push({ time: event.time, code: 'm', data: match[1] ?? '' });
      last = match.index + match[0].length;
    }
    const rest = event.data.slice(last);
    if (rest) out.push({ time: event.time, code: 'o', data: rest });
  }
  return out;
}

/** The first time each marker label occurs. */
export function markerTimes(events: CastEvent[]): Map<string, number> {
  const times = new Map<string, number>();
  for (const event of events) {
    if (event.code === 'm' && !times.has(event.data)) times.set(event.data, event.time);
  }
  return times;
}

export function duration(events: CastEvent[]): number {
  return events.length === 0 ? 0 : events[events.length - 1]!.time;
}
