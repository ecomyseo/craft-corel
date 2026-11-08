"""Self-test of check_merged_work.py on tiny temporary git repositories.

Usage: python3 scripts/ci/test_check_merged_work.py
"""

import io
import os
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_merged_work as cmw

DAY = 86400
NOW = 1_790_000_000


def numbered(prefix, n):
    return "".join(f"{prefix} line {i}\n" for i in range(n))


TEST_FILE = """\
pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() { assert_eq!(add(1, 2), 3); }

    #[test]
    fn adds_negative() { assert_eq!(add(-1, -2), -3); }
}
"""


class Repo:
    def __init__(self, path):
        self.path = path
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.name", "Test")
        self.git("config", "user.email", "test@example.org")
        self.git("config", "commit.gpgsign", "false")

    def git(self, *args, when=NOW):
        env = dict(os.environ, GIT_AUTHOR_DATE=f"{when} +0000", GIT_COMMITTER_DATE=f"{when} +0000")
        return subprocess.run(
            ["git", *args], cwd=self.path, env=env, check=True, capture_output=True, text=True
        ).stdout.strip()

    def write(self, path, text):
        full = os.path.join(self.path, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as f:
            f.write(text)

    def remove(self, path):
        os.remove(os.path.join(self.path, path))

    def commit(self, message, when=NOW):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message, when=when)
        return self.git("rev-parse", "HEAD")


class MergedWork(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Repo(self.tmp.name)
        self.cwd = os.getcwd()
        os.chdir(self.tmp.name)
        self.repo.write("Cargo.toml", '[package]\nname = "demo"\n')
        self.repo.write("README.md", numbered("readme", 20))
        self.root = self.repo.commit("Start", when=NOW - 20 * DAY)

    def tearDown(self):
        os.chdir(self.cwd)
        self.tmp.cleanup()

    def run_check(self, base, head="HEAD", body=""):
        out = io.StringIO()
        code = cmw.check(base, head, body, now=NOW + DAY, out=out)
        return code, out.getvalue()

    def feature(self, lines=20, when=NOW - DAY):
        self.repo.write("src/feature.rs", numbered("feature", lines))
        return self.repo.commit("Feature: a new thing (#12)", when=when)

    def test_deleting_another_pull_requests_lines_fails(self):
        feat = self.feature()
        self.repo.remove("src/feature.rs")
        self.repo.commit("Something else")
        code, out = self.run_check(feat)
        self.assertEqual(code, 1, out)
        self.assertIn("LOST", out)
        self.assertIn(feat[:9], out)
        self.assertIn("src/feature.rs", out)

    def test_replacing_a_whole_file_with_an_older_copy_fails(self):
        self.repo.write("src/feature.rs", numbered("old", 30))
        self.repo.commit("Old", when=NOW - 10 * DAY)
        self.repo.write("src/feature.rs", numbered("old", 5) + numbered("new", 25))
        feat = self.repo.commit("Feature (#12)", when=NOW - DAY)
        self.repo.write("src/feature.rs", numbered("old", 30))
        self.repo.commit("Stale copy")
        code, out = self.run_check(feat)
        self.assertEqual(code, 1, out)
        self.assertIn(" 25  " + feat[:9], out)

    def test_replaces_line_lets_it_pass(self):
        feat = self.feature()
        self.repo.remove("src/feature.rs")
        self.repo.commit("Something else")
        for body in (
            f"Replaces: {feat[:9]}",
            "Fixes #3\n\n**Replaces:** #12 (the old way)",
            f"Replaces:\n\n- {feat}\n",
        ):
            code, out = self.run_check(feat, body=body)
            self.assertEqual(code, 0, body + "\n" + out)
            self.assertIn("replaced", out)
        code, out = self.run_check(feat, body="Replaces: #13\n\nSomething about #12")
        self.assertEqual(code, 1, out)

    def test_fewer_lines_than_the_rule_pass(self):
        feat = self.feature(lines=14)
        self.repo.remove("src/feature.rs")
        self.repo.commit("Something else")
        code, out = self.run_check(feat)
        self.assertEqual(code, 0, out)

    def test_old_work_passes(self):
        feat = self.feature(when=NOW - 70 * DAY + 5 * DAY)  # 65 days before the check
        self.repo.git("commit", "-q", "--allow-empty", "-m", "later")
        code, out = self.run_check(feat)
        self.assertEqual(code, 0, out)
        self.repo.remove("src/feature.rs")
        self.repo.commit("Something else")
        code, out = self.run_check(feat)
        self.assertEqual(code, 0, out)

    def test_the_first_commit_does_not_count(self):
        self.repo.remove("README.md")
        self.repo.commit("Drop the readme of the first commit")
        code, out = self.run_check(self.root)
        self.assertEqual(code, 0, out)

    def test_moved_lines_pass(self):
        feat = self.feature()
        self.repo.remove("src/feature.rs")
        self.repo.write("src/other.rs", "// moved here\n" + numbered("feature", 20))
        self.repo.commit("Move")
        code, out = self.run_check(feat)
        self.assertEqual(code, 0, out)

    def test_the_merge_base_is_the_base(self):
        # A branch made before the feature was merged does not delete it.
        before = self.repo.git("rev-parse", "HEAD")
        self.feature()
        self.repo.git("checkout", "-q", "-b", "topic", before)
        self.repo.write("src/topic.rs", "fn topic() {}\n")
        self.repo.commit("Topic")
        code, out = self.run_check("main", "topic")
        self.assertEqual(code, 0, out)

    def test_a_removed_test_fails_unless_listed(self):
        self.repo.write("src/lib.rs", TEST_FILE)
        base = self.repo.commit("Tests")
        self.repo.write(
            "src/lib.rs", TEST_FILE.replace("    #[test]\n    fn adds_negative", "    fn adds_negative")
        )
        self.repo.commit("Drop a test")
        code, out = self.run_check(base)
        self.assertEqual(code, 1, out)
        self.assertIn("REMOVED   demo::tests::adds_negative", out)
        for body in (
            "Removed tests: adds_negative",
            "Removed tests:\n- `demo::tests::adds_negative`: obsolete",
        ):
            code, out = self.run_check(base, body=body)
            self.assertEqual(code, 0, body + "\n" + out)

    def test_a_moved_test_passes(self):
        self.repo.write("src/lib.rs", TEST_FILE)
        base = self.repo.commit("Tests")
        self.repo.write("src/lib.rs", TEST_FILE.replace("mod tests", "mod checks"))
        self.repo.commit("Rename the module")
        code, out = self.run_check(base)
        self.assertEqual(code, 0, out)
        self.assertIn("moved", out)


class Parsing(unittest.TestCase):
    def test_rust_tests(self):
        src = r"""
fn not_a_test() { let s = "}{ #[test] fn fake() {}"; let c = '}'; let l: &'static str = "x"; }
/* #[test] fn in_comment() {} /* nested } */ */
// #[test] fn line_comment() {}
const R: &str = r#"mod fake { #[test] fn raw() {} }"#;
#[test]
fn top() {}
mod outer {
    mod inner {
        #[test]
        #[should_panic]
        fn deep() { if true { } }
    }
    #[test] fn after_inner() {}
}
mod declared;
#[test] fn last() {}
"""
        self.assertEqual(
            cmw.rust_tests(src),
            [("", "top"), ("outer::inner", "deep"), ("outer", "after_inner"), ("", "last")],
        )

    def test_module_of(self):
        crates = {"crates/io": "tracedraw_io", "apps/app": "app"}
        self.assertEqual(cmw.module_of("crates/io/src/lib.rs", crates), "tracedraw_io")
        self.assertEqual(cmw.module_of("crates/io/src/a/mod.rs", crates), "tracedraw_io::a")
        self.assertEqual(cmw.module_of("crates/io/src/a/b.rs", crates), "tracedraw_io::a::b")
        self.assertEqual(cmw.module_of("apps/app/tests/round.rs", crates), "app::tests::round")

    def test_listed(self):
        body = "Fixes #1\n\nReplaces: #5, abcdef1\n- #6\n  and 1234567\n\nRemoved tests:\n* a::b\n* c\nMore text #7"
        self.assertEqual(cmw.replaced_refs(body), ({5, 6}, {"abcdef1", "1234567"}))
        self.assertEqual(cmw.removed_test_names(body), {"a::b", "c"})
        self.assertEqual(cmw.replaced_refs("No list here #5 abcdef1"), (set(), set()))


if __name__ == "__main__":
    unittest.main()
