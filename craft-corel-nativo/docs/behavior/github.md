# GitHub (File > GitHub)

Drawings kept in a GitHub repository, through GitHub's REST API
(`api.github.com`), on the desktop and in the browser.

- **Open from GitHub...**: repository (`owner/name` or its github.com
  address), branch (`main` when empty), the file's path in the
  repository, and a token. The file's description comes first (its blob
  id, and its text when it is under 1 MB); a larger file is fetched again
  as raw bytes. A `.tdraw` opens as a new drawing named after the file;
  the drawing remembers the repository, branch, path and the version it
  was opened at.
- **Save** (Ctrl+S) on such a drawing, and **Save to GitHub...** for any
  drawing, open the same form with a commit message ("Update
  <file name>"). OK commits the drawing (as `.tdraw`, with its fonts)
  to that path on that branch, over the version it was opened or last
  saved at. When the file changed on GitHub meanwhile, GitHub refuses
  (409) and the dialog says so: save under another path or branch, or
  open the newer version first. A save to a path where a file already
  exists, from a drawing that did not come from it, is refused the same
  way. Save As still saves a file on the computer.
- **History...** (a drawing from GitHub): the commits of the branch that
  changed the file, newest first (50), with their short id, date, author
  and first line. **Open** opens the file as it was at that commit, as a
  new drawing named "<name> @ <short id>" that is not tied to GitHub.
- **Token**: a fine-grained personal access token with Contents read and
  write on the repository. Public repositories open without one. It is
  kept for the session; **Remember the token on this computer** keeps it
  in the preferences file (in the browser: the page's local storage) in
  plain text. The form remembers the last repository, branch and path.
- While a request is out the dialog shows a spinner; the answer opens the
  drawing or closes the dialog, or the reason shows in red: a token
  GitHub refuses (401), no access or too many requests (403), no such
  file, branch or repository (404), a newer version (409), a file
  already there (422), a path that is a folder, a file that is not a
  drawing, a drawing over 100 MB.

Not yet: a branch per session, pull requests, a visual diff between
commits, merging conflicts in the interface, Git LFS, signing in with
OAuth, keeping the token in the system's keyring.

## Checks

- `github::tests::repositories_and_paths_are_read_as_typed`
- `github::tests::requests_name_the_file_and_carry_the_token`
- `github::tests::answers_are_read`
- `github::tests::open_edit_save_and_conflict`
- `github::tests::large_files_and_earlier_versions`
- `github::tests::live_open_of_a_public_drawing` (run by hand, needs the
  network): `sample.tdraw` of this repository opens without a token and
  its history lists commits.
