//! The system's printers for the Print dialog's Printer list: the
//! printers CUPS knows (Linux and macOS), listed by `lpstat` and printed
//! to with `lp`, which takes the print PDF as it is; on Windows every
//! printer Windows has, local ones and network connections alike, listed
//! and printed to through PowerShell (the sheets drawn by Craft Plumilla and
//! handed to the printer as pictures, so no PDF program is needed). In
//! the browser the list is empty and the print goes through the PDF
//! viewer, as before; on Android it is empty too and the print goes to
//! the system's print service, which lists the printers itself. See
//! docs/behavior/printing.md.

use std::path::Path;

/// What a printer is doing, as the system reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrinterState {
    #[default]
    Unknown,
    Ready,
    Printing,
    WarmingUp,
    Stopped,
    Offline,
}

impl PrinterState {
    /// The i18n key of the Print dialog's Status row.
    pub fn key(self) -> &'static str {
        match self {
            PrinterState::Unknown => "print.status_unknown",
            PrinterState::Ready => "print.status_ready",
            PrinterState::Printing => "print.status_printing",
            PrinterState::WarmingUp => "print.status_warming_up",
            PrinterState::Stopped => "print.status_stopped",
            PrinterState::Offline => "print.status_offline",
        }
    }
}

/// A printer the system has.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Printer {
    /// The name `lp -d` takes.
    pub name: String,
    /// The system's default destination.
    pub default: bool,
    /// The Print dialog's Status, Location and Comment rows.
    pub state: PrinterState,
    pub location: String,
    pub comment: String,
}

/// `printers` in name order (ignoring case); the default stays where its
/// name puts it (#614).
pub fn by_name(mut printers: Vec<Printer>) -> Vec<Printer> {
    printers.sort_by_cached_key(|p| (p.name.to_lowercase(), p.name.clone()));
    printers
}

/// The printers `lpstat -l -p -d` lists ("printer NAME is idle..." lines,
/// each followed by indented details such as "Description:" and
/// "Location:", and "system default destination: NAME"), in name order.
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub fn parse_lpstat(output: &str) -> Vec<Printer> {
    let mut out: Vec<Printer> = Vec::new();
    let mut default: Option<String> = None;
    // The printer the indented lines below belong to.
    let mut current: Option<usize> = None;
    for raw in output.lines() {
        let indented = raw.starts_with([' ', '\t']);
        let line = raw.trim();
        if indented {
            let Some(p) = current.and_then(|i| out.get_mut(i)) else {
                continue;
            };
            if let Some(v) = line.strip_prefix("Description:") {
                p.comment = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("Location:") {
                p.location = v.trim().to_string();
            }
            continue;
        }
        current = None;
        if let Some(rest) = line.strip_prefix("printer ") {
            let mut words = rest.splitn(2, char::is_whitespace);
            let name = words.next().unwrap_or_default();
            let after = words.next().unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let state = if after.contains("now printing") {
                PrinterState::Printing
            } else if after.contains("is idle") {
                PrinterState::Ready
            } else if after.contains("disabled") {
                PrinterState::Stopped
            } else {
                PrinterState::Unknown
            };
            match out.iter().position(|p| p.name == name) {
                Some(i) => current = Some(i),
                None => {
                    out.push(Printer {
                        name: name.to_string(),
                        state,
                        ..Default::default()
                    });
                    current = Some(out.len() - 1);
                }
            }
        } else if let Some(rest) = line.strip_prefix("system default destination:") {
            let name = rest.trim();
            if !name.is_empty() {
                default = Some(name.to_string());
            }
        }
    }
    if let Some(name) = default {
        if let Some(p) = out.iter_mut().find(|p| p.name == name) {
            p.default = true;
        }
    }
    by_name(out)
}

/// The PowerShell command that lists Windows' printers, one per line:
/// name, default (True or False), PrinterStatus, WorkOffline, location
/// and comment, separated by tabs. `Win32_Printer` holds the printers
/// installed on the computer and the network printers connected for the
/// user (#614).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub const WINDOWS_LIST: &str = "[Console]::OutputEncoding = [Text.Encoding]::UTF8; \
Get-CimInstance Win32_Printer | ForEach-Object { \
\"$($_.Name)`t$($_.Default)`t$($_.PrinterStatus)`t$($_.WorkOffline)`t\
$($_.Location -replace '\\s+', ' ')`t$($_.Comment -replace '\\s+', ' ')\" }";

/// The state of a `Win32_Printer`'s PrinterStatus (3 idle, 4 printing,
/// 5 warming up, 6 stopped, 7 offline), offline whatever it says when
/// WorkOffline is set.
fn windows_state(status: &str, offline: &str) -> PrinterState {
    if offline.trim().eq_ignore_ascii_case("true") {
        return PrinterState::Offline;
    }
    match status.trim().parse::<u32>() {
        Ok(3) => PrinterState::Ready,
        Ok(4) => PrinterState::Printing,
        Ok(5) => PrinterState::WarmingUp,
        Ok(6) => PrinterState::Stopped,
        Ok(7) => PrinterState::Offline,
        _ => PrinterState::Unknown,
    }
}

/// The printers [`WINDOWS_LIST`] printed, in name order.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn parse_windows_list(output: &str) -> Vec<Printer> {
    let mut out: Vec<Printer> = Vec::new();
    for line in output.lines() {
        let line = line
            .trim_end_matches(['\r', '\n'])
            .trim_start_matches('\u{feff}');
        let fields: Vec<&str> = line.split('\t').collect();
        let field = |i: usize| fields.get(i).map(|f| f.trim()).unwrap_or("");
        let name = field(0);
        if name.is_empty() || out.iter().any(|p| p.name == name) {
            continue;
        }
        out.push(Printer {
            name: name.to_string(),
            default: field(1).eq_ignore_ascii_case("true"),
            state: windows_state(field(2), field(3)),
            location: field(4).to_string(),
            comment: field(5).to_string(),
        });
    }
    by_name(out)
}

/// The printers the system has, in name order; empty where none is set
/// up or there is no CUPS. It can take seconds (a print server that does
/// not answer): the Print dialog calls it off the interface's thread.
#[cfg(all(
    not(test),
    not(any(target_arch = "wasm32", target_os = "windows", target_os = "android"))
))]
pub fn system_printers() -> Vec<Printer> {
    let out = std::process::Command::new("lpstat")
        .args(["-l", "-p", "-d"])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output();
    match out {
        Ok(o) => parse_lpstat(&String::from_utf8_lossy(&o.stdout)),
        Err(e) => {
            log::info!("no printer list: lpstat: {e}");
            Vec::new()
        }
    }
}

#[cfg(all(target_os = "windows", not(test)))]
pub fn system_printers() -> Vec<Printer> {
    match powershell().args(["-Command", WINDOWS_LIST]).output() {
        Ok(o) => parse_windows_list(&String::from_utf8_lossy(&o.stdout)),
        Err(e) => {
            log::info!("no printer list: powershell: {e}");
            Vec::new()
        }
    }
}

/// PowerShell without a profile, a prompt or a console window.
#[cfg(target_os = "windows")]
fn powershell() -> std::process::Command {
    use std::os::windows::process::CommandExt as _;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut c = std::process::Command::new("powershell.exe");
    c.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
    ])
    .creation_flags(CREATE_NO_WINDOW);
    c
}

/// None in a browser, and none on Android, whose print service lists and
/// picks the printers in its own dialog (#1398).
#[cfg(any(test, target_arch = "wasm32", target_os = "android"))]
pub fn system_printers() -> Vec<Printer> {
    Vec::new()
}

/// The printer the dialog starts on: the one remembered when the system
/// still has it, else the system's default, else the first listed, else
/// none (the print goes through the PDF viewer).
pub fn starting_printer(remembered: Option<&str>, printers: &[Printer]) -> Option<String> {
    if let Some(r) = remembered {
        if printers.iter().any(|p| p.name == r) {
            return Some(r.to_string());
        }
    }
    printers
        .iter()
        .find(|p| p.default)
        .or_else(|| printers.first())
        .map(|p| p.name.clone())
}

/// The program and arguments that open `printer`'s own settings (the
/// Print dialog's Properties button): on Windows the printer's printing
/// preferences, on macOS the system's Printers & Scanners settings, on
/// Linux and other CUPS systems the printer's page of the CUPS web
/// interface in the browser.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub fn properties_command(printer: &str) -> (String, Vec<String>) {
    if cfg!(target_os = "windows") {
        (
            "rundll32.exe".into(),
            vec![
                "printui.dll,PrintUIEntry".into(),
                "/e".into(),
                "/n".into(),
                printer.into(),
            ],
        )
    } else if cfg!(target_os = "macos") {
        (
            "open".into(),
            vec!["x-apple.systempreferences:com.apple.Print-Scan-Settings.extension".into()],
        )
    } else {
        let mut url = String::from("http://localhost:631/printers/");
        for b in printer.bytes() {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                url.push(b as char);
            } else {
                url.push_str(&format!("%{b:02X}"));
            }
        }
        ("xdg-open".into(), vec![url])
    }
}

/// Open `printer`'s settings ([`properties_command`]); Err says why not.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_properties(printer: &str) -> Result<(), String> {
    let (program, args) = properties_command(printer);
    let mut c = std::process::Command::new(&program);
    c.args(&args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c.spawn().map(|_| ()).map_err(|e| format!("{program}: {e}"))
}

#[cfg(target_arch = "wasm32")]
pub fn open_properties(printer: &str) -> Result<(), String> {
    Err(format!("no printer {printer} here"))
}

/// The `lp` command that prints `pdf` on `printer`: `copies` copies,
/// collated as whole sets or sheet by sheet, under `title` in the queue.
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub fn lp_args(printer: &str, pdf: &Path, copies: u32, collate: bool, title: &str) -> Vec<String> {
    let mut args = vec!["-d".to_string(), printer.to_string()];
    if copies > 1 {
        args.push("-n".into());
        args.push(copies.min(999).to_string());
        args.push("-o".into());
        args.push(format!(
            "collate={}",
            if collate { "true" } else { "false" }
        ));
    }
    let title = title.trim();
    if !title.is_empty() {
        args.push("-t".into());
        args.push(title.to_string());
    }
    args.push("--".into());
    args.push(pdf.display().to_string());
    args
}

/// Send `pdf` to `printer` through `lp`. Ok with what `lp` said (the
/// request id), Err with its complaint.
#[cfg(not(any(target_arch = "wasm32", target_os = "windows")))]
pub fn print_pdf(
    printer: &str,
    pdf: &Path,
    copies: u32,
    collate: bool,
    title: &str,
) -> Result<String, String> {
    let out = std::process::Command::new("lp")
        .args(lp_args(printer, pdf, copies, collate, title))
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("lp: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() {
            format!("lp: {}", out.status)
        } else {
            err
        })
    }
}

#[cfg(any(target_arch = "wasm32", target_os = "windows"))]
#[allow(dead_code)]
pub fn print_pdf(
    printer: &str,
    _pdf: &Path,
    _copies: u32,
    _collate: bool,
    _title: &str,
) -> Result<String, String> {
    Err(format!("no printer {printer} here"))
}

/// `s` as a PowerShell string in single quotes (the quote marks
/// PowerShell takes for one, typographic ones too, doubled).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn ps_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if matches!(c, '\'' | '\u{2018}' | '\u{2019}' | '\u{201a}' | '\u{201b}') {
            out.push(c);
        }
        out.push(c);
    }
    out.push('\'');
    out
}

/// One picture of part of a printed sheet: its PNG and where it goes, in
/// mm from the sheet's top-left corner.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct Tile {
    pub png: Vec<u8>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A sheet for a Windows printer: its size in mm and the pictures of its
/// parts that are not blank.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct Sheet {
    pub width: f64,
    pub height: f64,
    pub tiles: Vec<Tile>,
}

/// A tile written to disk for the print script: its file and place in mm.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct TileFile {
    pub path: std::path::PathBuf,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The PowerShell script that prints `sheets` (each its width and height
/// in mm and its tiles: picture files drawn at `dpi` with their place in
/// mm) on the Windows printer `printer`: `copies` copies, collated or
/// not, under `title` in the queue. Each sheet takes the printer's paper
/// of its size when it has one (either way round), turns the paper for a
/// wide sheet, and is drawn at its size from the paper's corner, the
/// printer's unprintable edge taken off, as a PDF viewer prints a page at
/// actual size. Tiles drawn at the printer's own resolution go pixel for
/// pixel; others are smoothed, their edges never blended with the paper,
/// so strips meet with no seam (#1404). Exits with 2 when the printer is
/// gone.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn windows_print_script(
    printer: &str,
    sheets: &[(f64, f64, Vec<TileFile>)],
    dpi: f64,
    copies: u32,
    collate: bool,
    title: &str,
) -> String {
    // Lists are built with `Add`: `@(@(w, h))` of a single entry flattens
    // to `@(w, h)` in PowerShell, so `$sizes[0][0]` was `$null`, the paper
    // size check threw "Argument types do not match" and nothing reached
    // the spooler whatever the printer (#614).
    let mut list = String::new();
    for (w, h, tiles) in sheets {
        list.push_str("$t = New-Object System.Collections.ArrayList\n");
        for t in tiles {
            list.push_str(&format!(
                "[void]$t.Add(@({}, {:.4}, {:.4}, {:.4}, {:.4}))\n",
                ps_quote(&t.path.display().to_string()),
                t.x,
                t.y,
                t.width,
                t.height
            ));
        }
        list.push_str(&format!(
            "[void]$sheets.Add(@{{ w = {w:.3}; h = {h:.3}; tiles = $t }})\n"
        ));
    }
    format!(
        r#"$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$sheets = New-Object System.Collections.ArrayList
{list}$doc = New-Object System.Drawing.Printing.PrintDocument
$doc.PrinterSettings.PrinterName = {printer}
if (-not $doc.PrinterSettings.IsValid) {{ [Console]::Error.WriteLine('Printer not found: ' + {printer}); exit 2 }}
$doc.DocumentName = {title}
$doc.PrinterSettings.Copies = {copies}
$doc.PrinterSettings.Collate = ${collate}
$doc.OriginAtMargins = $false
# No "Printing page" window from this process, which has none.
$doc.PrintController = New-Object System.Drawing.Printing.StandardPrintController
$state = @{{ i = 0 }}
$doc.add_QueryPageSettings({{
    param($sender, $e)
    $w = $sheets[$state.i].w; $h = $sheets[$state.i].h
    foreach ($p in $doc.PrinterSettings.PaperSizes) {{
        $pw = $p.Width * 0.254; $ph = $p.Height * 0.254
        if (([Math]::Abs($pw - [Math]::Min($w, $h)) -lt 2) -and ([Math]::Abs($ph - [Math]::Max($w, $h)) -lt 2)) {{
            $e.PageSettings.PaperSize = $p
            break
        }}
    }}
    $e.PageSettings.Landscape = ($w -gt $h)
}})
$doc.add_PrintPage({{
    param($sender, $e)
    $g = $e.Graphics
    $g.PageUnit = [System.Drawing.GraphicsUnit]::Millimeter
    if ([Math]::Abs($g.DpiX - {dpi}) -lt 1 -and [Math]::Abs($g.DpiY - {dpi}) -lt 1) {{
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    }} else {{
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    }}
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    $attr = New-Object System.Drawing.Imaging.ImageAttributes
    $attr.SetWrapMode([System.Drawing.Drawing2D.WrapMode]::TileFlipXY)
    $mx = $e.PageSettings.HardMarginX * 0.254; $my = $e.PageSettings.HardMarginY * 0.254
    foreach ($tile in $sheets[$state.i].tiles) {{
        $img = [System.Drawing.Image]::FromFile($tile[0])
        $x = [single]($tile[1] - $mx); $y = [single]($tile[2] - $my)
        $to = [System.Drawing.PointF[]]@(
            (New-Object System.Drawing.PointF($x, $y)),
            (New-Object System.Drawing.PointF([single]($x + $tile[3]), $y)),
            (New-Object System.Drawing.PointF($x, [single]($y + $tile[4]))))
        $from = New-Object System.Drawing.RectangleF(0, 0, $img.Width, $img.Height)
        $g.DrawImage($img, $to, $from, [System.Drawing.GraphicsUnit]::Pixel, $attr)
        $img.Dispose()
    }}
    $attr.Dispose()
    $state.i++
    $e.HasMorePages = $state.i -lt $sheets.Count
}})
$doc.Print()
[Console]::Out.WriteLine('sent ' + $state.i + ' sheet(s) at {dpi} dpi to ' + {printer})
"#,
        printer = ps_quote(printer),
        title = ps_quote(title),
        dpi = dpi.round(),
        copies = copies.clamp(1, 999),
        collate = if collate { "true" } else { "false" },
    )
}

/// Print `sheets` (their tiles drawn at `dpi`) on the Windows printer
/// `printer` through [`windows_print_script`]. Ok when Windows took the
/// job, Err with what went wrong.
#[cfg(target_os = "windows")]
pub fn print_sheets_windows(
    printer: &str,
    sheets: &[Sheet],
    dpi: f64,
    copies: u32,
    collate: bool,
    title: &str,
) -> Result<String, String> {
    let dir = std::env::temp_dir().join("tracedraw-print");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut files = Vec::with_capacity(sheets.len());
    let mut written = Vec::new();
    for (k, sheet) in sheets.iter().enumerate() {
        let mut tiles = Vec::with_capacity(sheet.tiles.len());
        for (n, t) in sheet.tiles.iter().enumerate() {
            let path = dir.join(format!("sheet-{}-{}.png", k + 1, n + 1));
            std::fs::write(&path, &t.png).map_err(|e| e.to_string())?;
            written.push(path.clone());
            tiles.push(TileFile {
                path,
                x: t.x,
                y: t.y,
                width: t.width,
                height: t.height,
            });
        }
        files.push((sheet.width, sheet.height, tiles));
    }
    let script = dir.join("print.ps1");
    // With its byte order mark PowerShell reads the script as UTF-8, so
    // printer and file names in any script survive.
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend_from_slice(
        windows_print_script(printer, &files, dpi, copies, collate, title).as_bytes(),
    );
    std::fs::write(&script, bytes).map_err(|e| e.to_string())?;
    let result = run_print_script(&script, PRINT_TIMEOUT);
    for path in &written {
        let _ = std::fs::remove_file(path);
    }
    // What happened goes in the log, so a print that did not come out
    // can be told apart from one that Windows took (#614).
    match &result {
        Ok(said) => log::info!("print on {printer}: {said}"),
        Err(e) => log::warn!("print on {printer} failed: {e}"),
    }
    result.map(|_| String::new())
}

/// The PowerShell lines that say the resolution of the printer `printer`:
/// "X Y" of its current setting, then of each it offers.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn windows_resolution_script(printer: &str) -> String {
    format!(
        r#"Add-Type -AssemblyName System.Drawing
$s = New-Object System.Drawing.Printing.PrinterSettings
$s.PrinterName = {printer}
if (-not $s.IsValid) {{ exit 2 }}
$r = $s.DefaultPageSettings.PrinterResolution
[Console]::Out.WriteLine([string]$r.X + ' ' + [string]$r.Y)
foreach ($r in $s.PrinterResolutions) {{ [Console]::Out.WriteLine([string]$r.X + ' ' + [string]$r.Y) }}
"#,
        printer = ps_quote(printer)
    )
}

/// The resolution in dots per inch from [`windows_resolution_script`]'s
/// output: the current setting's finer direction when it is given in dots
/// (a named quality such as "High" comes as a negative number), else the
/// finest the printer offers; `None` when it says nothing usable.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn parse_windows_resolution(output: &str) -> Option<f64> {
    let dots: Vec<Option<f64>> = output
        .lines()
        .map(|l| {
            let mut it = l.split_whitespace().map(|v| v.parse::<f64>().ok());
            let (x, y) = (it.next()??, it.next()??);
            (x > 0.0 && y > 0.0).then_some(x.max(y))
        })
        .collect();
    match dots.first() {
        Some(Some(d)) => Some(*d),
        _ => dots
            .iter()
            .skip(1)
            .flatten()
            .copied()
            .fold(None, |m: Option<f64>, d| Some(m.map_or(d, |m| m.max(d)))),
    }
}

/// The resolution the Windows printer `printer` prints at, `None` when it
/// cannot be told.
#[cfg(target_os = "windows")]
pub fn printer_resolution_windows(printer: &str) -> Option<f64> {
    let out = powershell()
        .arg("-Command")
        .arg(windows_resolution_script(printer))
        .output()
        .ok()?;
    let found = out
        .status
        .success()
        .then(|| parse_windows_resolution(&String::from_utf8_lossy(&out.stdout)))
        .flatten();
    log::info!("print on {printer}: resolution {found:?}");
    found
}

/// The edges of the paper a printer cannot print on (its hard margins),
/// mm, the paper upright (portrait) as the printer feeds it (#1924).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardMargins {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl HardMargins {
    /// What is shown when the system does not say: a typical 5 mm.
    pub const FALLBACK: HardMargins = HardMargins {
        left: 5.0,
        right: 5.0,
        top: 5.0,
        bottom: 5.0,
    };

    /// Each margin a sensible size: none negative, none past half the
    /// paper (a value that is not believable gives `None`).
    fn checked(self, paper_w: f64, paper_h: f64) -> Option<HardMargins> {
        let m = [self.left, self.right, self.top, self.bottom];
        let ok = m.iter().all(|v| v.is_finite() && *v >= -0.01)
            && self.left + self.right < paper_w
            && self.top + self.bottom < paper_h;
        ok.then(|| HardMargins {
            left: self.left.max(0.0),
            right: self.right.max(0.0),
            top: self.top.max(0.0),
            bottom: self.bottom.max(0.0),
        })
    }

    /// The margins of a sheet `width` x `height`: as they are on an
    /// upright sheet; on a sheet lying on its side (wider than tall) the
    /// paper is fed upright and the sheet's top lies along the paper's
    /// left edge (turned a quarter counterclockwise), so the sheet's
    /// left is the paper's bottom.
    pub fn for_sheet(self, width: f64, height: f64) -> HardMargins {
        if width > height {
            HardMargins {
                left: self.bottom,
                right: self.top,
                top: self.left,
                bottom: self.right,
            }
        } else {
            self
        }
    }
}

/// The hard margins in a CUPS PPD file for the paper `size` (a PageSize
/// name such as "A4"), else its default size: the `*ImageableArea`
/// (lower left and upper right corners, points) inside the
/// `*PaperDimension` (points), PPD 4.3. `None` when the file does not
/// give both.
pub fn parse_ppd_margins(ppd: &str, size: Option<&str>) -> Option<HardMargins> {
    let default = ppd.lines().find_map(|l| {
        let l = l.trim();
        l.strip_prefix("*DefaultImageableArea:")
            .or_else(|| l.strip_prefix("*DefaultPageSize:"))
            .map(|v| v.trim().to_string())
    });
    let size = size.map(str::to_string).or(default)?;
    let numbers = |key: &str| -> Option<Vec<f64>> {
        ppd.lines().find_map(|l| {
            let rest = l.trim().strip_prefix(key)?;
            let (name, value) = rest.split_once(':')?;
            let name = name.split('/').next()?.trim();
            if name != size {
                return None;
            }
            let v: Vec<f64> = value
                .trim()
                .trim_matches('"')
                .split_whitespace()
                .filter_map(|n| n.parse().ok())
                .collect();
            Some(v)
        })
    };
    let area = numbers("*ImageableArea ")?;
    let paper = numbers("*PaperDimension ")?;
    let ([llx, lly, urx, ury], [w, h]) = (area.get(..4)?, paper.get(..2)?) else {
        return None;
    };
    let mm = 25.4 / 72.0;
    HardMargins {
        left: llx * mm,
        bottom: lly * mm,
        right: (w - urx) * mm,
        top: (h - ury) * mm,
    }
    .checked(w * mm, h * mm)
}

/// The paper size `lpoptions -p NAME` says the printer is set to (its
/// `media` or `PageSize` option), as a PPD names it.
pub fn parse_lpoptions_size(output: &str) -> Option<String> {
    output.split_whitespace().find_map(|o| {
        let (k, v) = o.split_once('=')?;
        if !matches!(k, "media" | "PageSize") {
            return None;
        }
        // A list of media ("A4,Tray1") starts with the size.
        let v = v.trim_matches('\'').split(',').next()?;
        let v = match v.to_ascii_lowercase().as_str() {
            "a4" | "iso_a4_210x297mm" => "A4".to_string(),
            "a3" | "iso_a3_297x420mm" => "A3".to_string(),
            "a5" | "iso_a5_148x210mm" => "A5".to_string(),
            "letter" | "na_letter_8.5x11in" => "Letter".to_string(),
            "legal" | "na_legal_8.5x14in" => "Legal".to_string(),
            _ => v.to_string(),
        };
        Some(v)
    })
}

/// The hard margins of the CUPS printer `printer`, from its PPD file and
/// the paper size it is set to; `None` when CUPS has no PPD for it (a
/// driverless queue) or the file does not say.
#[cfg(all(
    not(test),
    not(any(target_arch = "wasm32", target_os = "windows", target_os = "android"))
))]
pub fn printer_margins(printer: &str) -> Option<HardMargins> {
    if printer.is_empty() || printer.contains(['/', '\\']) || printer.starts_with('.') {
        return None;
    }
    let ppd = std::fs::read(format!("/etc/cups/ppd/{printer}.ppd")).ok()?;
    let size = std::process::Command::new("lpoptions")
        .args(["-p", printer])
        .env("LC_ALL", "C")
        .output()
        .ok()
        .and_then(|o| parse_lpoptions_size(&String::from_utf8_lossy(&o.stdout)));
    let found = parse_ppd_margins(&String::from_utf8_lossy(&ppd), size.as_deref());
    log::info!("printer {printer}: hard margins {found:?}");
    found
}

/// The PowerShell lines that say the paper and printable area of the
/// printer `printer`'s default page, upright, in hundredths of an inch:
/// "W H X Y AW AH" (paper width and height, then the printable area's
/// left, top, width and height).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn windows_margins_script(printer: &str) -> String {
    format!(
        r#"Add-Type -AssemblyName System.Drawing
$s = New-Object System.Drawing.Printing.PrinterSettings
$s.PrinterName = {printer}
if (-not $s.IsValid) {{ exit 2 }}
$p = $s.DefaultPageSettings
$p.Landscape = $false
$z = $p.PaperSize; $a = $p.PrintableArea
$c = [Globalization.CultureInfo]::InvariantCulture
[Console]::Out.WriteLine([string]::Format($c, '{{0}} {{1}} {{2}} {{3}} {{4}} {{5}}', $z.Width, $z.Height, $a.X, $a.Y, $a.Width, $a.Height))
"#,
        printer = ps_quote(printer)
    )
}

/// The hard margins from [`windows_margins_script`]'s output.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn parse_windows_margins(output: &str) -> Option<HardMargins> {
    let v: Vec<f64> = output
        .lines()
        .next()?
        .split_whitespace()
        .filter_map(|n| n.parse().ok())
        .collect();
    let [w, h, x, y, aw, ah] = v.get(..6)? else {
        return None;
    };
    if *w <= 0.0 || *h <= 0.0 || *aw <= 0.0 || *ah <= 0.0 {
        return None;
    }
    let mm = 0.254;
    HardMargins {
        left: x * mm,
        top: y * mm,
        right: (w - x - aw) * mm,
        bottom: (h - y - ah) * mm,
    }
    .checked(w * mm, h * mm)
}

/// The hard margins of the Windows printer `printer` (its default
/// paper's printable area, which the driver reports as GDI's device
/// offsets do), `None` when it cannot be told.
#[cfg(all(target_os = "windows", not(test)))]
pub fn printer_margins(printer: &str) -> Option<HardMargins> {
    let out = powershell()
        .arg("-Command")
        .arg(windows_margins_script(printer))
        .output()
        .ok()?;
    let found = out
        .status
        .success()
        .then(|| parse_windows_margins(&String::from_utf8_lossy(&out.stdout)))
        .flatten();
    log::info!("printer {printer}: hard margins {found:?}");
    found
}

/// No printer's margins in a browser or on Android (its print service
/// lays the page out).
#[cfg(any(test, target_arch = "wasm32", target_os = "android"))]
pub fn printer_margins(_printer: &str) -> Option<HardMargins> {
    None
}

type KnownMargins = std::collections::HashMap<Option<String>, Option<HardMargins>>;

/// The hard margins of printers, read once each off the interface's
/// thread (a printer's driver can take a second to answer) and kept.
#[derive(Debug, Default)]
pub struct MarginsCache {
    /// By printer name (`None`: the system's default printer): the
    /// margins it reported, `None` inside when it reported none; missing
    /// while not read yet.
    known: std::sync::Arc<std::sync::Mutex<KnownMargins>>,
    asked: std::sync::Mutex<std::collections::HashSet<Option<String>>>,
}

impl MarginsCache {
    /// The hard margins of `printer` (`None`: the system's default),
    /// once read; until then, or when the system does not say, `None`.
    /// The first call starts the reading.
    pub fn get(&self, printer: Option<&str>) -> Option<HardMargins> {
        let key = printer.map(str::to_string);
        if let Ok(known) = self.known.lock() {
            if let Some(m) = known.get(&key) {
                return *m;
            }
        }
        let first = self
            .asked
            .lock()
            .map(|mut a| a.insert(key.clone()))
            .unwrap_or(false);
        if first && !crate::files::WEB {
            let known = self.known.clone();
            let _ = std::thread::Builder::new()
                .name("printer-margins".into())
                .spawn(move || {
                    let name = key
                        .clone()
                        .or_else(|| starting_printer(None, &system_printers()));
                    let m = name.as_deref().and_then(printer_margins);
                    if let Ok(mut k) = known.lock() {
                        k.insert(key, m);
                    }
                });
        }
        None
    }

    /// Set what `printer` reports (tests).
    #[cfg(test)]
    pub fn set(&self, printer: Option<&str>, margins: Option<HardMargins>) {
        if let Ok(mut k) = self.known.lock() {
            k.insert(printer.map(str::to_string), margins);
        }
        if let Ok(mut a) = self.asked.lock() {
            a.insert(printer.map(str::to_string));
        }
    }
}

/// How long the print script may run before it is taken as hung.
#[cfg(target_os = "windows")]
const PRINT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// Run the print script, stopping it when it runs longer than `limit`;
/// Ok with what it said when it ended well, Err with its error.
#[cfg(target_os = "windows")]
fn run_print_script(
    script: &std::path::Path,
    limit: std::time::Duration,
) -> Result<String, String> {
    use std::io::Read as _;
    let mut child = powershell()
        .arg("-File")
        .arg(script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("powershell: {e}"))?;
    let start = web_time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if start.elapsed() > limit => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "the printer did not take the job within {} s",
                    limit.as_secs()
                ));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
            Err(e) => return Err(format!("powershell: {e}")),
        }
    };
    let mut out = String::new();
    let mut err = String::new();
    if let Some(mut o) = child.stdout.take() {
        let _ = o.read_to_string(&mut out);
    }
    if let Some(mut e) = child.stderr.take() {
        let _ = e.read_to_string(&mut err);
    }
    if status.success() {
        Ok(out.trim().to_string())
    } else {
        let err = err.trim().to_string();
        Err(if err.is_empty() {
            format!("powershell: {status}")
        } else {
            err
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Printer list comes from `lpstat`: every printer in name order,
    /// the default marked where it stands, with its state, description
    /// and location (#300, #614).
    #[test]
    fn the_printer_list_is_read_from_lpstat() {
        let out = "printer Office_HP disabled since Mon 20 Oct 2026 09:00:00 AM -\n\
                   \tPaused\n\
                   \tDescription: HP LaserJet 4250\n\
                   \tLocation: Room 3-102\n\
                   printer EPSON-L3250-Series is idle.  enabled since Tue 21 Oct 2026 10:00:00 AM\n\
                   \tDescription: EPSON L3250 Series\n\
                   \tLocation:\n\
                   printer PDF now printing PDF-12.  enabled since Tue 21 Oct 2026 10:00:00 AM\n\
                   system default destination: Office_HP\n";
        let p = parse_lpstat(out);
        let names: Vec<&str> = p.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["EPSON-L3250-Series", "Office_HP", "PDF"]);
        assert!(!p[0].default && p[1].default && !p[2].default);
        assert_eq!(p[1].state, PrinterState::Stopped);
        assert_eq!(p[1].comment, "HP LaserJet 4250");
        assert_eq!(p[1].location, "Room 3-102");
        assert_eq!(p[0].state, PrinterState::Ready);
        assert_eq!(p[0].location, "");
        assert_eq!(p[2].state, PrinterState::Printing);
        // No printers, or no default: the list as given.
        assert!(parse_lpstat("lpstat: No destinations added.\n").is_empty());
        assert!(parse_lpstat("no system default destination\n").is_empty());
        let p =
            parse_lpstat("printer A is idle.\nprinter B is idle.\nno system default destination\n");
        assert_eq!(p.len(), 2);
        assert!(!p[0].default);
    }

    /// Windows lists its printers through PowerShell, network ones too,
    /// in name order with the default marked in place, their state,
    /// location and comment (#614).
    #[test]
    fn the_windows_printer_list_holds_network_printers() {
        let out = "\u{feff}Microsoft Print to PDF\tFalse\t3\tFalse\t\t\r\n\
                   \\\\office-srv\\HP LaserJet 4250\tTrue\t4\tFalse\tRoom 3-102\tColour, A3\r\n\
                   \\\\office-srv\\Canon iR-ADV C5535\tFalse\t3\tTrue\t\t\r\n\
                   \r\n\
                   Fax\tFalse\r\n";
        let p = parse_windows_list(out);
        let names: Vec<&str> = p.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "\\\\office-srv\\Canon iR-ADV C5535",
                "\\\\office-srv\\HP LaserJet 4250",
                "Fax",
                "Microsoft Print to PDF",
            ]
        );
        assert!(p[1].default && !p[0].default && !p[3].default);
        assert_eq!(p[1].state, PrinterState::Printing);
        assert_eq!(p[1].location, "Room 3-102");
        assert_eq!(p[1].comment, "Colour, A3");
        assert_eq!(p[0].state, PrinterState::Offline, "WorkOffline");
        assert_eq!(p[3].state, PrinterState::Ready);
        assert_eq!(p[2].state, PrinterState::Unknown, "an older listing");
        assert!(WINDOWS_LIST.contains("PrinterStatus") && WINDOWS_LIST.contains("Comment"));
        assert!(parse_windows_list("").is_empty());
        // The listing command asks for every printer Windows has.
        assert!(WINDOWS_LIST.contains("Win32_Printer"));
    }

    fn tile(path: &str, x: f64, y: f64, w: f64, h: f64) -> TileFile {
        TileFile {
            path: std::path::PathBuf::from(path),
            x,
            y,
            width: w,
            height: h,
        }
    }

    /// The Windows print script names the printer, the copies and the
    /// sheets with their tiles, quoting what it is given.
    #[test]
    fn the_windows_print_script_names_the_printer_copies_and_sheets() {
        let sheets = [
            (
                210.0,
                297.0,
                vec![
                    tile(r"C:\Temp\sheet-1-1.png", 10.0, 0.0, 190.0, 68.25),
                    tile(r"C:\Temp\sheet-1-2.png", 12.5, 68.25, 20.0, 5.0),
                ],
            ),
            (297.0, 210.0, vec![]),
        ];
        let s = windows_print_script(r"\\srv\Bob's printer", &sheets, 600.0, 3, false, "Poster");
        assert!(
            s.contains(r"$doc.PrinterSettings.PrinterName = '\\srv\Bob''s printer'"),
            "{s}"
        );
        assert!(s.contains("$doc.PrinterSettings.Copies = 3"));
        assert!(s.contains("$doc.PrinterSettings.Collate = $false"));
        assert!(
            s.contains(
                "$t = New-Object System.Collections.ArrayList\n\
                 [void]$t.Add(@('C:\\Temp\\sheet-1-1.png', 10.0000, 0.0000, 190.0000, 68.2500))\n\
                 [void]$t.Add(@('C:\\Temp\\sheet-1-2.png', 12.5000, 68.2500, 20.0000, 5.0000))\n\
                 [void]$sheets.Add(@{ w = 210.000; h = 297.000; tiles = $t })\n\
                 $t = New-Object System.Collections.ArrayList\n\
                 [void]$sheets.Add(@{ w = 297.000; h = 210.000; tiles = $t })\n"
            ),
            "{s}"
        );
        // A sheet with one tile keeps a list of one, not a flattened
        // entry: the flattened form made every one-sheet print fail (#614).
        let one = windows_print_script("P", &sheets[..1], 600.0, 1, false, "");
        assert!(!one.contains("@(@("), "{one}");
        assert!(s.contains("$doc.DocumentName = 'Poster'"));
        // No status window from a process that has no window.
        let controller = s
            .find(
                "$doc.PrintController = New-Object System.Drawing.Printing.StandardPrintController",
            )
            .expect("the standard print controller");
        assert!(controller < s.find("$doc.Print()").expect("Print"));
        // What was sent is said, for the log.
        assert!(s.contains("[Console]::Out.WriteLine('sent '"), "{s}");
        // Typographic quotes close a PowerShell string too.
        assert_eq!(ps_quote("a\u{2019}b"), "'a\u{2019}\u{2019}b'");
        assert!(windows_print_script("P", &sheets, 600.0, 0, true, "").contains("Copies = 1"));
    }

    /// Tiles drawn at the printer's own resolution go pixel for pixel,
    /// others smoothed; their edges are never blended with the paper, so
    /// the strips of a sheet meet with no seam, and nothing is rounded to
    /// whole millimetres (#1404).
    #[test]
    fn windows_tiles_go_pixel_for_pixel_at_the_printers_resolution() {
        let sheets = [(210.0, 297.0, vec![tile("a.png", 1.0, 2.0, 3.0, 4.0)])];
        let s = windows_print_script("P", &sheets, 1200.0, 1, false, "");
        assert!(
            s.contains(
                "if ([Math]::Abs($g.DpiX - 1200) -lt 1 -and [Math]::Abs($g.DpiY - 1200) -lt 1)"
            ),
            "{s}"
        );
        let exact = s
            .find("InterpolationMode]::NearestNeighbor")
            .expect("exact");
        let smooth = s
            .find("InterpolationMode]::HighQualityBicubic")
            .expect("smooth");
        assert!(exact < smooth);
        assert!(s.contains("SetWrapMode([System.Drawing.Drawing2D.WrapMode]::TileFlipXY)"));
        assert!(s.contains(
            "$g.DrawImage($img, $to, $from, [System.Drawing.GraphicsUnit]::Pixel, $attr)"
        ));
        assert!(!s.contains("Rectangle]::Round"), "{s}");
        assert!(s.contains("sheet(s) at 1200 dpi"), "{s}");
    }

    /// The printer's resolution: its current setting, else the finest it
    /// offers when the setting is a named quality (#1404).
    #[test]
    fn the_windows_printer_resolution_is_read() {
        assert_eq!(
            parse_windows_resolution("600 600\r\n300 300\r\n600 600\r\n"),
            Some(600.0)
        );
        assert_eq!(parse_windows_resolution("1200 600\n"), Some(1200.0));
        // "High" is -4: the finest offered instead.
        assert_eq!(
            parse_windows_resolution("-4 -1\n-4 -1\n300 300\n1200 1200\n600 600\n"),
            Some(1200.0)
        );
        assert_eq!(parse_windows_resolution("-3 -1\n-3 -1\n"), None);
        assert_eq!(parse_windows_resolution(""), None);
        assert_eq!(parse_windows_resolution("x y\n"), None);
        let script = windows_resolution_script("Bob's");
        assert!(script.contains("$s.PrinterName = 'Bob''s'"), "{script}");
        assert!(script.contains("DefaultPageSettings.PrinterResolution"));
        assert!(script.contains("foreach ($r in $s.PrinterResolutions)"));
    }

    /// Write the print script for the printer in `PRINTER_NAME`, one A4
    /// sheet in two tiles at 600 dpi, to the path in `PRINT_SCRIPT_OUT`
    /// (a check on a Windows machine runs it as Craft Plumilla does);
    /// `OLD_CONTROLLER=1` leaves the standard controller out, as before
    /// #614.
    #[test]
    #[ignore]
    fn dump_windows_print_script() {
        let (Ok(out), Ok(printer)) = (
            std::env::var("PRINT_SCRIPT_OUT"),
            std::env::var("PRINTER_NAME"),
        ) else {
            return;
        };
        let dir = std::path::Path::new(&out)
            .parent()
            .expect("folder")
            .to_path_buf();
        let mut tiles = Vec::new();
        for (k, colour) in [[200, 30, 30, 255], [30, 30, 200, 255]]
            .into_iter()
            .enumerate()
        {
            let png = dir.join(format!("sheet-1-{}.png", k + 1));
            image::RgbaImage::from_pixel(4961, 3508, image::Rgba(colour))
                .save(&png)
                .expect("tile");
            let y = k as f64 * 148.5;
            tiles.push(TileFile {
                path: png,
                x: 0.0,
                y,
                width: 4961.0 * 25.4 / 600.0,
                height: 3508.0 * 25.4 / 600.0,
            });
        }
        let mut script =
            windows_print_script(&printer, &[(210.0, 297.0, tiles)], 600.0, 1, false, "Check");
        if std::env::var("OLD_CONTROLLER").is_ok_and(|v| v == "1") {
            script = script
                .lines()
                .filter(|l| !l.contains("StandardPrintController"))
                .collect::<Vec<_>>()
                .join("\n");
        }
        let mut bytes = vec![0xef, 0xbb, 0xbf];
        bytes.extend_from_slice(script.as_bytes());
        std::fs::write(&out, bytes).expect("script");
    }

    #[test]
    fn the_dialog_starts_on_the_remembered_printer_else_the_default() {
        let printers =
            parse_lpstat("printer A is idle.\nprinter B is idle.\nsystem default destination: B\n");
        assert_eq!(starting_printer(Some("A"), &printers).as_deref(), Some("A"));
        assert_eq!(
            starting_printer(Some("Gone"), &printers).as_deref(),
            Some("B")
        );
        assert_eq!(starting_printer(None, &printers).as_deref(), Some("B"));
        assert_eq!(starting_printer(Some("A"), &[]), None);
        // The default is chosen where it stands, not moved first.
        let printers = parse_lpstat(
            "printer A is idle.\nprinter B is idle.\nprinter C is idle.\nsystem default destination: C\n",
        );
        assert_eq!(printers[2].name, "C");
        assert_eq!(starting_printer(None, &printers).as_deref(), Some("C"));
    }

    /// Properties opens the printer's own settings: its printing
    /// preferences on Windows, Printers & Scanners on macOS, its CUPS web
    /// page elsewhere (#614).
    #[test]
    fn properties_open_the_printers_own_settings() {
        let (program, args) = properties_command("Office HP/2");
        if cfg!(target_os = "windows") {
            assert_eq!(program, "rundll32.exe");
            assert_eq!(
                args,
                ["printui.dll,PrintUIEntry", "/e", "/n", "Office HP/2"]
            );
        } else if cfg!(target_os = "macos") {
            assert_eq!(program, "open");
            assert_eq!(
                args,
                ["x-apple.systempreferences:com.apple.Print-Scan-Settings.extension"]
            );
        } else {
            assert_eq!(program, "xdg-open");
            assert_eq!(args, ["http://localhost:631/printers/Office%20HP%2F2"]);
        }
    }

    /// `lp` gets the printer, the copies with their collation, the title
    /// and the file; one copy needs no count.
    #[test]
    fn lp_is_told_the_printer_copies_and_title() {
        let pdf = Path::new("/tmp/tracedraw-print/Poster.pdf");
        assert_eq!(
            lp_args("EPSON-L3250-Series", pdf, 1, true, "Poster"),
            [
                "-d",
                "EPSON-L3250-Series",
                "-t",
                "Poster",
                "--",
                "/tmp/tracedraw-print/Poster.pdf"
            ]
        );
        assert_eq!(
            lp_args("A", pdf, 3, false, ""),
            [
                "-d",
                "A",
                "-n",
                "3",
                "-o",
                "collate=false",
                "--",
                "/tmp/tracedraw-print/Poster.pdf"
            ]
        );
        assert_eq!(
            lp_args("A", pdf, 2, true, " ")[4..6],
            ["-o", "collate=true"]
        );
    }

    /// #1924: a PPD's imageable area for the paper the printer is set
    /// to (else its default) gives the hard margins, points to mm.
    #[test]
    fn the_hard_margins_come_from_the_ppd() {
        let ppd = r#"*PPD-Adobe: "4.3"
*DefaultPageSize: Letter
*DefaultImageableArea: Letter
*ImageableArea Letter/US Letter: "18 36 594 756"
*ImageableArea A4/A4: "12.24 12.24 582.76 829.65"
*PaperDimension Letter/US Letter: "612 792"
*PaperDimension A4/A4: "595.28 841.89"
"#;
        let mm = 25.4 / 72.0;
        let letter = parse_ppd_margins(ppd, None).expect("default size");
        assert!((letter.left - 18.0 * mm).abs() < 1e-9);
        assert!((letter.bottom - 36.0 * mm).abs() < 1e-9);
        assert!((letter.right - 18.0 * mm).abs() < 1e-9);
        assert!((letter.top - 36.0 * mm).abs() < 1e-9);
        let a4 = parse_ppd_margins(ppd, Some("A4")).expect("A4");
        assert!(
            (a4.left - 12.24 * mm).abs() < 1e-9 && (a4.top - (841.89 - 829.65) * mm).abs() < 1e-9
        );
        assert_eq!(parse_ppd_margins(ppd, Some("A3")), None);
        assert_eq!(parse_ppd_margins("*PPD-Adobe: \"4.3\"", None), None);
        assert_eq!(
            parse_lpoptions_size(
                "copies=1 device-uri=usb://x media=iso_a4_210x297mm sides=one-sided"
            ),
            Some("A4".into())
        );
        assert_eq!(
            parse_lpoptions_size("PageSize=Letter"),
            Some("Letter".into())
        );
        assert_eq!(parse_lpoptions_size("copies=1"), None);
    }

    /// #1924: the Windows printable area (hundredths of an inch) gives
    /// the hard margins on each side; nonsense gives none. A sheet on its
    /// side takes them turned a quarter.
    #[test]
    fn the_windows_printable_area_gives_the_hard_margins() {
        // A4 with 4.2 mm left and top, 5 mm right, 6.2 mm bottom.
        let out = "827 1169 16.5 16.5 790.8 1128.1\n";
        let m = parse_windows_margins(out).expect("margins");
        assert!((m.left - 16.5 * 0.254).abs() < 1e-9);
        assert!((m.top - 16.5 * 0.254).abs() < 1e-9);
        assert!((m.right - (827.0 - 16.5 - 790.8) * 0.254).abs() < 1e-9);
        assert!((m.bottom - (1169.0 - 16.5 - 1128.1) * 0.254).abs() < 1e-9);
        assert_eq!(parse_windows_margins(""), None);
        assert_eq!(parse_windows_margins("827 1169 500 0 800 1100"), None);
        let s = windows_margins_script("Office's Laser");
        assert!(s.contains("PrintableArea") && s.contains("'Office''s Laser'"));
        let side = m.for_sheet(297.0, 210.0);
        assert_eq!(
            (side.left, side.right, side.top, side.bottom),
            (m.bottom, m.top, m.left, m.right)
        );
        assert_eq!(m.for_sheet(210.0, 297.0), m);
    }
}
