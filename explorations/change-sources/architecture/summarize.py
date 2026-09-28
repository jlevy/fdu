#!/usr/bin/env python3
"""Medians and spreads per scope/policy from bench-*.tsv, watch-first.tsv, phase4.tsv."""
import csv, glob, os, statistics, sys
R = sys.argv[1]
def med(xs): return statistics.median(xs) if xs else float('nan')
for tsv in sorted(glob.glob(os.path.join(R, 'bench-*.tsv'))):
    rows = list(csv.DictReader(open(tsv), delimiter='\t'))
    rows = [r for r in rows if r.get('label') and r['label'] != 'label']
    if not rows: continue
    label = rows[0]['label']
    summ = next((r['summary'] for r in rows if r['policy'] == 'off'), '')
    print(f"\n== {label}: {summ}")
    by = {}
    for r in rows:
        if r['round'] == '0': print(f"   round0 auto: real={r['real_s']} rss={int(r['max_rss_bytes'])//2**20} MB rc={r['rc']} files={r['files_walked']}"); continue
        by.setdefault(r['policy'], []).append(r)
    for pol, rs in by.items():
        reals = [float(r['real_s']) for r in rs]; rss = [int(r['max_rss_bytes']) for r in rs]
        us = [float(r['user_s']) for r in rs]; sy = [float(r['sys_s']) for r in rs]
        rcs = sorted(set(r['rc'] for r in rs))
        print(f"   {pol:18s} median {med(reals):6.2f}s  spread {min(reals):.2f}-{max(reals):.2f}  user {med(us):.2f} sys {min(sy):.1f}-{max(sy):.1f}  RSS {med(rss)//2**20:.0f} MB  rc={rcs}  n={len(rs)}")
wf = os.path.join(R, 'watch-first.tsv')
if os.path.exists(wf):
    print("\n== watch-first (load + full reconcile), read-only")
    for r in csv.DictReader(open(wf), delimiter='\t'):
        print(f"   {r['label']:14s} first_report={r['first_report_s']}s source={r['source']} rss={int(r['rss_kib'] or 0)//1024} MB  err={r['stderr'][:80]}")
p4 = os.path.join(R, 'phase4.tsv')
if os.path.exists(p4):
    print("\n== phase4 (no gitignore: tree=index retained vs summary=reducer)")
    by = {}
    for r in csv.DictReader(open(p4), delimiter='\t'):
        by.setdefault((r['label'], r['policy']), []).append(r)
    for (l, p), rs in sorted(by.items()):
        reals = [float(r['real_s']) for r in rs]; rss = [int(r['max_rss_bytes']) for r in rs]
        print(f"   {l:8s} {p:26s} median {med(reals):6.2f}s spread {min(reals):.2f}-{max(reals):.2f} RSS {med(rss)//2**20:.0f} MB n={len(rs)}")
