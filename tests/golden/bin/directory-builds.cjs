// Real stale build directories, with one recent descendant and an empty match.
const fs = require('node:fs');
const path = require('node:path');

const old = new Date('2000-01-01T00:00:00Z');
for (const [name, size] of [
  ['a/.venv', 30], ['b/node_modules', 50], ['c/target', 70],
  ['d/.venv', 90], ['empty/.venv', 0],
]) {
  const dir = path.join('builds', name);
  fs.mkdirSync(dir, { recursive: true });
  if (size) {
    const file = path.join(dir, 'payload');
    fs.writeFileSync(file, Buffer.alloc(size));
    fs.utimesSync(file, old, old);
  }
  fs.utimesSync(dir, old, old);
}
// Updating a descendant, without touching the directory, must still make it recent.
const now = new Date();
fs.utimesSync(path.join('builds', 'd', '.venv', 'payload'), now, now);
