import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { parse, stringify } from 'yaml';
const corpus = new Map(JSON.parse(readFileSync('corpus.json', 'utf8')).map(c => [c.id, c]));
// npm yaml as an emitter too, both schema versions
const emitLines = [];
for (const [id, c] of corpus) {
  for (const version of ['1.2', '1.1']) {
    try {
      const y = stringify({ path: c.s }, { version });
      emitLines.push(`npm-yaml-stringify(${version})\t${id}\tOK\t${Buffer.from(y, 'utf8').toString('hex')}`);
    } catch (e) {
      emitLines.push(`npm-yaml-stringify(${version})\t${id}\tEMITERR\t${Buffer.from(String(e), 'utf8').toString('hex')}`);
    }
  }
}
writeFileSync('emit-node.tsv', emitLines.join('\n') + '\n');
const out = [];
for (const f of readdirSync('.').filter(f => f.startsWith('emit-') && f.endsWith('.tsv')).sort()) {
  for (const line of readFileSync(f, 'utf8').split('\n')) {
    if (!line) continue;
    const [name, idS, status, h] = line.split('\t');
    const id = Number(idS); const s = corpus.get(id).s;
    for (const version of ['1.2', '1.1']) {
      const p = `npm-yaml(${version})`;
      if (status !== 'OK') { out.push([name, id, p, 'emit-error', '']); continue; }
      const text = Buffer.from(h, 'hex').toString('utf8');
      try {
        const v = parse(text, { version, strict: true, uniqueKeys: true, prettyErrors: false });
        const got = v && typeof v === 'object' ? v.path : v;
        if (typeof got === 'string' && got === s) out.push([name, id, p, 'ok', '']);
        else out.push([name, id, p, 'wrong', `${typeof got}:${JSON.stringify(got)}`.slice(0, 80)]);
      } catch (e) {
        out.push([name, id, p, 'error', String(e.message).split('\n')[0].slice(0, 80)]);
      }
    }
  }
}
writeFileSync('results-node.json', JSON.stringify(out));
console.log(out.length, 'node parse results');
