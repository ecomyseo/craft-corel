#!/usr/bin/env python3
"""Build the browser build's font catalog: every family of the
google/fonts collection at the commit web/fonts.txt pins, by the paths of
its files in that repository.

    python3 web/font_catalog.py web/fonts.txt site/fonts [--google-fonts DIR]

Writes `catalog.json` into the output folder:

    {"base": "https://raw.githubusercontent.com/google/fonts/<commit>/",
     "families": {"Abel": ["ofl/abel/Abel-Regular.ttf"], ...}}

The page offers these families with the ones of its font folder
(`fonts.json`, built by font_folder.py, which wins for a family both
have) and fetches a family's files from that address the first time a
drawing's text needs it, so the catalog costs nothing until used.

The families and their files come from each family's METADATA.pb: a
sparse, blobless fetch of just those files at the pinned commit (a few
megabytes), or the checkout `--google-fonts DIR` names. Families whose
files are not TrueType or OpenType, or whose paths are not plain, are
left out.
"""

import json
import os
import re
import subprocess
import sys
import tempfile

BASE = "https://raw.githubusercontent.com/google/fonts/%s/"
LICENCE_DIRS = ("ofl", "apache", "ufl")
PLAIN_PATH = re.compile(r"^(ofl|apache|ufl)/[a-z0-9_]+/[A-Za-z0-9._\-\[\],]+\.(ttf|otf)$")


def read_commit(list_path):
    with open(list_path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line.startswith("commit "):
                return line.split()[1]
    return None


def fetch_metadata(commit, into):
    """A sparse, blobless fetch of every METADATA.pb at `commit`."""
    run = lambda *args: subprocess.run(["git", "-C", into] + list(args), check=True, capture_output=True)
    subprocess.run(["git", "init", "-q", into], check=True, capture_output=True)
    run("remote", "add", "origin", "https://github.com/google/fonts")
    run("config", "core.sparseCheckout", "true")
    run("sparse-checkout", "set", "--no-cone", *["/%s/*/METADATA.pb" % d for d in LICENCE_DIRS])
    run("fetch", "-q", "--depth", "1", "--filter=blob:none", "origin", commit)
    run("checkout", "-q", "FETCH_HEAD")


def parse_metadata(text):
    """The family name and the font file names of a METADATA.pb (protocol
    buffer text format): the top-level `name`, and `filename` in each
    `fonts` block."""
    family = None
    files = []
    depth = 0
    block = None
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.endswith("{"):
            if depth == 0:
                block = line[:-1].strip()
            depth += 1
            continue
        if line == "}":
            depth = max(0, depth - 1)
            if depth == 0:
                block = None
            continue
        m = re.match(r'^(\w+):\s*"(.*)"\s*$', line)
        if not m:
            continue
        key, value = m.group(1), m.group(2)
        if depth == 0 and key == "name" and family is None:
            family = value
        elif depth == 1 and block == "fonts" and key == "filename" and value not in files:
            files.append(value)
    return family, files


def catalog(root, skip):
    families = {}
    for d in LICENCE_DIRS:
        top = os.path.join(root, d)
        if not os.path.isdir(top):
            continue
        for folder in sorted(os.listdir(top)):
            meta = os.path.join(top, folder, "METADATA.pb")
            if not os.path.isfile(meta):
                continue
            with open(meta, encoding="utf-8", errors="replace") as f:
                family, files = parse_metadata(f.read())
            if not family or family in skip or family in families:
                continue
            paths = ["%s/%s/%s" % (d, folder, name) for name in files]
            paths = [p for p in paths if PLAIN_PATH.match(p)]
            if paths:
                families[family] = paths
    return families


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    list_path, out = argv[1], argv[2]
    commit = read_commit(list_path)
    if not commit:
        print("error: no commit line in %s" % list_path)
        return 1
    os.makedirs(out, exist_ok=True)
    skip = set()
    folder_list = os.path.join(out, "fonts.json")
    if os.path.isfile(folder_list):
        with open(folder_list, encoding="utf-8") as f:
            skip = set(json.load(f).keys())
    if "--google-fonts" in argv:
        families = catalog(argv[argv.index("--google-fonts") + 1], skip)
    else:
        with tempfile.TemporaryDirectory() as tmp:
            fetch_metadata(commit, tmp)
            families = catalog(tmp, skip)
    doc = {"base": BASE % commit, "families": dict(sorted(families.items()))}
    with open(os.path.join(out, "catalog.json"), "w", encoding="utf-8") as f:
        json.dump(doc, f, ensure_ascii=False, separators=(",", ":"))
    files = sum(len(v) for v in families.values())
    print("font catalog: %d families, %d files (%d left to the folder)" % (len(families), files, len(skip)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
