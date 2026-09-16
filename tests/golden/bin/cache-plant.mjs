#!/usr/bin/env node
// Plant cache files an earlier or foreign program could have left, so the lifecycle
// flags can be goldened against them.
//
// No build under test can write a snapshot in an older format or under another engine
// fingerprint, and those are exactly the files a release upgrade leaves behind. This
// helper makes them from the one snapshot fdu just wrote, touching only the prologue
// every format shares: the eight-byte magic, then the format version, then the engine
// fingerprint. It never runs fdu and prints no product output, only what it planted.
//
// Usage: cache-plant <stale|directory|other-engine|foreign|leftovers>
//
//   stale         beside the current snapshot, add one in format version 1, a copy
//                 under another engine fingerprint, a truncated copy, and a file that is
//                 not a snapshot at all
//   directory     add a directory under a snapshot's name, so a listing has to describe
//                 something that is not a regular file and has no size to report
//   other-engine  give the current snapshot another engine fingerprint, in place
//   foreign       replace the current snapshot with a file that is not a snapshot
//   leftovers     add what a killed writer leaves: a staging file too old to be anyone's,
//                 one young enough to be a live writer's, and a sidecar with no snapshot.
//                 Ages are set outright, so the session never waits for a clock.

import { mkdirSync, readdirSync, readFileSync, utimesSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// The cache directory the sessions set through XDG_CACHE_HOME.
const CACHE_DIR = ".cache/fdu";
const MAGIC = Buffer.from("FDUSNAP\0", "latin1");
const VERSION_OFFSET = MAGIC.length;
const FINGERPRINT_OFFSET = VERSION_OFFSET + 4;
const FINGERPRINT_BYTES = 8;
const CONTENT_MAGIC = Buffer.from("FDUCTNT\0planted", "latin1");
// Older than the day a writer's own reaper waits, so a clear may take it.
const BEYOND_THE_REAPER = new Date(Date.now() - 2 * 24 * 60 * 60 * 1000);
// Every planted name sorts ahead of any real root hash, so listings stay in a fixed
// order whatever the sandbox path hashes to.
const PLANTED_PREFIX = "000000000000000";

const action = process.argv[2];

// The snapshot fdu wrote for the tree under test: the one layout name not planted here.
const written = readdirSync(CACHE_DIR).filter(
  (name) => /^[0-9a-f]{16}\.fdu$/.test(name) && !name.startsWith(PLANTED_PREFIX),
);
if (written.length !== 1) {
  console.error(`cache-plant: expected one snapshot in ${CACHE_DIR}, found ${written.length}`);
  process.exit(2);
}
const currentPath = join(CACHE_DIR, written[0]);
const current = readFileSync(currentPath);

function withOtherEngine(image) {
  const copy = Buffer.from(image);
  for (let offset = FINGERPRINT_OFFSET; offset < FINGERPRINT_OFFSET + FINGERPRINT_BYTES; offset += 1) {
    copy[offset] ^= 0xff;
  }
  return copy;
}

switch (action) {
  case "stale": {
    const version = Buffer.alloc(4);
    version.writeUInt32LE(1);
    writeFileSync(join(CACHE_DIR, `${PLANTED_PREFIX}1.fdu`), Buffer.concat([MAGIC, version]));
    writeFileSync(join(CACHE_DIR, `${PLANTED_PREFIX}2.fdu`), withOtherEngine(current));
    writeFileSync(join(CACHE_DIR, `${PLANTED_PREFIX}3.fdu`), current.subarray(0, current.length - 1));
    writeFileSync(join(CACHE_DIR, "notes.txt"), "not a snapshot\n");
    console.log("planted: format 1, another engine, truncated, notes.txt");
    break;
  }
  case "directory":
    // Under a snapshot's own name, so the name cannot be what saves it from a clear.
    mkdirSync(join(CACHE_DIR, `${PLANTED_PREFIX}7.fdu`), { recursive: true });
    console.log("planted: a directory under a snapshot's name");
    break;
  case "other-engine":
    writeFileSync(currentPath, withOtherEngine(current));
    console.log("planted: another engine");
    break;
  case "foreign":
    writeFileSync(currentPath, "not a snapshot");
    console.log("planted: not a snapshot");
    break;
  case "leftovers": {
    const abandoned = join(CACHE_DIR, `.${PLANTED_PREFIX}4.fdu.tmp.1.0011223344556677.0`);
    writeFileSync(abandoned, current);
    utimesSync(abandoned, BEYOND_THE_REAPER, BEYOND_THE_REAPER);
    writeFileSync(join(CACHE_DIR, `.${PLANTED_PREFIX}6.fdu.tmp.1.0011223344556677.0`), current);
    writeFileSync(join(CACHE_DIR, `${PLANTED_PREFIX}5.fdu.content`), CONTENT_MAGIC);
    console.log("planted: abandoned staging file, in-flight staging file, orphaned sidecar");
    break;
  }
  default:
    console.error("usage: cache-plant <stale|directory|other-engine|foreign|leftovers>");
    process.exit(2);
}
