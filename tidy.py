"""Formats, checks that the targets mirror each other, and builds with warnings as errors.

Each target's files under Targets and IR/Extensions/Targets pair with the others' through Targets/Vocabulary.txt,
whose first row names the targets and each row a word per target, the first target's spelling standing for all. Paired
files compare token by token, so layout is free; every block of differing lines sits below a mark of its own target,
`(* Target: reason *)` on a line of its own, and every mark sits above such a block."""

import difflib
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
TREES = [ROOT / "Targets", ROOT / "IR" / "Extensions" / "Targets"]
VOCABULARY = ROOT / "Targets" / "Vocabulary.txt"
MIRRORED = ("*.ml", "*.mli", "dune")


def vocabulary():
    rows = [line.split() for line in VOCABULARY.read_text().splitlines() if line.strip()]
    return rows[0], rows


def translator(rows, column):
    words = {row[column]: row[0] for row in rows if row[column] != "-"}
    ordered = sorted(words, key=len, reverse=True)
    upper = "|".join(re.escape(word) for word in ordered if word[0].isupper())
    lower = "|".join(re.escape(word) for word in ordered if not word[0].isupper())
    pattern = re.compile(rf"(?:(?:{upper})|(?<![A-Za-z0-9])(?:{lower}))(?![a-z0-9])")
    return lambda text: pattern.sub(lambda found: words[found.group(0)], text)


def mark(target):
    return re.compile(rf"^\s*(\| )?(?:\(\* {target}: .+ \*\)|; {target}: .+)$")


def files(target, translate):
    found = {}
    for tree in TREES:
        for pattern in MIRRORED:
            for path in (tree / target).rglob(pattern):
                found[translate(str(path.relative_to(ROOT)))] = path
    return found


def carried(lines, index, target):
    """The bar a mark in a variant declaration holds for the constructor below it."""
    found = mark(target).match(lines[index - 1]) if index > 0 else None
    return "| " if found and found.group(1) else ""


def read(target, path, translate):
    lines = path.read_text().splitlines()
    marks = {index for index, line in enumerate(lines) if mark(target).match(line)}
    kept = [index for index in range(len(lines)) if index not in marks]
    tokens = [
        (token, ordinal)
        for ordinal, index in enumerate(kept)
        for token in re.findall(r"\w+|[^\w\s]", carried(lines, index, target) + translate(lines[index]))
    ]
    tokens = [
        (token, ordinal)
        for position, (token, ordinal) in enumerate(tokens)
        if not (token == "|" and position > 0 and tokens[position - 1][0] in ("function", "with", "="))
    ]
    return [token for token, _ in tokens], [ordinal for _, ordinal in tokens], kept, marks, lines


def above(lines, index, target):
    """The nearest line above the index that is neither blank nor in a documentation comment, which the formatter keeps
    between a mark and the item it documents; a block starting within a comment starts at the comment."""
    start = index
    while start > 0 and "*)" not in lines[start - 1] and "(*" not in lines[start]:
        start -= 1
    if "(*" in lines[start]:
        index = start
    index -= 1
    while index >= 0:
        line = lines[index].strip()
        if not line:
            index -= 1
        elif line.endswith("*)") and not mark(target).match(lines[index]):
            start = index
            while start > 0 and "(*" not in lines[start]:
                start -= 1
            if not lines[start].lstrip().startswith("(**"):
                return index
            index = start - 1
        else:
            return index
    return index


def blocks(runs):
    """Differing runs of lines, joined where at most one line between them matches."""
    joined = []
    for first, last in sorted(runs):
        if joined and first <= joined[-1][1] + 2:
            joined[-1][1] = max(joined[-1][1], last)
        else:
            joined.append([first, last])
    return joined


def mirroring():
    targets, rows = vocabulary()
    translators = [translator(rows, column) for column in range(len(targets))]
    trees = [files(target, translate) for target, translate in zip(targets, translators)]
    errors = []
    for key in sorted(set().union(*trees)):
        sides = [
            read(target, tree[key], translate) if key in tree else ([], [], [], set(), [])
            for target, translate, tree in zip(targets, translators, trees)
        ]
        runs = [[] for _ in targets]
        for left, right in ((left, right) for left in range(len(targets)) for right in range(left + 1, len(targets))):
            matcher = difflib.SequenceMatcher(None, sides[left][0], sides[right][0], autojunk=False)
            for tag, low, high, other_low, other_high in matcher.get_opcodes():
                if tag != "equal":
                    for side, start, end in ((left, low, high), (right, other_low, other_high)):
                        if start < end:
                            runs[side].append((sides[side][1][start], sides[side][1][end - 1]))
        for column, (target, tree) in enumerate(zip(targets, trees)):
            if key not in tree:
                continue
            path = tree[key].relative_to(ROOT)
            _, _, kept, marks, lines = sides[column]
            required = {above(lines, kept[first], target): kept[first] for first, _ in blocks(runs[column])}
            for line in sorted(set(required) - marks):
                errors.append(f"{path}:{required[line] + 1}: differs from its counterparts without a {target} mark above")
            for line in sorted(marks - set(required)):
                errors.append(f"{path}:{line + 1}: marks no difference")
    for error in errors:
        print(error)
    return not errors


subprocess.run(["nix-shell", "--run", "dune fmt --root ."], cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
mirrored = mirroring()
built = subprocess.run(["nix-shell", "--run", "dune build --root ."], cwd=ROOT).returncode == 0
sys.exit(0 if mirrored and built else 1)
