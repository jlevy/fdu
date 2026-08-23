---
sandbox: true
path:
  - $FDU_BIN
fixtures:
  - fixtures/project
env:
  FORCE_COLOR: "0"
  LANG: C
  LC_ALL: C
  NO_COLOR: "1"
  TZ: UTC
  XDG_CACHE_HOME: .cache
---
# What a Run Costs

Every other golden pins what fdu *answered*. This one pins what it *did*.

The two come apart in the direction that is hardest to notice: a change can leave every
byte of output identical and quietly double the work behind it, and no test that
compares output would say a word.
`FDU_COUNTERS=1` already records the work; the missing half was a machine form and
somewhere to assert on it.

Absolute counts are not that place.
They move with the filesystem, the platform, and the allocator, so a golden pinning
`stats: 10` fails on the next machine for reasons that have nothing to do with fdu.
*Relations between* counts do not move — every entry is statted once, a cache-only
answer stats nothing at all — and those are what these sessions compute.

Nothing here is wall-clock.
A timing gate on a shared runner measures the runner.

## A Cold Walk Stats Each Entry Once and Opens Each Directory Once

The floor this engine is measured against.
`stats` exceeds `dir_entries` by exactly one because the root is statted and is nobody’s
directory entry; `dir_opens` exceeds the reported directory count by one for the same
reason.

```console
$ node -e "const {spawnSync}=require('node:child_process'); const counters=(args)=>{const run=spawnSync(process.env.FDU,args,{encoding:'utf8',env:{...process.env,FDU_COUNTERS:'1'}}); const line=run.stderr.split('\n').find((l)=>l.startsWith('__FDU_COUNTERS__=')); if(!line){console.log(JSON.stringify({instrumented:false})); process.exit(0);} return {counts:JSON.parse(line.slice('__FDU_COUNTERS__='.length)),report:JSON.parse(run.stdout)};}; const cold=counters(['--cache','off','--color','never','--view','summary','--format','json','project']); const dirs=cold.report.reports[0].summary.dirs; console.log(JSON.stringify({schema:cold.counts.schema,stats_cover_every_entry_and_the_root:cold.counts.stats===cold.counts.dir_entries+1,every_directory_opened_once:cold.counts.dir_opens===dirs+1,metadata_only_reads_no_file_bodies:cold.counts.file_opens===0&&cold.counts.bytes_read===0}));"
{"schema":"fdu.counters/1","stats_cover_every_entry_and_the_root":true,"every_directory_opened_once":true,"metadata_only_reads_no_file_bodies":true}
? 0
```

## Cache-Only Touches Nothing

`--cache only` promises to answer from the snapshot without reading the tree.
Until now that promise was checked by reading `source: cache_only` off the report, which
is fdu reporting on itself.
This checks the syscalls.

The warming run asks for a tree rather than a summary on purpose: a single unfiltered
summary is answered by a transient tier that retains no index, so it leaves no snapshot
to read back.

```console
$ node -e "const {spawnSync}=require('node:child_process'); const warm=spawnSync(process.env.FDU,['--color','never','--view','tree','--format','json','project'],{encoding:'utf8'}); if(warm.status!==0) throw new Error(warm.stderr); const run=spawnSync(process.env.FDU,['--cache','only','--color','never','--view','summary','--format','json','project'],{encoding:'utf8',env:{...process.env,FDU_COUNTERS:'1'}}); const line=run.stderr.split('\n').find((l)=>l.startsWith('__FDU_COUNTERS__=')); if(!line){console.log(JSON.stringify({instrumented:false})); process.exit(0);} const counts=JSON.parse(line.slice('__FDU_COUNTERS__='.length)); const report=JSON.parse(run.stdout); console.log(JSON.stringify({source:report.source,touched_the_tree:counts.dir_opens>0||counts.dir_entries>0||counts.stats>0}));"
{"source":"cache_only","touched_the_tree":false}
? 0
```

## Analysis Reads File Bodies, and Only Analysis Does

The one setting that makes a run cost more than a metadata walk, shown as the syscalls
it adds rather than as a flag it accepts.

```console
$ node -e "const {spawnSync}=require('node:child_process'); const counters=(args)=>{const run=spawnSync(process.env.FDU,args,{encoding:'utf8',env:{...process.env,FDU_COUNTERS:'1'}}); const line=run.stderr.split('\n').find((l)=>l.startsWith('__FDU_COUNTERS__=')); if(!line){console.log(JSON.stringify({instrumented:false})); process.exit(0);} return JSON.parse(line.slice('__FDU_COUNTERS__='.length));}; const bare=counters(['--cache','off','--color','never','--view','types','--format','json','project']); const read=counters(['--cache','off','--color','never','--analyze','lines','--view','types','--format','json','project']); console.log(JSON.stringify({metadata_only_opens_no_files:bare.file_opens===0,analysis_opens_and_reads:read.file_opens>0&&read.bytes_read>0,same_walk_either_way:bare.dir_entries===read.dir_entries}));"
{"metadata_only_opens_no_files":true,"analysis_opens_and_reads":true,"same_walk_either_way":true}
? 0
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
