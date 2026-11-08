"""Valida UTF-8 y excepciones literales documentadas sin excluir sus archivos."""
from pathlib import Path
import argparse
import hashlib
import json

EXTENSIONS = {".css", ".csv", ".html", ".ini", ".js", ".json", ".md", ".php", ".py", ".sql", ".svg", ".tpl", ".txt", ".xml", ".xlf", ".yaml", ".yml"}
MARKERS = tuple(map(chr, [195, 194, 65533, 402, 198])) + (chr(226) + chr(8364),)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--exceptions", required=True, type=Path)
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    allowed = json.loads(args.exceptions.read_text(encoding="utf-8"))
    expected = {(item["path"], item["line"], item["line_sha256"]): item["reason"] for item in allowed}
    used, failures = set(), []
    count = 0
    for path in root.rglob("*"):
        if not path.is_file() or ".git" in path.relative_to(root).parts:
            continue
        if path.suffix.lower() not in EXTENSIONS | {".rs", ".toml", ".lock"}:
            continue
        relative = path.relative_to(root).as_posix()
        try:
            text = path.read_text(encoding="utf-8", errors="strict")
        except UnicodeError:
            failures.append({"path": relative, "error": "UTF-8 inválido"})
            continue
        count += 1
        # Source tokens and translations are decoded strictly above. The
        # publication guard's marker policy applies to its supported formats.
        if path.suffix.lower() not in EXTENSIONS or "vendor" in path.relative_to(root).parts:
            continue
        for number, line in enumerate(text.splitlines(), 1):
            if any(marker in line for marker in MARKERS):
                key = (relative, number, hashlib.sha256(line.encode("utf-8")).hexdigest())
                if key in expected:
                    used.add(key)
                else:
                    failures.append({"path": relative, "line": number, "error": "Marcador sin excepción exacta"})
    for unused in expected.keys() - used:
        failures.append({"path": unused[0], "line": unused[1], "error": "Excepción obsoleta"})
    print(json.dumps({"passed": not failures, "utf8_files": count, "documented_intentional_lines": len(used), "failures": failures}, ensure_ascii=True))
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
