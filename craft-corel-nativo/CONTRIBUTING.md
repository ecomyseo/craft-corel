# Contributing to TraceDraw

Thank you for helping. Read `AGENTS.md` first: it holds the engineering
rules (commands, no panics on input, layering, tests, units, language).
This file covers what every contribution must also respect.

## Reporting a problem

Use one of the forms under **New issue**, so the report has what it takes
to find the problem:

- **One problem per issue.** A report with several problems is hard to
  follow and cannot be closed until all of them are done; open one for
  each.
- **The TraceDraw version** (Help > About), and check it still happens
  in the latest release.
- **What to repeat it with:** the steps, and a drawing that shows it
  (zipped, as GitHub takes no `.tdraw` or `.cdr` files), a screenshot or
  a short video. For a `.cdr` that opens wrong the file is needed: a
  small drawing made just for the report is best. Never attach
  confidential work.
- **Bug or feature:** something that exists and works wrong is a bug;
  something TraceDraw does not do yet is a feature request.

Reports without what it takes to repeat the problem are marked
`needs info` until the missing details come.

## Clean contributions

- Send only work you wrote yourself, or work under a license compatible
  with MIT OR Apache-2.0 that you have the right to submit, with its
  origin stated in the pull request.
- Never send code, data or assets that were decompiled, disassembled or
  dumped from any program, or copied from proprietary software, its
  documentation or its installation files. This includes material
  produced by AI tools from such sources.
- Never port code from other implementations of a file format. Write it
  from public format notes and from files you own, and record what you
  learned, its source and the date in `CLEANROOM.md`.
- Bundle only original or openly licensed assets (icons, fonts, images,
  palettes, templates, test files) and list each one in
  `ATTRIBUTION.md` with its author, source and license.
- TraceDraw stands on its own: do not name other vendors or their
  products in code, comments, docs, commit messages or file names, and do
  not describe a feature as modelled on another program. Give features
  descriptive names of their own. The one exception is the trademark
  notice (README, `PRIVACY.md`, About box, web page, store listings),
  which names the owner of the `.cdr` trademark to say that TraceDraw is
  independent of it.

## Sign-off (DCO)

Every commit must carry a sign-off line:

```text
Signed-off-by: Your Name <you@example.com>
```

`git commit -s` adds it. By signing off you certify the Developer
Certificate of Origin 1.1 (https://developercertificate.org/): that you
wrote the contribution or otherwise have the right to submit it under
the project's licenses, and that you understand the contribution and
your sign-off are public and kept with the project.

Pull requests with unsigned commits, or with material whose origin is
unclear, are not merged.

Everyone commits and signs off under their own name, maintainers
included. The one exception is the project's own account (the
organization's owner), whose commits are authored and signed off as
`TraceDraw Maintainers <contact@tracedraw.org>`; that address reaches the
people who answer for them.

## Workflow

- Run `cargo fmt --all`, `cargo clippy --workspace --tests` and
  `cargo test --workspace` before pushing; the build must stay free of
  warnings.
- Every change comes with tests; format code gets round-trip and
  malformed-input tests.
- User-visible strings go through the translation files (`docs/i18n.md`).
- A new tool or effect gets a page in `docs/behavior/` and a row in
  `docs/features.md`.

## Pull requests

- Start from a freshly fetched `main` and rebase on it again before
  pushing when the work took long. Change only the lines your change
  needs; never write a whole file back from an older copy.
- A pull request never undoes merged work by accident. The `merged-work`
  check (`scripts/ci/check_merged_work.py`) blames every line the pull
  request deletes or replaces on its base. It fails when the pull request
  deletes 15 or more lines of one file that another pull request merged
  in the last 60 days; lines that only move (the same text added back
  elsewhere in the diff) do not count. When replacing that work is the
  point, say so in the description:

  ```
  Replaces: #1783, 2e009af98 (why)
  ```

  with the pull request, its issue or the commit, one per item, on the
  line or as a list below it.
- Tests are never deleted silently. The same check fails when a `#[test]`
  function disappears (a test that only moves to another module is
  fine), unless the description lists it with the reason:

  ```
  Removed tests:
  - tracedraw_io::cmyk_bitmap::tests::a_cmyk_jpeg_reads_back_with_its_inks (why)
  ```
- Run it before pushing:
  `python3 scripts/ci/check_merged_work.py --base origin/main --head HEAD
  --body-file description.md` (add `--repo tracedraw/tracedraw` to group
  lines by pull request through `gh`).
- Branches named `claude/<topic>` need no pull request opened by hand:
  the `claude-pr` workflow opens one for every push to such a branch and
  turns on auto-merge, so it lands as soon as the required checks pass.
  The pull request takes its title from the branch's newest commit and
  its description from the commit messages, so write them accordingly.

## Review and credit

- Every pull request from outside the project is read in full for
  security before it is merged: network access, running programs, file
  and secret access, new dependencies, build scripts and CI changes,
  obfuscated or binary content. Keep pull requests small and focused so
  this stays quick.
- Commit with your own name and an email that belongs to your GitHub
  account (if a coding assistant makes the commits, set
  `git config user.name` and `user.email` to yours first); GitHub's
  contributor list goes by the commit's author email. A pull request that
  needs a conflict resolved is merged keeping your commits as you wrote
  them.
- Once merged, you are listed in [`CONTRIBUTORS.md`](CONTRIBUTORS.md)
  with what you contributed.

## Roles

TraceDraw lives in the [`tracedraw`](https://github.com/tracedraw)
GitHub organization (team pages are visible to its members only). Everyone starts as a contributor; reviewers and
maintainers are invited from the people who contribute code. How
reviewers and maintainers work is in [`MAINTAINING.md`](MAINTAINING.md).

| Role | On GitHub | Can | Cannot |
|---|---|---|---|
| Contributor | anyone, from a fork | open issues and pull requests, comment, send sample files | label, close or merge |
| Reviewer | the `reviewers` team (Triage) | label, assign, close and reopen issues and pull requests; review and approve pull requests | push, merge, release, change settings |
| Maintainer | the `maintainers` team (Maintain) | everything a reviewer can, and merge approved pull requests, manage labels and milestones, prepare releases | change settings or branch protection, see secrets, sign or publish builds |
| Owner | the organization's owners | everything, including settings, secrets, signing and the store listings | |

- **Becoming a reviewer:** at least two merged pull requests of your own,
  signed off by you, that follow `AGENTS.md` without needing to be
  redone.
- **Becoming a maintainer:** some weeks as a reviewer with careful
  reviews.
- **Every member of the organization uses two-factor authentication**;
  GitHub removes members without it.
- `main` is protected for everyone, owners included: every change comes
  through a pull request with one approval from someone other than its
  author and a green build, so no single person, and no single mistake,
  reaches `main` alone. A bad merge is reverted, never rewritten.
- Roles are for people who are active. A member who has not been active
  for six months is thanked and moved back to contributor, and can come
  back at any time.

## License

Contributions are licensed under MIT OR Apache-2.0, like the rest of the
project.
