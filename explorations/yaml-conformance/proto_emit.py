"""Prototype policies: (B) strict scalar policy for a Rust utility, and a fixed frontmatter-format represent_str."""
import io, json, re
corpus = json.load(open("corpus.json"))

# ---- (B) strict policy: plain only when unambiguous under YAML 1.1 AND 1.2 (spec + known parsers) ----
KEYWORDS = {"true", "false", "yes", "no", "on", "off", "y", "n", "null"}
DOT_NUMERIC = re.compile(r"\.[0-9_.]*(?:[eE][-+]?[0-9_]*)?|\.(?:inf|nan)", re.I)
ESCAPE = {'"': '\\"', "\\": "\\\\", "\n": "\\n", "\r": "\\r", "\t": "\\t"}

def must_escape(ch):
    c = ord(ch)
    return c < 0x20 or 0x7F <= c <= 0x9F or c in (0x2028, 0x2029, 0xFEFF, 0xFFFE, 0xFFFF)

def dq(s):  # JSON-compatible double-quoted YAML scalar
    return '"' + "".join(ESCAPE.get(ch) or (f"\\u{ord(ch):04x}" if must_escape(ch) else ch) for ch in s) + '"'

def plain_ok(s):
    return (s != ""
        and all(ch.isascii() and (ch.isalnum() or ch in "._/-+") for ch in s)
        and not (s[0].isdigit() or s[0] in "-+")
        and not DOT_NUMERIC.fullmatch(s)
        and s.lower() not in KEYWORDS)

def b_scalar(s): return s if plain_ok(s) else dq(s)

# ---- frontmatter-format fix: keep ruamel's syntax analysis, but force double quotes whenever
# any YAML 1.1 or 1.2 implicit resolver would claim the plain text, or the text holds
# characters the PyYAML-lineage emitter mishandles; use '|' only for block-safe text. ----
from ruamel.yaml.resolver import VersionedResolver
_r11, _r12 = VersionedResolver(version=(1, 1)), VersionedResolver(version=(1, 2))
SPEC11_FLOAT = re.compile(r"[-+]?(?:[0-9][0-9_]*)?\.[0-9.]*(?:[eE][-+][0-9]+)?")  # yaml.org/type/float.html
def _implicit_nonstr(s):
    from ruamel.yaml.nodes import ScalarNode
    for r in (_r11, _r12):
        if r.resolve(ScalarNode, s, (True, False)) != "tag:yaml.org,2002:str": return True
    return bool(SPEC11_FLOAT.fullmatch(s)) or s in ("=", "<<") or s.lower() in KEYWORDS
BAD_ANYWHERE = re.compile("[\x00-\x08\x0b-\x1f\x7f-\x9f  ﻿￾￿]")
def block_safe(s):
    lines = s.rstrip("\n").split("\n")
    return ("\n" in s and "\r" not in s and "\t" not in s and not BAD_ANYWHERE.search(s)
            and not lines[0].startswith(" ") and all(l.strip() for l in lines))

def fixed_frontmatter_yaml():
    from frontmatter_format.yaml_util import new_yaml
    y = new_yaml(typ="rt")
    def represent_str(dumper, data):
        if block_safe(data): style = "|"
        elif _implicit_nonstr(data) or BAD_ANYWHERE.search(data) or "\r" in data or "\n" in data: style = '"'
        else: style = None
        return dumper.represent_scalar("tag:yaml.org,2002:str", data, style=style)
    y.representer.add_representer(str, represent_str)
    return y

y = fixed_frontmatter_yaml()
def fm_fixed(s):
    b = io.StringIO(); y.dump({"path": s}, b); return b.getvalue()

with open("emit-proto.tsv", "w") as f:
    for name, fn in [("proposed-B-strict-policy", lambda s: f"path: {b_scalar(s)}\n"), ("frontmatter-format-fixed-prototype", fm_fixed)]:
        for c in corpus:
            f.write(f"{name}\t{c['id']}\tOK\t{fn(c['s']).encode().hex()}\n")
