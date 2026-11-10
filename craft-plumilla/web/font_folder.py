#!/usr/bin/env python3
"""Build the browser build's font folder from web/fonts.txt.

    python3 web/font_folder.py web/fonts.txt site/fonts [--google-fonts DIR]

Copies (or downloads) the font files the list names into the output
folder, the licence of each source into `licences/`, and writes
`fonts.json`: an object mapping each family, as the font files name it,
to its file names. The page fetches a family's files the first time a
drawing's text needs it. Files that cannot be found are reported and
skipped; the folder is built from whatever is found.

`--google-fonts DIR` copies the google/fonts files from a local checkout
instead of downloading them at the pinned commit.
"""

import json
import os
import shutil
import struct
import sys
import urllib.parse
import urllib.request

FONT_ROOTS = [
    "/usr/share/fonts",
    "/usr/share/texmf/fonts",
    "/usr/share/texlive/texmf-dist/fonts",
]
LICENCE_FILES = ["OFL.txt", "LICENSE.txt", "UFL.txt", "LICENCE.txt"]


def family_name(data):
    """The family a font file names (typographic family first, as the text
    engine reads it), or None."""
    try:
        if data[:4] == b"ttcf":
            (offset,) = struct.unpack(">I", data[12:16])
        else:
            offset = 0
        (num_tables,) = struct.unpack(">H", data[offset + 4 : offset + 6])
        name_at = None
        for i in range(num_tables):
            rec = data[offset + 12 + 16 * i : offset + 28 + 16 * i]
            tag, _, table_offset, _ = struct.unpack(">4sIII", rec)
            if tag == b"name":
                name_at = table_offset
        if name_at is None:
            return None
        _, count, string_offset = struct.unpack(">HHH", data[name_at : name_at + 6])
        found = {}
        for i in range(count):
            rec = data[name_at + 6 + 12 * i : name_at + 18 + 12 * i]
            platform, encoding, language, name_id, length, at = struct.unpack(">HHHHHH", rec)
            if name_id not in (1, 16):
                continue
            raw = data[name_at + string_offset + at : name_at + string_offset + at + length]
            if platform == 3 and encoding in (0, 1) and language == 0x409:
                text = raw.decode("utf-16-be", "replace")
            elif platform == 1 and encoding == 0 and language == 0:
                text = raw.decode("mac_roman", "replace")
            else:
                continue
            found.setdefault(name_id, text)
        return found.get(16) or found.get(1)
    except (struct.error, IndexError):
        return None


def output_name(path):
    """The file name in the folder: a variable font's axis list
    ("Roboto[wdth,wght].ttf") becomes "-VF"."""
    base = os.path.basename(path)
    stem, ext = os.path.splitext(base)
    if "[" in stem:
        stem = stem.split("[")[0].rstrip("-_") + "-VF"
    return stem + ext


def find_installed(name):
    for root in FONT_ROOTS:
        for folder, _, files in os.walk(root):
            if name in files:
                return os.path.join(folder, name)
    return None


def google_fonts_bytes(path, commit, local):
    if local:
        full = os.path.join(local, path)
        if os.path.isfile(full):
            with open(full, "rb") as f:
                return f.read()
        return None
    url = "https://raw.githubusercontent.com/google/fonts/%s/%s" % (
        commit,
        urllib.parse.quote(path),
    )
    try:
        with urllib.request.urlopen(url, timeout=120) as r:
            return r.read()
    except OSError as e:
        print("warning: %s: %s" % (url, e))
        return None


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    list_path, out = argv[1], argv[2]
    local = None
    if "--google-fonts" in argv:
        local = argv[argv.index("--google-fonts") + 1]
    os.makedirs(os.path.join(out, "licences"), exist_ok=True)
    commit = None
    families = {}
    missing = 0
    with open(list_path, encoding="utf-8") as f:
        lines = [l.strip() for l in f]
    for line in lines:
        if not line or line.startswith("#"):
            continue
        if line.startswith("commit "):
            commit = line.split()[1]
            continue
        parts = [p.strip() for p in line.split("|")]
        if parts[0] == "licence" and len(parts) == 2 and parts[1].startswith("deb:"):
            package = parts[1][4:]
            src = "/usr/share/doc/%s/copyright" % package
            if os.path.isfile(src):
                shutil.copyfile(src, os.path.join(out, "licences", package + ".txt"))
            else:
                print("warning: no licence for %s" % package)
            continue
        if len(parts) != 3:
            print("warning: unreadable line: %s" % line)
            continue
        label, source, files = parts[0], parts[1], parts[2].split()
        got = 0
        for path in files:
            if source == "gf":
                data = google_fonts_bytes(path, commit, local)
            elif source.startswith("deb:"):
                found = find_installed(path)
                data = open(found, "rb").read() if found else None
            else:
                print("warning: unknown source %s" % source)
                data = None
            if data is None:
                print("warning: %s: %s not found" % (label, path))
                missing += 1
                continue
            name = family_name(data)
            if not name:
                print("warning: %s: %s names no family" % (label, path))
                continue
            if name != label:
                print("note: %s: %s names its family %r" % (label, path, name))
            out_name = output_name(path)
            with open(os.path.join(out, out_name), "wb") as f:
                f.write(data)
            files_of = families.setdefault(name, [])
            if out_name not in files_of:
                files_of.append(out_name)
            got += 1
        if got == 0:
            continue
        # The licence of the family's source.
        if source == "gf":
            folder = os.path.dirname(files[0])
            for lic in LICENCE_FILES:
                data = google_fonts_bytes(folder + "/" + lic, commit, local)
                if data:
                    target = os.path.join(out, "licences", os.path.basename(folder) + ".txt")
                    with open(target, "wb") as f:
                        f.write(data)
                    break
            else:
                print("warning: %s: no licence file found" % label)
        elif source.startswith("deb:"):
            package = source[4:]
            src = "/usr/share/doc/%s/copyright" % package
            if os.path.isfile(src):
                shutil.copyfile(src, os.path.join(out, "licences", package + ".txt"))
            else:
                print("warning: no licence for %s" % package)
    with open(os.path.join(out, "fonts.json"), "w", encoding="utf-8") as f:
        json.dump(families, f, ensure_ascii=False, indent=0, sort_keys=True)
    size = sum(
        os.path.getsize(os.path.join(out, n))
        for n in os.listdir(out)
        if os.path.isfile(os.path.join(out, n))
    )
    print(
        "font folder: %d families, %.1f MB, %d files not found"
        % (len(families), size / 1e6, missing)
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
