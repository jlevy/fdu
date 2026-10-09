// Directories whose newest activity sits a fixed distance before now, so a tree row's age
// reads one stable value on every route. Each time is stamped in the middle of its unit,
// half a day past 3d, half a month past 2mo, and half a year past 1y, so a run that takes
// seconds cannot cross a unit boundary.
//
// `installed/` is a package installed three days ago from an archive that kept its
// one-year-old times: by its files alone it would read 1y, and by its activity, which
// counts its own time, it reads 3d. `docs/` holds a file newer than the directory.
const fs = require('node:fs');
const path = require('node:path');

const now = Date.now();
const day = 24 * 60 * 60 * 1000;
const ago = (days) => new Date(now - days * day);
const threeDays = ago(3.5);
const twoMonths = ago(75);
const oneYear = ago(548);

for (const [name, size, time] of [
  ['installed/pkg/index.js', 400, oneYear],
  ['docs/notes.md', 300, twoMonths],
  ['old/data.bin', 200, oneYear],
]) {
  const file = path.join('ages', name);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, Buffer.alloc(size));
  fs.utimesSync(file, time, time);
}
// Directories last, deepest first: creating a child sets its parent's time.
for (const [name, time] of [
  ['installed/pkg', oneYear],
  ['installed', threeDays],
  ['docs', oneYear],
  ['old', oneYear],
]) {
  const dir = path.join('ages', name);
  fs.utimesSync(dir, time, time);
}
