import json, sys, glob
import yaml as pyyaml
from ruamel.yaml import YAML
corpus = {c["id"]: c for c in json.load(open("corpus.json"))}

def rt_load(t):
    y = YAML(typ="rt"); return y.load(t)
def safe_load_ruamel(t):
    y = YAML(typ="safe", pure=True); return y.load(t)
parsers = {"PyYAML(1.1,pure)": pyyaml.safe_load, "ruamel-safe(1.2)": safe_load_ruamel, "ruamel-rt(1.2)": rt_load}
if hasattr(pyyaml, "CSafeLoader"):
    parsers["PyYAML(1.1,libyaml C)"] = lambda t: pyyaml.load(t, Loader=pyyaml.CSafeLoader)

out = []
for path in sorted(glob.glob("emit-*.tsv")):
    for line in open(path):
        name, id_, status, h = line.rstrip("\n").split("\t")
        s = corpus[int(id_)]["s"]
        if status != "OK":
            for p in parsers: out.append([name, int(id_), p, "emit-error", bytes.fromhex(h).decode()[:80]])
            continue
        text = bytes.fromhex(h).decode("utf-8")
        for p, fn in parsers.items():
            try:
                v = fn(text)
                got = v.get("path", "<missing>") if hasattr(v, "get") else v
                if isinstance(got, str) and type(got) is not str: got = str(got)
                if type(got) is str and got == s: out.append([name, int(id_), p, "ok", ""])
                else: out.append([name, int(id_), p, "wrong", f"{type(got).__name__}:{got!r}"[:80]])
            except Exception as e:
                out.append([name, int(id_), p, "error", f"{type(e).__name__}: {str(e).splitlines()[0] if str(e) else ''}"[:80]])
json.dump(out, open("results-py.json", "w"))
print(len(out), "python parse results")
