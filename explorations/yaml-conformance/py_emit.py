"""Emit {path: s} for every corpus string with Python-side emitters and fdu rule re-implementations."""
import io, json, math

corpus = json.load(open("corpus.json"))

def rust_f64_parses(v: str) -> bool:
    # Rust's str::parse::<f64> grammar (core::num::dec2flt): optional sign, then
    # "inf" | "infinity" | "nan" (case-insensitive), or decimal digits with optional '.'
    # and optional exponent; at least one digit in the mantissa; no underscores/whitespace.
    import re
    return re.fullmatch(r"[+-]?(?:(?i:inf|infinity|nan)|(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?)", v) is not None

def fdu_quote(text: str, branch: bool) -> str:
    out = ['"']
    for ch in text:
        c = ord(ch)
        if ch == '"': out.append('\\"')
        elif ch == '\\': out.append('\\\\')
        elif ch == '\n': out.append('\\n')
        elif ch == '\r': out.append('\\r')
        elif ch == '\t': out.append('\\t')
        elif c < 0x20 or (branch and (0x7f <= c <= 0x9f or ch in '￾￿')):
            out.append('\\u%04x' % c)
        else: out.append(ch)
    out.append('"')
    return "".join(out)

def fdu_scalar(value: str, branch: bool) -> str:
    charset_ok = all((ch.isascii() and ch.isalnum()) or ch in "._/-+" for ch in value)
    if branch:
        start_ok = not (value[:1].isdigit() or value[:1] in ("-", "+")) if value else True
        words = {"true","false","null","yes","no","on","off","y","n","~",".inf",".nan"}
    else:
        start_ok = not value.startswith("-")
        words = {"true","false","null","yes","no","on","off","~"}
    safe = (value != "" and charset_ok and start_ok and not rust_f64_parses(value)
            and value.lower() not in words)
    # Rust to_ascii_lowercase only folds ASCII; the charset check already restricts to ASCII.
    return value if safe else fdu_quote(value, branch)

def emit_all():
    import yaml as pyyaml
    from ruamel.yaml import YAML
    from frontmatter_format.yaml_util import to_yaml_string
    emitters = {
        "fdu-release-0.1.0-rules": lambda s: f"path: {fdu_scalar(s, False)}\n",
        "fdu-branch-rules": lambda s: f"path: {fdu_scalar(s, True)}\n",
        "pyyaml-safe_dump": lambda s: pyyaml.safe_dump({"path": s}, allow_unicode=True, sort_keys=False),
        "pyyaml-safe_dump-ascii": lambda s: pyyaml.safe_dump({"path": s}, sort_keys=False),
    }
    def ruamel_safe(s):
        y = YAML(typ="safe", pure=True); y.default_flow_style = False
        b = io.StringIO(); y.dump({"path": s}, b); return b.getvalue()
    emitters["ruamel-safe-pure"] = ruamel_safe
    emitters["frontmatter-format(ruamel rt)"] = lambda s: to_yaml_string({"path": s})
    with open("emit-py.tsv", "w") as f:
        for name, fn in emitters.items():
            for c in corpus:
                try:
                    y = fn(c["s"]); f.write(f"{name}\t{c['id']}\tOK\t{y.encode('utf-8').hex()}\n")
                except Exception as e:
                    f.write(f"{name}\t{c['id']}\tEMITERR\t{repr(e).encode().hex()}\n")

if __name__ == "__main__":
    # self-check the f64 grammar against known Rust behaviour
    for v, want in [("1e3", True), ("inf", True), ("Infinity", True), ("NaN", True), (".5", True), ("1.", True), ("1_000", False), ("0x10", False), (".", False), ("e3", False), ("+.5", True), ("._1", False)]:
        assert rust_f64_parses(v) == want, v
    emit_all()
