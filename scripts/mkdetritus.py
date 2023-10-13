#!/usr/bin/env python3

import os
import string
from pathlib import Path

def create_large_file(name, size=30):
    with open(name,"wb") as f:
        f.seek((2**size)-1)
        f.write(b"\0")

dirs = [
    ".cache",
    ".coverage",
    ".DS_Store",
    ".mypy_cache",
    ".pylint_cache",
    ".pytest_cache",
    ".ruff_cache",
    "__pycache__",
]

files = [
    # file
    ".bash_history",
    ".python_history",
    "pip-log.txt",
]

file_endings = [
    ".log",
    ".o",
    ".pyc",
]

bad_examples = [
    ".logending",
    ".ofine",
    ".offline",
]

for f in files:
    p = Path(f)
    p.touch()

# for i in bad_examples:
#     p = Path(i)
#     p.touch()

for e in file_endings:
    for c in string.ascii_lowercase:
        p = Path(f'{c}{e}')
        p.touch()

for d in dirs:
	os.mkdir(d)

create_large_file("hello.log")


