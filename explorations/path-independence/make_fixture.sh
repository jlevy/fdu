#!/bin/bash
# Build the path-independence fixture at $1 (removed first).
set -euo pipefail
R="$1"
rm -rf "$R"
mkdir -p "$R"/{src/nested/deep,docs,data,build/obj,empty,big,vendor}
cd "$R"
# Root .gitignore ignores a directory and a pattern
printf 'build/\n*.log\n' > .gitignore
# Nested .gitignore ignores a pattern
printf '*.tmp\n' > src/.gitignore
# Large .gitignore (>1KiB) that ignores *.bak and vendor-ish stuff
{ for i in $(seq 1 120); do echo "generated_pattern_number_$i/"; done; echo '*.bak'; } > big/.gitignore
cat > src/main.rs <<'X'
// Entry point
fn main() {
    // say hello
    println!("hello world");

    let x = 1 + 2;
    println!("{}", x);
}
X
cat > src/nested/deep/util.rs <<'X'
/// Adds numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
X
cat > src/lib.py <<'X'
"""Module docstring."""
# comment line

def f(x):
    return x * 2  # trailing


class A:
    pass
X
cat > src/Main.hs <<'X'
-- Haskell comment
module Main where

main :: IO ()
main = putStrLn "hi"
X
printf 'scratch data that is ignored by pattern\n' > src/scratch.tmp
cat > docs/readme.md <<'X'
# Title

Some prose words here, with *emphasis* and a [link](http://example.com).

- list item one
- list item two

```rust
fn code_in_doc() {}
```
X
cat > docs/notes.txt <<'X'
Plain text notes. These are several words of prose
spread over two lines.

A second paragraph follows.
X
head -c 9000 /dev/urandom > data/blob.bin 2>/dev/null || dd if=/dev/urandom of=data/blob.bin bs=9000 count=1 2>/dev/null
printf '\x00\x01\x02binary\x00' > build/obj/out.o
printf 'build log line\nanother\n' > app.log
printf 'hidden secret config\n' > .hidden
printf 'keep me\n' > big/keep.txt
printf 'backup content that is ignored only if big/.gitignore is read\n' > big/old.bak
printf 'vendored code\n' > vendor/thing.c
ln -s src/main.rs link_to_main
ln -s src link_to_src
# differing mtimes (all well in the past)
touch -h -t 202001010101 link_to_main link_to_src
touch -t 202101010101 src/main.rs
touch -t 202201010101 src/lib.py
touch -t 202301010101 src/Main.hs
touch -t 202302010101 src/nested/deep/util.rs
touch -t 202303010101 docs/readme.md
touch -t 202304010101 docs/notes.txt
touch -t 202305010101 data/blob.bin
touch -t 202306010101 build/obj/out.o app.log .hidden
touch -t 202307010101 big/keep.txt big/old.bak big/.gitignore src/scratch.tmp vendor/thing.c
touch -t 202308010101 .gitignore src/.gitignore
touch -t 202309010101 src/nested/deep src/nested docs data build/obj build empty big vendor src
touch -t 202310010101 .
