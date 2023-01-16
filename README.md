# quickdiff

`quickdiff` is a python library for quickly finding nested differences between two python objects.

## Usage:
```python 
from quickdiff import *

a = {1: 1, 2: 2, 3: [3], 4: 4}
b = {1: 1, 2: 4, 3: [3, 4], 5: 5, 6: 6}

report = quickdiff(a, b)

assert report == DiffReport(
    val_changes=[ValChange(path=[2], a=2, b=4)],
    type_and_val_changes=[],
    dict_items_added=[DictDiff(path=[], key=5, val=5), DictDiff(path=[], key=6, val=6)],
    dict_items_removed=[DictDiff(path=[], key=4, val=4)],
    iter_len_mismatch=[IterLenMismatch(path=[3], a_len=1, b_len=2)]
)
```

Diff objects (`ValChange`, `DictDiff`, etc) are NamedTuples for improved ergonomics and thus can be unpacked as you would any tuple:

```python

for path, a, b in report.val_changes:
    print(path, a, b)

# ([2], 2, 4)
```

## Why not DeepDiff

I wrote this because [DeepDiff](https://pypi.org/project/deepdiff/) is quite slow as it's written in pure Python and has a lot of features.

Quickdiff on the other hand is simple and written in Rust. The current implementation yields a 16x performance boost on my personal benchmarks.

## Development

Use `maturin` for development:

```bash
pip install maturin
```

Compile development version with:
```bash
maturin development
```

Run tests:
```bash
python -m unittest discover tests
```

## Roadmap

- [ ] support for sets (currently is treated as an iterator)
- [ ] parallelize for improved performance (by using `pyo3-ffi` to sidestep the Python runtime)
- [ ] attribute diff checking for python objects
- [ ] support custom `__eq__()` implementations
