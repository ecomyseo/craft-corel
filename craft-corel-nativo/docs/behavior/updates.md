# Updates

Desktop only (the browser build is always the latest; there Help >
Updates opens the releases page).

- Help > Updates asks GitHub for the project's latest release
  (`api.github.com/repos/tracedraw/tracedraw/releases/latest`) and says
  whether it is newer than this version (numbers compared as major,
  minor, patch; drafts, pre-releases and the nightly build never count).
- A newer release offers Download, Release notes, Skip this version and
  Close. Download opens the link of the release file named exactly for
  this system, its processor (x86_64, or arm64 for aarch64) and how this
  copy was installed; the system's installer does the installing:
  - Linux run as an AppImage (`APPIMAGE` set): `tracedraw-<v>-linux-<arch>.AppImage`,
    where x86_64 looks for `linux-amd64` first and then `linux-x86_64`
    (releases up to 0.19.8). From 0.19.9 on the release names the x86_64
    AppImage `linux-amd64`, which sorts before `linux-arm64`: updaters of
    0.19.8 and earlier take the first AppImage in GitHub's list, sorted
    by name, and so get the x86_64 file (#1038). The tar.gz keeps
    `linux-x86_64`;
  - Linux installed from the `.deb` (dpkg's list for `tracedraw` names
    the running program): `tracedraw_<v>_<amd64|arm64>.deb`;
  - Linux unpacked from the tar.gz (a program outside `/usr`):
    `tracedraw-<v>-linux-<arch>.tar.gz`;
  - Windows installed with setup: `tracedraw-<v>-windows-<arch>-setup.exe`;
    the portable copy (the `tracedraw-settings` folder next to the
    program): `tracedraw-<v>-windows-<arch>-portable.zip`;
  - macOS: `tracedraw-<v>-macos-<arch>.dmg`.
  When the release has no such file (an rpm or other system package
  under `/usr`, a Flatpak or Snap, a processor without a build such as
  Windows on arm64) no file is downloaded and Download opens the release
  page instead; a file for another processor or package is never offered
  (#1038: the x86_64 AppImage took the arm64 one, listed first).
- The update card (`ui/update_card.rs`): once a newer release is known
  (found by the check at start or by Help > Updates) and was not
  skipped, a card opens in the middle of the window, over a dimmed
  window, with the TraceDraw icon, the name, an UPDATE badge, the two
  versions ("v0.28.0 to 0.29.0") and a line of text. Update now starts
  the install described below (where the copy cannot install by itself
  it opens the Updates box, which offers Download); nothing is
  downloaded before the click. While the file downloads the card shows a
  spinner and the megabytes done; once it is checked, the button becomes
  Restart now (Close and install for the Windows setup and the `.deb`).
  What's new opens the release notes. Dismiss, the cross and Escape
  close it for the session; a card dismissed during the download comes
  back once, when the file is ready. It waits for any dialog (the
  first-start question, recovery) and does not show over one. Help >
  Updates shows its own box instead of the card.
- Every start while an update is known: the check at start runs once a
  day, and also at every start while the last check found a newer release
  that is not installed or skipped (`settings.pending_update`), so the
  card is there each time TraceDraw opens until the update is done. It is
  the same request the daily check makes (GitHub's latest release, nothing
  about the user); a check that finds this version the latest forgets it.
  Skip this version in the Updates box stops it.
- Install in place: a copy that can replace itself shows Install first
  in the dialog (no uninstalling, the settings and drawings stay). It
  needs the release file for this system, processor and kind of install
  to carry a SHA-256 digest (GitHub gives one for every uploaded file); a
  release without one only gets Download. The download shows the
  megabytes done and is checked for size and digest before anything is
  touched; a short, long or mismatched download is deleted, the installed
  copy is left as it was, and the dialog says why and offers Download.
  Six kinds of copy (`updates::InstallTarget`), every one that has a
  release file:
  - Linux AppImage (`APPIMAGE` names the running file): the release's
    AppImage is written next to the running one (`.<name>.update`), given
    the running file's permissions and moved over it. The new version is
    in place (`InstallState::Installed`).
  - Windows installed with setup (not the portable copy, not the Store
    copy): the release's `tracedraw-<v>-windows-<arch>-setup.exe` is
    downloaded to `%TEMP%\tracedraw-update` (`InstallState::Ready`) and
    nothing is run yet.
  - macOS app bundle (`X.app/Contents/MacOS/<program>`; a build run from
    the source tree has no bundle and gets Download): the disk image is
    downloaded, mounted read-only and hidden (`hdiutil attach -nobrowse`),
    its app is copied next to the running bundle with `ditto`, and the
    two are swapped by renaming (the old bundle moves aside first and
    comes back if the new one cannot be moved in). The image is
    unmounted and the download removed on every outcome. A bundle in a
    folder the user cannot write to fails the install and offers
    Download.
  - Linux unpacked from the tar.gz: the archive is downloaded, checked,
    and its `tracedraw` replaces the running program (under the running
    file's name, in case the copy was renamed) and its `tracedraw-cli`
    the tool beside it, if one is there. Every new file is written as a
    hidden `.<name>.update` first and moved over the old one by rename
    (a running program can be replaced that way); a damaged archive, one
    without the program, or a folder that cannot be written changes
    nothing.
  - Windows portable zip: the zip's top-level files (the program, the
    tool, README and licences) replace the ones beside the running
    program. Entries in a folder, with a path separator, a drive or a
    leading dot are skipped, so the `tracedraw-settings` folder, in the
    zip and beside the program, is never touched. Windows cannot
    overwrite a running program but can rename it: the old file is set
    aside as `.<name>.old`, the new one moved in (the old one moves back
    if that fails) and the leftover removed at the next start
    (`updates::clean_leftovers`).
  - Linux installed from the `.deb` (offered only when `pkexec` is on the
    `PATH`): the package is downloaded and checked to
    `$XDG_CACHE_HOME/tracedraw-update` (`~/.cache/...`, mode 0700; never
    a shared `/tmp`, because the file is installed as the administrator
    and must not be swappable after the check) and nothing is run yet
    (`InstallState::Ready`).
  After an in-place install (AppImage, app, tar.gz, portable zip) the
  dialog offers Restart now and
  Later; after a Windows or `.deb` download, Close and install and Later. Later
  closes the dialog and changes nothing: the new version runs from the
  next start (Windows: the downloaded setup is not run). Restart exits
  the usual way (File > Exit): drawings with unsaved changes are asked
  about first, and answering Cancel to any of them cancels Restart
  (`Installer::arm`/`disarm`). At exit, after the settings are saved,
  `Installer::run_pending` starts the new version once:
  - AppImage: the new file, without `APPDIR`, `ARGV0` and `OWD` of the old
    mount;
  - Windows: `cmd /C start "" <setup> /SILENT /SUPPRESSMSGBOXES
    /NORESTART /relaunch=1`. The shell is used so that the setup can ask
    for administrator approval (a copy installed for all users). Setup
    keeps the install folder and install mode of the existing copy, its
    settings and file associations, and `packaging/windows/tracedraw.iss`
    starts the new TraceDraw at the end when `/relaunch=1` is given;
  - macOS: `open -n <bundle>`;
  - tar.gz and portable zip: the program again;
  - `.deb`: `sh -c 'pkexec dpkg -i "$1"; rm -f "$1"; exec "$2"'`: the
    password is asked by the system's authentication dialog, the file is
    removed and the program (`/usr/bin/tracedraw`) starts whether the
    install worked or not (a refused password leaves the old version,
    which starts).
  Copies with no release file (an rpm or other system package, Flatpak,
  Snap) keep Download or their store; the Microsoft Store and Google Play
  copies are updated by the store and the browser build is always the
  latest. The path of the running program is taken once, before an update
  replaces it (afterwards the system no longer names it).
- First start: before any check runs on its own, a question asks
  whether to check for updates, Always check or Never check
  (`Dialog::UpdatePolicy`, settings `update_check_asked`). It waits for
  any other start-up dialog to close. Until it is answered no request is
  made, so a first start is silent on the network. Settings saved by
  older versions have no answer, so the question is asked once after
  updating. The answer sets the option below.
- Options > Global > General, Check for updates at start (set by the
  first-start answer): once a day, the same check runs in the background
  at start; only a newer release that was not skipped is shown (by the update
  card), nothing otherwise.
- A failed check (no connection) says why and offers the releases page.

## Checks

- `updates::tests::versions_compare_by_number`
- `updates::tests::the_download_fits_the_system` (the file names of the
  v0.19.8 release, in GitHub's order and reversed)
- `updates::tests::no_file_of_another_system_is_offered`
- `updates::tests::the_x86_64_appimage_is_named_amd64_from_0_19_9` (the new
  name, the old one as fallback, the order an 0.19.8 updater sees, and the
  name in `build-appimage.sh`)
- `updates::tests::how_a_linux_copy_was_installed`
- `updates::tests::the_same_or_older_drafts_and_pre_releases_are_not_offered`
- `ui::dialogs::tests::every_dialog_draws_and_accepts_enter` (the dialog)
- Checked against GitHub from the app: 0.4.0 reads as the latest version.
- `updates::tests::the_appimage_comes_with_its_digest`
- `updates::tests::installing_replaces_the_file_only_when_it_checks_out`
- `ui::dialogs::tests::the_updates_dialog_draws_every_state`
- `app::tests::no_update_check_before_the_first_start_question`
- `app::tests::the_first_start_answer_sets_the_update_option`
- `app::tests::older_settings_have_not_answered_the_update_question`
- `updates::tests::live_install_from_the_latest_release` (run by hand,
  needs the network): the 0.4.0 AppImage downloaded through GitHub's
  redirect, its digest checked and the file put in place.
  Run again against v0.19.8: the x86_64 AppImage is picked, downloaded
  and checked.
- `updates::tests::the_setup_and_the_disk_image_come_with_their_digest`
- `updates::tests::the_app_bundle_is_found_from_the_program`
- `updates::tests::swapping_the_bundle_never_leaves_the_app_missing`
- `updates::tests::a_downloaded_file_is_kept_only_when_it_checks_out`
- `updates::tests::restart_starts_the_new_version` (the commands, and the
  `/relaunch` entry in `tracedraw.iss`)
- `updates::tests::the_new_version_starts_only_after_restart`
- `ui::dialogs::tests::the_updates_dialog_draws_every_state` (Ready too)
- `documents::tests::cancelling_the_exit_cancels_restart`
- `updates::tests::every_kind_of_copy_with_a_file_updates_in_place`
- `updates::tests::the_tar_gz_replaces_the_program_and_its_tool`
- `updates::tests::a_bad_tar_gz_changes_nothing`
- `updates::tests::the_portable_zip_keeps_the_settings_folder`
- `updates::tests::a_running_program_is_set_aside_not_overwritten`
- `updates::tests::archive_entries_are_installed_under_safe_names_only`
- `updates::tests::the_deb_installs_as_the_administrator_then_starts_the_program`
- Not run in CI: the Windows setup run, the macOS mount and copy and
  `pkexec dpkg` (they need those systems); their commands are built by
  tested functions.
- `ui::update_card::tests::the_card_shows_only_for_a_release_to_update_to`
- `ui::update_card::tests::update_now_installs_when_it_can`
- `ui::update_card::tests::restart_needs_an_update_in_place`
- `ui::update_card::tests::a_dismissed_card_comes_back_when_the_download_is_ready`
- `app::tests::a_found_update_is_remembered_until_it_is_gone`
- `ui::update_card::tests::the_card_draws_in_every_state`
