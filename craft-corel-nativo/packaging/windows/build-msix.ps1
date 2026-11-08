# Build the MSIX package of TraceDraw for the Microsoft Store.
#
# The Store signs the package itself, so this package is not signed. The
# identity comes from the app's Product identity page in Partner Center,
# passed in the environment (see docs/code-signing.md):
#   MSIX_IDENTITY_NAME   Package/Identity/Name, e.g. 12345TraceDraw.TraceDraw
#   MSIX_PUBLISHER       Package/Identity/Publisher, e.g. CN=1A2B3C4D-...
#   MSIX_PUBLISHER_NAME  Package/Properties/PublisherDisplayName
# With them the package goes to dist\ (and so into the release); without
# them a check package with a placeholder identity is built in msix-check\,
# which only proves that the manifest and the files pack.
#
# Usage: build-msix.ps1 -Version 1.2.3 -SrcDir target\x86_64-pc-windows-msvc\release

param(
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$SrcDir
)

$ErrorActionPreference = 'Stop'
trap {
    Write-Host "::error title=MSIX::$($_.Exception.Message)"
    exit 1
}
$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')

# MSIX versions are four numbers below 65536; the Store wants the last one
# to be 0. A release "1.2.3" becomes 1.2.3.0; a development build
# "0.1.0-dev.abc1234" becomes 0.1.0.<run number>.
if ($Version -notmatch '^(\d+)\.(\d+)\.(\d+)(.*)$') {
    throw "Version '$Version' does not start with three numbers"
}
$revision = 0
if ($Matches[4] -ne '') {
    $revision = [int]($env:GITHUB_RUN_NUMBER) % 65536
}
$msixVersion = "$($Matches[1]).$($Matches[2]).$($Matches[3]).$revision"

$identity = $env:MSIX_IDENTITY_NAME
$publisher = $env:MSIX_PUBLISHER
$publisherName = $env:MSIX_PUBLISHER_NAME
$store = $identity -and $publisher -and $publisherName
if ($store) {
    $outDir = Join-Path $root 'dist'
} else {
    Write-Host 'MSIX: no Store identity set, building a check package only'
    $identity = 'TraceDraw.Check'
    $publisher = 'CN=TraceDraw Check'
    $publisherName = 'TraceDraw'
    $outDir = Join-Path $root 'msix-check'
}

function Escape-Xml([string]$text) {
    return [System.Security.SecurityElement]::Escape($text)
}

$stage = Join-Path $env:RUNNER_TEMP 'tracedraw-msix'
if (-not $env:RUNNER_TEMP) { $stage = Join-Path ([System.IO.Path]::GetTempPath()) 'tracedraw-msix' }
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force (Join-Path $stage 'Assets') | Out-Null

Copy-Item (Join-Path $SrcDir 'tracedraw.exe') $stage
Copy-Item (Join-Path $SrcDir 'tracedraw-cli.exe') $stage
foreach ($f in 'README.md', 'LICENSE-MIT', 'LICENSE-APACHE') {
    Copy-Item (Join-Path $root $f) $stage
}
Copy-Item (Join-Path $PSScriptRoot 'msix\Assets\*.png') (Join-Path $stage 'Assets')

$manifest = Get-Content (Join-Path $PSScriptRoot 'msix\AppxManifest.xml') -Raw
$manifest = $manifest.Replace('{IDENTITY}', (Escape-Xml $identity))
$manifest = $manifest.Replace('{PUBLISHER}', (Escape-Xml $publisher))
$manifest = $manifest.Replace('{PUBLISHER_NAME}', (Escape-Xml $publisherName))
$manifest = $manifest.Replace('{VERSION}', $msixVersion)
[System.IO.File]::WriteAllText((Join-Path $stage 'AppxManifest.xml'), $manifest, (New-Object System.Text.UTF8Encoding $false))

# makeappx.exe from the newest Windows SDK on the machine.
$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
    Sort-Object FullName -Descending | Select-Object -First 1
if (-not $makeappx) { throw 'makeappx.exe not found (Windows SDK)' }

New-Item -ItemType Directory -Force $outDir | Out-Null
$out = Join-Path $outDir "tracedraw-$Version-windows-x86_64.msix"
Write-Host "makeappx: $($makeappx.FullName)"
$log = & $makeappx.FullName pack /d $stage /p $out /o 2>&1 | Out-String
Write-Host $log
if ($LASTEXITCODE -ne 0) {
    # The lines that say what went wrong, as annotations on the run.
    $log -split "`r?`n" | Where-Object { $_ -match 'error|0x8|line \d+' } |
        Select-Object -First 8 | ForEach-Object { Write-Host "::error title=MSIX::$($_.Trim())" }
    throw "makeappx failed with exit code $LASTEXITCODE"
}
Write-Host "MSIX $msixVersion -> $out"
