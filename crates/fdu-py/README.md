# fdu (Python)

Python bindings for [fdu](https://github.com/jlevy/fdu), a fast, incremental file
roll-up engine.

```python
import fdu_py

index = fdu_py.open("/path/to/tree")
print(index.complete)         # false when unreadable paths made the result partial
print(index.freshness)        # fresh, reconciling, stale, or partial
print(index.errors)           # details from the latest open/scan/refresh
print(index.total())          # {'files': ..., 'bytes': ..., 'by_extension': {...}}
print(index.children("src"))  # one call returns every child with its roll-up

mark = index.clock
result = index.refresh()      # reuses the original max_depth and returns error details
print(index.since(mark))      # what changed, or truncated=True if you fell behind
```

Every method is bulk: it returns a whole structured result in one call rather than a
cursor Python iterates.
Native work runs with the GIL released.

**Status: early scaffold**, not yet published to PyPI.

License: MIT.
