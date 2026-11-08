# Code signing for Windows

An unsigned installer makes Windows SmartScreen show "Windows protected
your PC" (More info > Run anyway), and some companies block unsigned
programs altogether. Signing tells Windows who published the file and
that it was not changed since. A signature alone does not remove every
warning: SmartScreen also builds a reputation for the publisher and the
file over the first downloads, and that reputation carries over between
releases signed with the same identity.

The build (`.github/workflows/build.yml`) is ready for three routes. Each
one starts working when its settings are added to the repository
(Settings > Secrets and variables > Actions); nothing in the code has to
change, and without them the files stay unsigned as before.

| Route | Cost | Who can use it | Publisher shown | In the build |
|---|---|---|---|---|
| SignPath Foundation | free | open-source projects that meet its terms | SignPath Foundation | signs the programs and the installer of each release after you approve it on signpath.io |
| Microsoft Store (MSIX) | free for individual developers (since September 2025) | anyone with a Partner Center account | your name, from the Store | builds a `.msix` you upload to Partner Center; the Store signs it and Store installs show no SmartScreen warning |
| Azure Artifact Signing (formerly Trusted Signing) | about USD 9.99 a month | organisations in the USA, Canada, the EU and the UK; individuals only in the USA and Canada | your validated name | signs the programs and the installer right in the build |

Other options, not wired into the build: an OV certificate from a
certificate authority (about USD 150 to 300 a year; since June 2023 its key
must live on a hardware token or a cloud HSM, which makes signing in CI
depend on that vendor's tools), the Certum Open Source certificate (low
cost, issued to "Open Source Developer, <your name>", used through the
SimplySign cloud with a code from a phone app, which is awkward to
automate). EV certificates no longer skip the SmartScreen reputation
(since 2024), so they buy nothing over OV for this.

For TraceDraw, a project by one person in the EU, the free routes are the
ones to take: apply to the SignPath Foundation for the installer and the
portable zip, and publish the MSIX in the Microsoft Store.

Status (2026-10-11): the SignPath Foundation declined the first
application. Its program is for projects with public signs of trust
(stars, forks and contributors, articles and discussions elsewhere,
sustained activity), and it invites a new application once the project is
better known. Until then the installer and the portable zip are unsigned,
and the Microsoft Store is the way to a signed install on Windows. The
cheapest paid route for one person in the EU is the Certum Open Source
certificate (from EUR 49 a year in the cloud, EUR 69 with a card, at the
time of writing).

## What gets signed

- `tracedraw.exe` and `tracedraw-cli.exe`, before they are packed (so the
  installer, the portable zip and the MSIX all carry signed programs).
- The installer, `tracedraw-<version>-windows-x86_64-setup.exe`.
- Not signed: the uninstaller the installer writes (it is never
  downloaded, so SmartScreen does not check it) and the macOS and Linux
  packages.

Signing runs for version tags (`v*`). Builds of `main` (the nightly) are
signed too when the repository variable `SIGN_NIGHTLY` is `true`; with
SignPath that means approving every push, so leave it off there.

## SignPath Foundation

1. Make sure the project meets the terms at <https://signpath.org/terms>:
   an OSI license for everything (TraceDraw is MIT or Apache-2.0), no
   malware, released, described on its download page, the "Code signing
   policy" section in the README (already there) and multi-factor
   authentication on GitHub and SignPath for everyone in the team roles.
2. Apply at <https://signpath.org/apply> with the repository URL. The
   certificate is issued to the SignPath Foundation, which appears as the
   publisher.
3. Once accepted, in SignPath create (or check) the project:
   - a signing policy (for example `release-signing`) whose approvers are
     you;
   - two artifact configurations with these slugs and the XML from this
     repository: `programs` (`packaging/windows/signpath/programs.xml`) and
     `installer` (`packaging/windows/signpath/installer.xml`);
   - a trusted build system link to GitHub (SignPath's "GitHub.com"
     connector) for this repository.
4. Create an API token for a user who may submit to that policy (a CI user
   in SignPath, with the Submitter role).
5. In GitHub add:
   - secret `SIGNPATH_API_TOKEN`: the token;
   - variable `SIGNPATH_ORGANIZATION_ID`: your organisation id (SignPath >
     Settings);
   - variable `SIGNPATH_PROJECT_SLUG`: the project slug (for example
     `tracedraw`);
   - variable `SIGNPATH_POLICY_SLUG`: the signing policy slug.
6. Make a release: Actions > build > Run workflow with the version (for
   example `0.3.0`), or push a version tag (`git tag v0.3.0 && git push
   origin v0.3.0`). The Windows
   job uploads the programs, waits (up to an hour) while you approve the
   request in SignPath, packs the installer with the signed programs, then
   sends the installer and waits for the second approval.
7. Add this line under "Code signing policy" in the README, as the
   Foundation asks: "Free code signing provided by
   [SignPath.io](https://about.signpath.io), certificate by
   [SignPath Foundation](https://signpath.org)."

## Microsoft Store

1. Register as an individual developer in Partner Center
   (<https://developer.microsoft.com/microsoft-store/register>, no fee for
   individuals).
2. Create the app and reserve the name "TraceDraw".
3. Open the app's Product management > Product identity page and copy
   three values into GitHub variables:
   - `MSIX_IDENTITY_NAME`: Package/Identity/Name (like
     `12345TraceDraw.TraceDraw`);
   - `MSIX_PUBLISHER`: Package/Identity/Publisher (like `CN=1A2B3C4D-...`);
   - `MSIX_PUBLISHER_NAME`: Package/Properties/PublisherDisplayName (your
     name as the Store shows it).
4. From then on every build attaches `tracedraw-<version>-windows-x86_64.msix`
   to its release (a release `1.2.3` becomes package version `1.2.3.0`;
   development builds use the run number as the fourth number). Upload the
   one of a version tag in a new submission under Packages; the Store
   checks it, signs it and publishes it.
5. For the submission itself (description, features, keywords, the
   privacy policy address and the reason for the full-trust capability)
   see [`store-listing.md`](store-listing.md).
6. The manifest is `packaging/windows/msix/AppxManifest.xml` (the program
   runs with full trust; `.tdraw` and `.cdr` open with it; `tracedraw-cli`
   works in a terminal); its tiles are in
   `packaging/windows/msix/Assets/`. Without the variables the build still
   packs a check package with a placeholder identity, so a broken manifest
   shows up in CI, but does not publish it. The package has a single
   application: the Store refuses a second one hidden from the Start menu (a
   "headless app") without a waiver, and an application may declare only one
   command-line alias, so the Store package has `tracedraw-cli` but not the
   `tracedraw` alias.
7. Submitting each version automatically. Once the app is live in the Store
   (the API cannot make the first submission), the `store` job of the build
   hands every version's MSIX to the Store and submits it for certification:
   the new submission copies the last published one and replaces only the
   package. It needs a Microsoft Entra application with access to the
   account:
   - Partner Center > Account settings > User management > Microsoft Entra
     applications: add an application (create a tenant first if the account
     has none) and give it the Manager role;
   - in the Entra admin center (<https://entra.microsoft.com/>), on that
     application: Certificates & secrets > New client secret;
   - in GitHub add the secrets `MSSTORE_TENANT_ID` (Identity > Overview >
     Tenant ID), `MSSTORE_CLIENT_ID` (the application's Application (client)
     ID), `MSSTORE_CLIENT_SECRET` (the secret's value) and
     `MSSTORE_SELLER_ID` (Partner Center > Account settings > Legal info >
     Seller ID), and the variable `MSSTORE_APP_ID` (the Store ID,
     `9P50L12VHF33`).

   Without them the job only notes that nothing was submitted. A draft
   left in Partner Center is discarded by the next version (`msstore
   publish` starts again from the last published submission), so finish
   or submit manual edits before tagging.

## Azure Artifact Signing

Only if you have an organisation in the EU or the UK (or live in the USA
or Canada):

1. Create an Artifact Signing account and an identity validation in the
   Azure portal, then a certificate profile (Public Trust).
2. Create an app registration (service principal) with a client secret
   and give it the "Artifact Signing Certificate Profile Signer" role on
   the account.
3. In GitHub add the secrets `AZURE_TENANT_ID`, `AZURE_CLIENT_ID` and
   `AZURE_CLIENT_SECRET`, and the variables `ARTIFACT_SIGNING_ENDPOINT`
   (the account's region endpoint, like `https://weu.codesigning.azure.net/`),
   `ARTIFACT_SIGNING_ACCOUNT` and `ARTIFACT_SIGNING_PROFILE`.
4. The next tagged build signs the programs and the installer with
   Microsoft's timestamp server. If both Azure and SignPath are set up,
   Azure is used.

## Checking a signature

In Windows, right-click the file > Properties > Digital Signatures, or in
PowerShell:

```powershell
Get-AuthenticodeSignature .\tracedraw-0.3.0-windows-x86_64-setup.exe | Format-List
```

`Status` should be `Valid` and `SignerCertificate` should name the
publisher. The job summary of each build says which signing route ran
("Windows code signing: signpath", "azure" or "none").
