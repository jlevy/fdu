#!/usr/bin/env python3
"""Path-independence matrix for fdu: does an answer depend on cache history, policy, or surface?

Usage: python matrix.py [phase ...]   phases: cold selfwarm warm mutation cross all
Every fdu invocation gets its own XDG_CACHE_HOME under runs/.
Outputs: out/<phase>/results.jsonl, out/<phase>/diffs/*.diff, out/summary-<phase>.txt
"""

import difflib
import json
import os
import shutil
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

DM = Path(__file__).resolve().parent
# The fdu under test: the installed command and the Python with the fdu package, e.g. a
# virtual environment holding a release wheel. Required, so a run never tests a stray build.
FDU = os.environ["FDU_BIN"]
PY = os.environ["FDU_PYTHON"]
TREE = DM / "tree"
RUNS = DM / "runs"
OUT = DM / "out"
WORKERS = 8


def mk(views=None, analyze=None, **kw):
    scope_keys = {"no_gitignore", "budget", "line_limit", "scan_depth", "one_fs"}
    spec = {"scope": {}, "sel": {}}
    for k, v in kw.items():
        (spec["scope"] if k in scope_keys else spec["sel"])[k] = v
    if views:
        spec["views"] = views
    if analyze:
        spec["analyze"] = analyze
    return spec


def cli_args(spec):
    a = []
    sc, se = spec.get("scope", {}), spec.get("sel", {})
    if sc.get("no_gitignore"):
        a.append("--no-gitignore")
    if "budget" in sc:
        a += ["--gitignore-budget", str(sc["budget"])]
    if "line_limit" in sc:
        a += ["--gitignore-line-limit", str(sc["line_limit"])]
    if "scan_depth" in sc:
        a += ["--scan-depth", str(sc["scan_depth"])]
    if sc.get("one_fs"):
        a.append("--one-filesystem")
    for g in se.get("include", []):
        a += ["--include", g]
    for g in se.get("exclude", []):
        a += ["--exclude", g]
    if "min_size" in se:
        a += ["--min-size", str(se["min_size"])]
    if "modified_since" in se:
        a += ["--modified-since", se["modified_since"]]
    if "kind" in se:
        a += ["--kind", ",".join(se["kind"])]
    if se.get("ignored") == "exclude":
        a.append("--exclude-ignored")
    if se.get("ignored") == "only":
        a.append("--only-ignored")
    if "depth" in se:
        a += ["--depth", str(se["depth"])]
    if "limit" in se:
        a += ["--limit", str(se["limit"])]
    if "sort" in se:
        a += ["--sort", se["sort"]]
    if se.get("reverse"):
        a.append("--reverse")
    if "size" in se:
        a += ["--size", se["size"]]
    if spec.get("views"):
        a += ["--view", ",".join(spec["views"])]
    if spec.get("analyze"):
        a += ["--analyze", spec["analyze"]]
    return a


R = {
    # scope
    "default": mk(),
    "nogi": mk(no_gitignore=True),
    "budget1k": mk(budget="1KiB"),
    "budgetall": mk(budget="all"),
    "linelim20": mk(line_limit="20"),
    "scandepth1": mk(scan_depth=1),
    "scandepth2": mk(scan_depth=2),
    "onefs": mk(one_fs=True),
    # selection
    "exclign": mk(ignored="exclude"),
    "onlyign": mk(ignored="only"),
    "incl_rs": mk(include=["*.rs"]),
    "excl_src": mk(exclude=["src/**"]),
    "minsize100": mk(min_size="100"),
    "modsince": mk(modified_since="2023-03-15T00:00:00Z", views=["files"]),
    "kind_file": mk(kind=["file"]),
    "kind_dirsym": mk(kind=["dir", "symlink"], views=["files"]),
    "limit2": mk(limit=2),
    "sort_name": mk(sort="name"),
    "sort_mtime_rev": mk(sort="mtime", reverse=True, views=["files"]),
    "depth1": mk(depth=1),
    "depthall": mk(depth="all"),
    "sizeapp": mk(size="apparent"),
    # views, metadata only
    "v_summary": mk(views=["summary"]),
    "v_tree": mk(views=["tree"]),
    "v_families": mk(views=["families"]),
    "v_types": mk(views=["types"]),
    "v_extensions": mk(views=["extensions"]),
    "v_languages": mk(views=["languages"]),
    "v_largest": mk(views=["largest"]),
    "v_recent": mk(views=["recent"]),
    "v_files": mk(views=["files"]),
    "v_full": mk(views=["full"]),
    "v_documents": mk(views=["documents"]),
    "v_summary_types": mk(views=["summary", "types"]),
    # analysis with implied views
    "a_none": mk(analyze="none"),
    "a_lines": mk(analyze="lines"),
    "a_code": mk(analyze="code"),
    "a_words": mk(analyze="words"),
    "a_all": mk(analyze="all"),
    "a_codewords": mk(analyze="code,words"),
    # analysis x views
    "a_lines_v_languages": mk(analyze="lines", views=["languages"]),
    "a_lines_v_documents": mk(analyze="lines", views=["documents"]),
    "a_lines_v_files": mk(analyze="lines", views=["files"]),
    "a_code_v_documents": mk(analyze="code", views=["documents"]),
    "a_code_v_summary": mk(analyze="code", views=["summary"]),
    "a_code_v_full": mk(analyze="code", views=["full"]),
    "a_words_v_tree": mk(analyze="words", views=["tree"]),
    "a_all_v_full": mk(analyze="all", views=["full"]),
    "a_all_v_files": mk(analyze="all", views=["files"]),
    # analysis x scope/selection
    "a_code_exclign": mk(analyze="code", ignored="exclude"),
    "a_all_nogi": mk(analyze="all", no_gitignore=True),
    "a_lines_budget1k": mk(analyze="lines", budget="1KiB"),
    "a_words_onlyign": mk(analyze="words", ignored="only"),
    "a_code_scandepth1": mk(analyze="code", scan_depth=1),
    "a_lines_incl_rs": mk(analyze="lines", include=["*.rs"]),
    "a_all_sizeapp": mk(analyze="all", size="apparent"),
    "a_code_langs_name_lim1": mk(analyze="code", views=["languages"], sort="name", limit=1),
    # summary tier variants
    "v_summary_exclign": mk(views=["summary"], ignored="exclude"),
    "v_summary_nogi": mk(views=["summary"], no_gitignore=True),
    "v_summary_scandepth1": mk(views=["summary"], scan_depth=1),
    "v_summary_minsize": mk(views=["summary"], min_size="100"),
    "v_summary_a_all": mk(views=["summary"], analyze="all"),
    "v_summary_a_lines": mk(views=["summary"], analyze="lines"),
}

# Warmers: (spec, cache policy used for the warm run)
W = {
    "W_default": (mk(), "auto"),
    "W_nogi": (mk(no_gitignore=True), "auto"),
    "W_all": (mk(analyze="all"), "auto"),
    "W_code": (mk(analyze="code"), "auto"),
    "W_lines": (mk(analyze="lines"), "auto"),
    "W_words": (mk(analyze="words"), "auto"),
    "W_budget1k": (mk(budget="1KiB"), "auto"),
    "W_summary": (mk(views=["summary"]), "auto"),
    "W_scandepth1": (mk(scan_depth=1), "auto"),
    "W_refresh": (mk(), "refresh"),
}


def cmdline(root, spec, policy):
    return [FDU, str(root), "--format", "json", "--cache", policy] + cli_args(spec)


def shq(argv):
    import shlex

    return " ".join(shlex.quote(x) for x in argv)


def run_cli(root, spec, policy, xdg):
    xdg = Path(xdg)
    xdg.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, XDG_CACHE_HOME=str(xdg))
    argv = cmdline(root, spec, policy)
    p = subprocess.run(argv, capture_output=True, text=True, env=env)
    res = {"surface": "cli", "argv": argv, "exit": p.returncode, "stderr": p.stderr.strip()}
    try:
        res["json"] = json.loads(p.stdout)
    except Exception:  # noqa: BLE001
        res["json"] = None
        res["stdout"] = p.stdout[:2000]
    return res


def run_py(root, spec, policy, xdg, mode="report"):
    xdg = Path(xdg)
    xdg.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, XDG_CACHE_HOME=str(xdg))
    job = {"root": str(root), "mode": mode, "cache": policy, "spec": spec}
    p = subprocess.run([PY, str(DM / "pyrun.py"), json.dumps(job)], capture_output=True, text=True, env=env)
    res = {"surface": f"py-{mode}", "job": job, "exit": p.returncode, "stderr": p.stderr.strip()}
    try:
        env_out = json.loads(p.stdout)
        if env_out["ok"]:
            res["json"] = env_out["dict"]
            res["exit"] = 0
        else:
            res["json"] = None
            res["stderr"] = env_out["error"]
            res["exit"] = "exc"
    except Exception:  # noqa: BLE001
        res["json"] = None
        res["stdout"] = p.stdout[:2000]
    return res


def cache_files(xdg):
    d = Path(xdg) / "fdu"
    return sorted(p.name for p in d.iterdir()) if d.exists() else []


def normalize(res):
    if res["json"] is None:
        return None, None
    d = json.loads(json.dumps(res["json"]))
    d.pop("generated_at", None)
    d.pop("scan_started_at", None)
    prov = {"source": d.pop("source", None), "freshness": d.pop("freshness", None)}
    return d, prov


def jdiff(a, b, path=""):
    out = []
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            p = f"{path}.{k}" if path else k
            if k not in a:
                out.append((p, "<absent>", b[k]))
            elif k not in b:
                out.append((p, a[k], "<absent>"))
            else:
                out += jdiff(a[k], b[k], p)
    elif isinstance(a, list) and isinstance(b, list):
        for i in range(max(len(a), len(b))):
            p = f"{path}[{i}]"
            if i >= len(a):
                out.append((p, "<absent>", b[i]))
            elif i >= len(b):
                out.append((p, a[i], "<absent>"))
            else:
                out += jdiff(a[i], b[i], p)
    elif a != b:
        out.append((path, a, b))
    return out


def general(path):
    import re

    return re.sub(r"\[\d+\]", "[]", path)


def compare(base, other):
    """Return a verdict dict comparing `other` against baseline `base`."""
    bn, bp = normalize(base)
    on, op = normalize(other)
    v = {"base_exit": base["exit"], "exit": other["exit"]}
    if bn is None and on is None:
        v["kind"] = "both-error"
        v["same"] = base["exit"] == other["exit"]
        v["stderr_same"] = base["stderr"] == other["stderr"]
        v["base_stderr"], v["stderr"] = base["stderr"], other["stderr"]
        return v
    if bn is None or on is None:
        v["kind"] = "error-mismatch"
        v["same"] = False
        v["base_stderr"], v["stderr"] = base["stderr"], other["stderr"]
        return v
    d = jdiff(bn, on)
    v["kind"] = "json"
    v["same"] = not d
    v["prov_base"], v["prov"] = bp, op
    v["prov_same"] = bp == op
    v["stderr_same"] = base["stderr"] == other["stderr"]
    if base["stderr"] != other["stderr"]:
        v["base_stderr"], v["stderr"] = base["stderr"], other["stderr"]
    v["paths"] = sorted({general(p) for p, _, _ in d})
    v["ndiff"] = len(d)
    v["sample"] = [(p, x, y) for p, x, y in d[:12]]
    return v


def write_diff(phase, name, base, other, verdict, preamble):
    ddir = OUT / phase / "diffs"
    ddir.mkdir(parents=True, exist_ok=True)
    bn, _ = normalize(base)
    on, _ = normalize(other)
    lines = [f"# {name}", *preamble, f"# verdict: {json.dumps({k: verdict[k] for k in verdict if k != 'sample'}, default=str)}", ""]
    for p, x, y in jdiff(bn, on) if (bn is not None and on is not None) else []:
        lines.append(f"~ {p}: {json.dumps(x)} -> {json.dumps(y)}")
    lines.append("")
    a = json.dumps(bn if bn is not None else {"exit": base["exit"], "stderr": base["stderr"]}, indent=1, sort_keys=True).splitlines()
    b = json.dumps(on if on is not None else {"exit": other["exit"], "stderr": other["stderr"]}, indent=1, sort_keys=True).splitlines()
    lines += difflib.unified_diff(a, b, "baseline", name, lineterm="")
    (ddir / f"{name}.diff").write_text("\n".join(lines) + "\n")


class Sink:
    def __init__(self, phase):
        self.phase = phase
        (OUT / phase).mkdir(parents=True, exist_ok=True)
        self.f = open(OUT / phase / "results.jsonl", "w")
        self.lock = threading.Lock()

    def put(self, rec):
        with self.lock:
            self.f.write(json.dumps(rec, default=str) + "\n")
            self.f.flush()

    def close(self):
        self.f.close()


def fresh(path):
    path = Path(path)
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True)
    return path


def pmap(fn, items):
    with ThreadPoolExecutor(WORKERS) as ex:
        return list(ex.map(fn, items))


# ---------------------------------------------------------------- phases


def load_cold():
    return {k: json.loads((OUT / "cold" / f"{k}.json").read_text()) for k in R}


def phase_cold():
    (OUT / "cold").mkdir(parents=True, exist_ok=True)
    sink = Sink("cold")

    def one(rid):
        spec = R[rid]
        base = run_cli(TREE, spec, "off", fresh(RUNS / "cold" / rid / "off"))
        (OUT / "cold" / f"{rid}.json").write_text(json.dumps(base, indent=1))
        # auto on an empty cache should equal off
        xdg = fresh(RUNS / "cold" / rid / "auto")
        first = run_cli(TREE, spec, "auto", xdg)
        v = compare(base, first)
        rec = {"R": rid, "argv": shq(base["argv"]), "exit": base["exit"], "prov": normalize(base)[1],
               "cache_after_auto": cache_files(xdg), "auto_vs_off": v}
        if not v["same"]:
            write_diff("cold", f"{rid}__auto-empty", base, first, v, [f"# base: {shq(base['argv'])}", f"# other: {shq(first['argv'])}"])
        sink.put(rec)
        shutil.rmtree(RUNS / "cold" / rid)

    pmap(one, list(R))
    sink.close()


def phase_selfwarm():
    cold = load_cold()
    sink = Sink("selfwarm")

    def one(rid):
        spec = R[rid]
        xdg = fresh(RUNS / "selfwarm" / rid)
        seq = [("auto1", "auto"), ("auto2", "auto"), ("readonly", "read-only"), ("only", "only")]
        for label, pol in seq:
            res = run_cli(TREE, spec, pol, xdg)
            v = compare(cold[rid], res)
            sink.put({"R": rid, "step": label, "policy": pol, "verdict": v, "cache": cache_files(xdg)})
            if not v["same"] and not (pol == "only" and v["kind"] == "error-mismatch"):
                write_diff("selfwarm", f"{rid}__{label}", cold[rid], res, v,
                           [f"# XDG_CACHE_HOME=$(mktemp -d); run: {shq(cmdline(TREE, spec, 'auto'))} then steps up to {label}: {shq(res['argv'])}"])
        shutil.rmtree(xdg)

    pmap(one, list(R))
    sink.close()


def phase_warm():
    cold = load_cold()
    sink = Sink("warm")
    cases = [(w, r, pol) for w in W for r in R for pol in ("auto", "read-only", "only")]

    def one(case):
        wid, rid, pol = case
        wspec, wpol = W[wid]
        xdg = fresh(RUNS / "warm" / f"{wid}__{rid}__{pol}")
        wres = run_cli(TREE, wspec, wpol, xdg)
        after_w = cache_files(xdg)
        res = run_cli(TREE, R[rid], pol, xdg)
        v = compare(cold[rid], res)
        sink.put({"W": wid, "R": rid, "policy": pol, "w_exit": wres["exit"], "w_prov": normalize(wres)[1],
                  "cache_after_w": after_w, "verdict": v})
        if not v["same"] and not (pol == "only" and v["kind"] == "error-mismatch"):
            write_diff("warm", f"{wid}__{rid}__{pol}", cold[rid], res, v,
                       [f"# export XDG_CACHE_HOME=$(mktemp -d)", f"# {shq(wres['argv'])}", f"# {shq(res['argv'])}",
                        f"# baseline: {shq(cold[rid]['argv'])}"])
        shutil.rmtree(xdg)

    pmap(one, cases)
    sink.close()


def mutate(tree, m):
    t = Path(tree)
    if m == "samesize":
        # same byte length, different lines/words; mtime naturally updated
        old = (t / "docs/notes.txt").read_bytes()
        new = (b"x\n" * (len(old) // 2) + b"y" * (len(old) % 2))
        assert len(new) == len(old)
        (t / "docs/notes.txt").write_bytes(new)
        old = (t / "src/main.rs").read_bytes()
        new = (b"// c\n" * (len(old) // 5)) + b"/" * (len(old) % 5)
        assert len(new) == len(old)
        (t / "src/main.rs").write_bytes(new)
    elif m == "samesize_keepmtime":
        st = os.stat(t / "docs/notes.txt")
        old = (t / "docs/notes.txt").read_bytes()
        new = (b"x\n" * (len(old) // 2) + b"y" * (len(old) % 2))
        (t / "docs/notes.txt").write_bytes(new)
        os.utime(t / "docs/notes.txt", ns=(st.st_atime_ns, st.st_mtime_ns))
        st = os.stat(t / "src/main.rs")
        old = (t / "src/main.rs").read_bytes()
        new = (b"// c\n" * (len(old) // 5)) + b"/" * (len(old) % 5)
        (t / "src/main.rs").write_bytes(new)
        os.utime(t / "src/main.rs", ns=(st.st_atime_ns, st.st_mtime_ns))
    elif m == "touch":
        os.utime(t / "src/lib.py")
    elif m == "add":
        (t / "src/new_module.py").write_text("# new\nprint('added')\n")
    elif m == "delete":
        (t / "docs/notes.txt").unlink()
    elif m == "gitignore_root":
        with open(t / ".gitignore", "a") as f:
            f.write("*.txt\n")
    elif m == "gitignore_samesize":
        (t / "src/.gitignore").write_text("*.py*\n")  # was '*.tmp\n', same length
    elif m == "gitignore_big_samesize":
        s = (t / "big/.gitignore").read_text().replace("*.bak\n", "*.txt\n")
        (t / "big/.gitignore").write_text(s)
    elif m == "symlink":
        (t / "link_to_main").unlink()
        os.symlink("src/lib.py", t / "link_to_main")
    else:
        raise ValueError(m)


MUTATIONS = ["samesize", "samesize_keepmtime", "touch", "add", "delete", "gitignore_root",
             "gitignore_samesize", "gitignore_big_samesize", "symlink"]
MUT_W = ["W_default", "W_all", "W_code", "W_nogi", "W_budget1k"]


def phase_mutation():
    sink = Sink("mutation")

    def one(case):
        m, wid = case
        base = fresh(RUNS / "mut" / f"{m}__{wid}")
        tree = base / "tree"
        shutil.rmtree(tree, ignore_errors=True)
        subprocess.run(["cp", "-Rp", str(TREE), str(tree)], check=True)
        xdg0 = base / "xdg0"
        wspec, wpol = W[wid]
        wres = run_cli(tree, wspec, wpol, xdg0)
        mutate(tree, m)
        for rid, spec in R.items():
            for pol in ("auto", "read-only"):
                xdg = base / f"xdg_{rid}_{pol}"
                subprocess.run(["cp", "-Rp", str(xdg0), str(xdg)], check=True)
                res = run_cli(tree, spec, pol, xdg)
                cold = run_cli(tree, spec, "off", fresh(base / f"cold_{rid}_{pol}"))
                v = compare(cold, res)
                sink.put({"M": m, "W": wid, "R": rid, "policy": pol, "w_exit": wres["exit"], "verdict": v})
                if not v["same"]:
                    write_diff("mutation", f"{m}__{wid}__{rid}__{pol}", cold, res, v,
                               [f"# fixture: make_fixture.sh then mutation '{m}' (see matrix.py mutate())",
                                f"# W: {shq(wres['argv'])}", f"# R: {shq(res['argv'])}"])
                shutil.rmtree(xdg)
                shutil.rmtree(base / f"cold_{rid}_{pol}")
        shutil.rmtree(base)

    pmap(one, [(m, w) for m in MUTATIONS for w in MUT_W])
    sink.close()


CROSS_W = ["W_default", "W_all", "W_lines", "W_summary"]


def phase_cross():
    cold = load_cold()
    sink = Sink("cross")
    cases = []
    for rid in R:
        cases.append(("cold", None, rid))
        for w in CROSS_W:
            cases.append(("warm-cli", w, rid))
            cases.append(("warm-py", w, rid))

    def one(case):
        kind, wid, rid = case
        spec = R[rid]
        tag = f"{kind}__{wid}__{rid}"
        recs = []
        if kind == "cold":
            for label, fn in [
                ("py-report-off", lambda x: run_py(TREE, spec, "off", x)),
                ("py-report-auto-empty", lambda x: run_py(TREE, spec, "auto", x)),
                ("py-open-off", lambda x: run_py(TREE, spec, "off", x, "open")),
                ("py-open-auto-empty", lambda x: run_py(TREE, spec, "auto", x, "open")),
                ("py-scan", lambda x: run_py(TREE, spec, "off", x, "scan")),
            ]:
                xdg = fresh(RUNS / "cross" / f"{tag}__{label}")
                res = fn(xdg)
                recs.append((label, None, res, cache_files(xdg)))
                shutil.rmtree(xdg)
        elif kind == "warm-cli":
            # CLI warmer, Python reader
            wspec, wpol = W[wid]
            for label, pol, mode in [("py-report-auto", "auto", "report"), ("py-report-readonly", "read-only", "report"),
                                     ("py-report-only", "only", "report"), ("py-open-auto", "auto", "open"),
                                     ("py-open-only", "only", "open")]:
                xdg = fresh(RUNS / "cross" / f"{tag}__{label}")
                wres = run_cli(TREE, wspec, wpol, xdg)
                res = run_py(TREE, spec, pol, xdg, mode)
                recs.append((label, wres, res, None))
                shutil.rmtree(xdg)
        else:
            # Python warmer (report and open), CLI reader
            wspec, _ = W[wid]
            for wmode in ("report", "open"):
                for pol in ("auto", "only"):
                    label = f"pyW-{wmode}__cli-{pol}"
                    xdg = fresh(RUNS / "cross" / f"{tag}__{label}")
                    wres = run_py(TREE, wspec, "auto", xdg, wmode)
                    after = cache_files(xdg)
                    res = run_cli(TREE, spec, pol, xdg)
                    recs.append((label, wres, res, after))
                    shutil.rmtree(xdg)
        for label, wres, res, cf in recs:
            v = compare(cold[rid], res)
            sink.put({"case": kind, "W": wid, "R": rid, "label": label, "w_exit": wres and wres["exit"],
                      "w_err": wres and (wres["stderr"] if wres["json"] is None else None), "cache": cf, "verdict": v})
            if not v["same"] and not ("only" in label and v["kind"] == "error-mismatch"):
                pre = [f"# baseline CLI: {shq(cold[rid]['argv'])}"]
                if wres:
                    pre.append(f"# warmer: {shq(wres['argv']) if 'argv' in wres else json.dumps(wres['job'])}")
                pre.append(f"# reader: {shq(res['argv']) if 'argv' in res else json.dumps(res['job'])}")
                write_diff("cross", f"{kind}__{wid}__{rid}__{label}", cold[rid], res, v, pre)

    pmap(one, cases)
    sink.close()


PHASES = {"cold": phase_cold, "selfwarm": phase_selfwarm, "warm": phase_warm, "mutation": phase_mutation, "cross": phase_cross}

if __name__ == "__main__":
    want = sys.argv[1:] or ["all"]
    if "all" in want:
        want = list(PHASES)
    if not TREE.exists():
        subprocess.run([str(DM / "make_fixture.sh"), str(TREE)], check=True)
    for ph in want:
        if ph in ("selfwarm", "warm", "mutation", "cross") and not (OUT / "cold").exists():
            phase_cold()
        d = OUT / ph
        if d.exists() and ph != "cold":
            shutil.rmtree(d)
        import time

        t0 = time.time()
        PHASES[ph]()
        print(f"{ph}: {time.time() - t0:.1f}s", flush=True)
