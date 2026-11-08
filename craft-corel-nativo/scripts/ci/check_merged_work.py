"""Fail a pull request that undoes work already merged.

Usage:
  python3 scripts/ci/check_merged_work.py --base <rev> --head <rev>
      [--body-file FILE | --event FILE] [--repo owner/name]
      [--days 60] [--min-lines 15] [--now EPOCH]

Two checks, both on the change from the merge base of <base> and <head>
to <head> (what the pull request itself changes):

1. Merged work. Every line the change deletes or replaces is blamed on
   the merge base. Lines whose text the change adds back anywhere (a move
   or a reflow) and blank lines do not count, nor do lines of the first
   commit of the history. The rest are grouped by the pull request that
   wrote them (by commit when the pull request is not known). The check
   fails when the change deletes `--min-lines` lines or more of one file
   that one other pull request merged within the last `--days` days,
   unless the pull request description lists that pull request, its
   issue (the "(#N)" of the commit subject) or the commit under a
   "Replaces:" line.

2. Tests. The names of the `#[test]` functions (with their crate and
   module path) in the Rust files the change touches, before and after.
   The check fails when one disappears, unless the description lists it
   under a "Removed tests:" line. A test that only moved (the same name
   comes back under another path) is reported, not failed.

The description lists go on the same line or on the list lines below:

    Replaces: #1783, 2e009af98
    Removed tests:
    - the_old_name_of_a_test

The days are counted back from the head commit's date (`--now` gives
another reference). With `--repo` and the `gh` command, the pull request
of each commit comes from GitHub; without them commits are grouped on
their own. `--event` reads the description and, when --base/--head are
not given, the base and head of a GitHub `pull_request` event payload.

Standard library and git only. Exit code 0: nothing lost; 1: merged work
or tests would be lost; 2: the check could not run.
"""

import argparse
import collections
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import time

DAYS = 60
MIN_LINES = 15
# Files whose deleted lines say nothing about lost work.
IGNORED_FILES = re.compile(r"(^|/)Cargo\.lock$|\.lock$")


class GitError(Exception):
    pass


def git(*args, input=None):
    run = subprocess.run(
        ["git", "-c", "core.quotePath=false", *args],
        input=input,
        capture_output=True,
        check=False,
    )
    if run.returncode != 0:
        raise GitError(f"git {' '.join(args)}: {run.stderr.decode(errors='replace').strip()}")
    return run.stdout.decode("utf-8", errors="replace")


# ---------------------------------------------------------------- the diff


def unquote_path(p):
    """A path as git prints it in a diff header, without a/ or b/."""
    if p.startswith('"') and p.endswith('"'):
        p = p[1:-1].encode("latin-1", errors="replace").decode("unicode_escape")
        p = p.encode("latin-1", errors="replace").decode("utf-8", errors="replace")
    return p


def parse_diff(text):
    """Deleted lines per base path ({path: [(line_no, text)]}) and the
    stripped text of every added line."""
    deleted = collections.defaultdict(list)
    added = set()
    path = None
    old_no = 0
    for line in text.split("\n"):
        if line.startswith("diff --git "):
            path = None
        elif line.startswith("--- "):
            p = line[4:]
            path = None if p == "/dev/null" else unquote_path(p)[2:]
        elif line.startswith("+++ "):
            pass
        elif line.startswith("@@ "):
            m = re.match(r"^@@ -(\d+)(?:,(\d+))? ", line)
            old_no = int(m.group(1)) if m else 0
        elif line.startswith("-") and path is not None:
            deleted[path].append((old_no, line[1:]))
            old_no += 1
        elif line.startswith("+"):
            added.add(line[1:].strip())
    return deleted, added


# ---------------------------------------------------------------- blame


def blame(rev, path, line_nos, since):
    """{line_no: commit} for the given lines of path at rev; lines of a
    boundary commit (the root, or older than `since`) are left out."""
    ranges = []
    for n in sorted(line_nos):
        if ranges and n == ranges[-1][1] + 1:
            ranges[-1][1] = n
        else:
            ranges.append([n, n])
    args = ["blame", "--porcelain", f"--since={since}"]
    for a, b in ranges:
        args += ["-L", f"{a},{b}"]
    out = git(*args, rev, "--", path)
    result = {}
    commit = None
    final = 0
    boundary = set()
    times = {}
    for line in out.split("\n"):
        if line.startswith("\t"):
            continue
        m = re.match(r"^([0-9a-f]{40}) \d+ (\d+)", line)
        if m:
            commit, final = m.group(1), int(m.group(2))
            result[final] = commit
        elif line == "boundary":
            boundary.add(commit)
        elif line.startswith("committer-time "):
            times[commit] = int(line.split()[1])
    return {n: c for n, c in result.items() if c not in boundary}, times


# ---------------------------------------------------------------- pull requests


def pr_of(repo, sha):
    """The merged pull request a commit came in with, from GitHub, or None."""
    try:
        run = subprocess.run(
            ["gh", "api", f"repos/{repo}/commits/{sha}/pulls"],
            capture_output=True,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if run.returncode != 0:
        return None
    try:
        pulls = json.loads(run.stdout)
    except ValueError:
        return None
    merged = [p for p in pulls if p.get("merged_at")]
    for p in merged or pulls:
        if isinstance(p.get("number"), int):
            return p["number"]
    return None


# ---------------------------------------------------------------- description


def listed(body, label):
    """The text listed under a "Label:" line of the description: the rest
    of that line and the list lines right below it."""
    lines = (body or "").replace("\r\n", "\n").split("\n")
    head = re.compile(r"^[\s>#*_`-]*" + re.escape(label) + r"[*_`]*\s*:[*_`]*(.*)$", re.IGNORECASE)
    item = re.compile(r"^\s*([-*+]|\d+[.)])\s+|^\s{2,}\S")
    out = []
    i = 0
    while i < len(lines):
        m = head.match(lines[i])
        i += 1
        if not m:
            continue
        out.append(m.group(1))
        while i < len(lines):
            if not lines[i].strip():
                i += 1
                continue
            if item.match(lines[i]) and not head.match(lines[i]):
                out.append(lines[i])
                i += 1
                continue
            break
    return "\n".join(out)


def replaced_refs(body):
    text = listed(body, "Replaces")
    numbers = {int(n) for n in re.findall(r"(?:#|/pull/|/issues/)(\d+)", text)}
    hashes = {h.lower() for h in re.findall(r"\b[0-9a-fA-F]{7,40}\b", text)}
    return numbers, hashes


def removed_test_names(body):
    text = listed(body, "Removed tests")
    return set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*", text))


# ---------------------------------------------------------------- tests


TOKEN = re.compile(
    r"""(?P<lc>//[^\n]*)
      | (?P<bc>/\*)
      | (?P<raw>(?<![A-Za-z0-9_])b?r(?P<hashes>\#*)")
      | (?P<str>"(?:[^"\\]|\\.)*")
      | (?P<chr>'(?:[^'\\\n]|\\[^\n]{1,10}?)')
      | (?P<attr>\#\s*\[\s*test\s*\])
      | (?P<fn>\bfn\s+(?P<fname>[A-Za-z_][A-Za-z0-9_]*))
      | (?P<mod>\bmod\s+(?P<mname>[A-Za-z_][A-Za-z0-9_]*)\s*\{)
      | (?P<open>\{)
      | (?P<close>\})""",
    re.DOTALL | re.VERBOSE,
)


COMMENT_EDGE = re.compile(r"/\*|\*/")


def rust_tests(src):
    """(module path inside the file, name) of each #[test] function."""
    tests = []
    mods = []  # (name, depth inside the module's braces)
    depth = 0
    pending = False
    pos = 0
    while True:
        m = TOKEN.search(src, pos)
        if not m:
            break
        pos = m.end()
        if m.group("bc"):
            level = 1
            while level and pos < len(src):
                nxt = COMMENT_EDGE.search(src, pos)
                if not nxt:
                    pos = len(src)
                    break
                level += 1 if nxt.group() == "/*" else -1
                pos = nxt.end()
        elif m.group("raw"):
            end = src.find('"' + m.group("hashes"), pos)
            pos = len(src) if end < 0 else end + 1 + len(m.group("hashes"))
        elif m.group("attr"):
            pending = True
        elif m.group("fn"):
            if pending:
                tests.append(("::".join(n for n, _ in mods), m.group("fname")))
            pending = False
        elif m.group("mod"):
            depth += 1
            mods.append((m.group("mname"), depth))
        elif m.group("open"):
            depth += 1
        elif m.group("close"):
            if mods and mods[-1][1] == depth:
                mods.pop()
            depth -= 1
    return tests


def crate_names(rev):
    """{crate directory: package name} at rev."""
    files = [
        f
        for f in git("ls-tree", "-r", "--name-only", rev).split("\n")
        if f == "Cargo.toml" or f.endswith("/Cargo.toml")
    ]
    names = {}
    for f, text in zip(files, cat_files(rev, files)):
        m = re.search(r"^\[package\][^\[]*?^\s*name\s*=\s*\"([^\"]+)\"", text or "", re.MULTILINE | re.DOTALL)
        if m:
            names[os.path.dirname(f)] = m.group(1).replace("-", "_")
    return names


def cat_files(rev, paths):
    """The contents of paths at rev (None for a missing one)."""
    if not paths:
        return []
    run = subprocess.run(
        ["git", "cat-file", "--batch"],
        input="".join(f"{rev}:{p}\n" for p in paths).encode(),
        capture_output=True,
        check=False,
    )
    if run.returncode != 0:
        raise GitError("git cat-file --batch: " + run.stderr.decode(errors="replace"))
    out = run.stdout
    texts = []
    pos = 0
    for _ in paths:
        nl = out.index(b"\n", pos)
        header = out[pos:nl].split()
        pos = nl + 1
        if len(header) < 3 or header[1] == b"missing":
            texts.append(None)
            continue
        size = int(header[2])
        texts.append(out[pos : pos + size].decode("utf-8", errors="replace"))
        pos += size + 1
    return texts


def module_of(path, crates):
    """crate::module path of a Rust file."""
    d = os.path.dirname(path)
    while True:
        if d in crates:
            break
        if not d:
            break
        d = os.path.dirname(d)
    crate = crates.get(d, d or "root")
    rel = path[len(d) + 1 :] if d else path
    rel = rel.removeprefix("src/")
    parts = rel[:-3].split("/") if rel.endswith(".rs") else rel.split("/")
    if parts and parts[-1] in ("lib", "main", "mod"):
        parts = parts[:-1]
    return "::".join([crate, *parts])


def tests_at(rev, paths):
    crates = crate_names(rev)
    found = collections.Counter()
    for path, src in zip(paths, cat_files(rev, paths)):
        if src is None or "#" not in src:
            continue
        module = module_of(path, crates)
        for inner, name in rust_tests(src):
            found["::".join(p for p in (module, inner, name) if p)] += 1
    return found


# ---------------------------------------------------------------- the checks


def check(base, head, body, repo=None, days=DAYS, min_lines=MIN_LINES, now=None, out=sys.stdout):
    def say(s=""):
        print(s, file=out)

    base_rev = git("merge-base", base, head).strip()
    head_rev = git("rev-parse", head).strip()
    if now is None:
        now = int(git("log", "-1", "--format=%ct", head_rev).strip())
    cutoff = now - days * 86400
    say(f"Change {base_rev[:9]}..{head_rev[:9]} (from the merge base with the base branch)")

    # 1. Merged work.
    diff = git("diff", "-U0", "-M", "--no-color", "--no-ext-diff", base_rev, head_rev)
    deleted, added = parse_diff(diff)
    jobs = {}
    for path, lines in deleted.items():
        if IGNORED_FILES.search(path):
            continue
        keep = [(n, t) for n, t in lines if t.strip() and t.strip() not in added]
        if keep:
            jobs[path] = keep
    by_file = collections.defaultdict(collections.Counter)  # path -> commit -> lines
    times = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        futures = {
            pool.submit(blame, base_rev, path, [n for n, _ in lines], f"{cutoff} +0000"): path
            for path, lines in jobs.items()
        }
        for fut in concurrent.futures.as_completed(futures):
            path = futures[fut]
            owners, t = fut.result()
            times.update(t)
            for n, _ in jobs[path]:
                c = owners.get(n)
                if c and times.get(c, 0) >= cutoff:
                    by_file[path][c] += 1

    commits = sorted({c for counts in by_file.values() for c in counts})
    info = {}
    for c in commits:
        ct, subject = git("log", "-1", "--format=%ct%x00%s", c).rstrip("\n").split("\x00", 1)
        info[c] = (int(ct), subject)
    prs = {}
    if repo and commits:
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            prs = dict(zip(commits, pool.map(lambda c: pr_of(repo, c), commits)))

    def group(c):
        return f"#{prs[c]}" if prs.get(c) else c[:9]

    per_group = collections.defaultdict(collections.Counter)  # (path, group) -> commit -> n
    for path, counts in by_file.items():
        for c, n in counts.items():
            per_group[(path, group(c))][c] += n

    numbers, hashes = replaced_refs(body)

    def replaced(c):
        if any(c.startswith(h) for h in hashes):
            return True
        if prs.get(c) in numbers:
            return True
        issues = {int(n) for n in re.findall(r"#(\d+)", info[c][1])}
        return bool(issues & numbers)

    rows = []  # (lines, path, group, commits, all listed under Replaces:)
    for (path, g), counts in per_group.items():
        n = sum(counts.values())
        if n >= min_lines:
            rows.append((n, path, g, sorted(counts, key=lambda c: -counts[c]), all(map(replaced, counts))))
    rows.sort(key=lambda r: (-r[0], r[1]))
    lost = [r for r in rows if not r[4]]
    total = sum(sum(c.values()) for c in by_file.values())
    say(
        f"Merged work: {total} deleted lines were written in the last {days} days by "
        f"{len(commits)} other commit(s); the rule is {min_lines}+ lines of one file from one pull request."
    )
    if rows:
        say()
        say(f"{'lines':>5}  {'from':<10}  {'merged':<10}  {'status':<8}  file")
        for n, path, g, cs, ok in rows:
            date = time.strftime("%Y-%m-%d", time.gmtime(max(info[c][0] for c in cs)))
            say(f"{n:>5}  {g:<10}  {date:<10}  {'replaced' if ok else 'LOST':<8}  {path}")
        say()
        say("Commits that wrote these lines:")
        shown = []
        for _, _, g, cs, _ in rows:
            shown += [c for c in cs if c not in shown]
        for c in shown:
            pr = f"#{prs[c]}" if prs.get(c) else "(pull request not known)"
            say(f"  {c[:9]}  {pr:<6}  {info[c][1][:100]}")
    if lost:
        say()
        say("FAIL: this change deletes merged work of the pull requests or commits above.")
        say('If that is on purpose, list each one in the description under "Replaces:"')
        say("(for example `Replaces: #1783, 2e009af98`); if it is not, rebase on a fresh")
        say("main and edit only the lines the change needs.")
    else:
        say(
            "OK: no merged work is lost."
            if not rows
            else "OK: every deletion above is listed under Replaces:."
        )

    # 2. Tests.
    status = git("diff", "--name-status", "-M", "-z", base_rev, head_rev).split("\0")
    old_paths, new_paths = [], []
    i = 0
    while i < len(status) and status[i]:
        st = status[i]
        if st[0] in "RC":
            old, new = status[i + 1], status[i + 2]
            i += 3
        else:
            old = new = status[i + 1]
            i += 2
        if st[0] in "MRD" and old.endswith(".rs"):
            old_paths.append(old)
        if st[0] in "MRCA" and new.endswith(".rs"):
            new_paths.append(new)
    before = tests_at(base_rev, old_paths)
    after = tests_at(head_rev, new_paths)
    gone = before - after
    new_names = collections.Counter(k.rsplit("::", 1)[-1] for k in (after - before).elements())
    allowed = removed_test_names(body)
    moved, removed, listed_ok = [], [], []
    for key in sorted(gone.elements()):
        name = key.rsplit("::", 1)[-1]
        if new_names[name] > 0:
            new_names[name] -= 1
            moved.append(key)
        elif any(key == a or key.endswith("::" + a) for a in allowed):
            listed_ok.append(key)
        else:
            removed.append(key)
    say()
    say(
        f"Tests: {sum(before.values())} #[test] functions before and {sum(after.values())} after in the {len(set(old_paths) | set(new_paths))} Rust files changed."
    )
    for key in moved:
        say(f"  moved     {key}")
    for key in listed_ok:
        say(f"  removed   {key} (listed under Removed tests:)")
    for key in removed:
        say(f"  REMOVED   {key}")
    if removed:
        say()
        say('FAIL: tests disappear. Keep them, or list each one under "Removed tests:"')
        say("in the description with the reason.")
    else:
        say("OK: no test disappears without being listed.")

    return 1 if lost or removed else 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--base")
    ap.add_argument("--head")
    ap.add_argument("--body-file", help="the pull request description")
    ap.add_argument("--event", help="a GitHub pull_request event payload (JSON)")
    ap.add_argument("--repo", help="owner/name, to look up pull requests with gh")
    ap.add_argument("--days", type=int, default=DAYS)
    ap.add_argument("--min-lines", type=int, default=MIN_LINES)
    ap.add_argument("--now", type=int, help="reference time (Unix seconds); default: the head commit's")
    a = ap.parse_args(argv)
    body = ""
    base, head = a.base, a.head
    if a.event:
        with open(a.event, encoding="utf-8") as f:
            pr = json.load(f).get("pull_request") or {}
        body = pr.get("body") or ""
        base = base or (pr.get("base") or {}).get("sha")
        head = head or (pr.get("head") or {}).get("sha")
    if a.body_file:
        with open(a.body_file, encoding="utf-8") as f:
            body = f.read()
    if not base or not head:
        ap.error("--base and --head (or --event) are needed")
    try:
        return check(base, head, body, a.repo, a.days, a.min_lines, a.now)
    except GitError as e:
        print(f"error: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
