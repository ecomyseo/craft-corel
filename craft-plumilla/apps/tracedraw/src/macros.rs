//! VBA macro projects in the app: loading `.gms` files and exported
//! modules, keeping a copy of each in the settings folder (`macros/`) so
//! they load again at start-up, the warnings a project gets before it
//! runs, and running a macro (`scripting::vba_run`) with what the app does
//! with its messages, forms, changes and errors. See docs/behavior/vba.md.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::scripting::vba_run::{Changes, FormEvent, Input, MacroRun, Request, RunError, Snapshot};
use std::path::{Path, PathBuf};
use tracedraw_vba as vba;
use tracedraw_vba_project as vp;

/// A loaded macro project.
#[derive(Debug, Clone)]
pub struct MacroProject {
    /// Its copy in the settings folder.
    pub file: PathBuf,
    pub project: vp::Project,
    /// What may not work here, found by reading its code.
    pub warnings: Vec<String>,
}

fn vba_kind(k: vp::ModuleKind) -> vba::ModuleKind {
    match k {
        vp::ModuleKind::Standard => vba::ModuleKind::Standard,
        vp::ModuleKind::Class => vba::ModuleKind::Class,
        vp::ModuleKind::Form => vba::ModuleKind::Form,
        vp::ModuleKind::Document => vba::ModuleKind::Document,
    }
}

impl MacroProject {
    pub fn new(file: PathBuf, project: vp::Project) -> MacroProject {
        let warnings = scan(&project);
        MacroProject {
            file,
            project,
            warnings,
        }
    }

    pub fn name(&self) -> &str {
        &self.project.name
    }

    /// The modules for the compiler (those with code).
    pub fn modules(&self) -> Vec<vba::Module> {
        self.project
            .modules
            .iter()
            .filter(|m| m.has_source)
            .map(|m| vba::Module::new(m.name.clone(), vba_kind(m.kind), m.source.clone()))
            .collect()
    }

    /// The macros it offers: (module, macro).
    pub fn macros(&self) -> Vec<(String, String)> {
        vba::list_macros(&self.modules())
    }

    /// What the compiler knows of the host and the forms.
    pub fn options(&self) -> vba::Options {
        vba::Options {
            host_names: crate::scripting::vg_global_names(),
            form_controls: self
                .project
                .forms
                .iter()
                .map(|f| {
                    (
                        f.name.clone(),
                        f.all_controls().iter().map(|c| c.name.clone()).collect(),
                    )
                })
                .collect(),
        }
    }

    pub fn compile(&self) -> Result<vba::Program, vba::CompileError> {
        vba::compile_with(&self.modules(), &self.options())
    }

    /// Write the project to its copy (the code as it is now).
    pub fn save(&self) -> Result<(), String> {
        let bytes = vp::write_gms(&self.project).map_err(|e| e.to_string())?;
        std::fs::write(&self.file, bytes).map_err(|e| e.to_string())
    }
}

/// What may not work in a project, read from its code: Windows functions
/// (`Declare`), file statements, objects of other programs, modules saved
/// without code, a protected project.
pub fn scan(p: &vp::Project) -> Vec<String> {
    let mut out = Vec::new();
    let mut declares = Vec::new();
    let mut files = false;
    let mut objects = Vec::new();
    let mut shell = false;
    for m in &p.modules {
        for line in m.source.lines() {
            let l = line.trim();
            if l.starts_with('\'') {
                continue;
            }
            let low = l.to_ascii_lowercase();
            let decl = low
                .strip_prefix("private ")
                .or_else(|| low.strip_prefix("public "))
                .unwrap_or(&low);
            if decl.starts_with("declare ") {
                let name = l
                    .split_whitespace()
                    .skip_while(|w| {
                        !w.eq_ignore_ascii_case("sub") && !w.eq_ignore_ascii_case("function")
                    })
                    .nth(1)
                    .unwrap_or("")
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>();
                if !name.is_empty() && !declares.contains(&name) {
                    declares.push(name);
                }
                continue;
            }
            let starts = |w: &str| low.starts_with(w);
            if (starts("open ") && low.contains(" for "))
                || starts("kill ")
                || starts("filecopy ")
                || starts("mkdir ")
                || starts("rmdir ")
                || low.contains("freefile")
                || low.contains(" dir(")
            {
                files = true;
            }
            if low.contains("shell(") || starts("shell ") {
                shell = true;
            }
            let mut rest = low.as_str();
            while let Some(i) = rest.find("createobject(\"") {
                let after = &rest[i + 14..];
                let name = after.split('"').next().unwrap_or("");
                if !name.eq_ignore_ascii_case("scripting.dictionary")
                    && !objects
                        .iter()
                        .any(|o: &String| o.eq_ignore_ascii_case(name))
                {
                    // The name as written in the code.
                    let at = l.to_ascii_lowercase().find(name).unwrap_or(0);
                    objects.push(l.get(at..at + name.len()).unwrap_or(name).to_string());
                }
                rest = after;
            }
        }
    }
    if p.protected {
        out.push(tr("macros.warn_protected"));
    }
    let without = p.modules_without_source();
    if !without.is_empty() {
        out.push(trf("macros.warn_no_source", &[("m", &without.join(", "))]));
    }
    if !declares.is_empty() {
        out.push(trf("macros.warn_declare", &[("f", &declares.join(", "))]));
    }
    if files {
        out.push(tr("macros.warn_files"));
    }
    if shell {
        out.push(tr("macros.warn_shell"));
    }
    if !objects.is_empty() {
        out.push(trf("macros.warn_objects", &[("o", &objects.join(", "))]));
    }
    for w in &p.warnings {
        out.push(w.clone());
    }
    out
}

/// Where loaded projects are kept.
pub fn default_dir() -> Option<PathBuf> {
    crate::settings::config_dir().map(|d| d.join("macros"))
}

/// Read a project from files the user chose: one `.gms` (or any file with
/// a VBA project inside), or module files (`.bas`, `.cls`, `.frm` with
/// their `.frx`).
pub fn read_project(paths: &[PathBuf]) -> Result<vp::Project, String> {
    let is_module = |p: &Path| {
        p.extension().and_then(|e| e.to_str()).is_some_and(|e| {
            ["bas", "cls", "frm", "frx"].contains(&e.to_ascii_lowercase().as_str())
        })
    };
    if paths.iter().all(|p| is_module(p)) {
        let mut files = Vec::new();
        for p in paths {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            files.push((name, crate::files::read(p).map_err(|e| e.to_string())?));
        }
        let name = paths
            .iter()
            .find(|p| !p.extension().is_some_and(|e| e.eq_ignore_ascii_case("frx")))
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Macros".into());
        return vp::read_named(&name, &files).map_err(|e| e.to_string());
    }
    let first = paths.first().ok_or("no file")?;
    let bytes = crate::files::read(first).map_err(|e| e.to_string())?;
    vp::read_gms(&bytes).map_err(|e| e.to_string())
}

/// The line (1-based) where `Sub name` or `Function name` starts.
pub fn sub_line(source: &str, name: &str) -> Option<u32> {
    source.lines().enumerate().find_map(|(i, l)| {
        (proc_name(l).is_some_and(|n| n.eq_ignore_ascii_case(name))).then_some(i as u32 + 1)
    })
}

/// The procedure a line declares: `[Public|Private] [Static] Sub|Function
/// Name(`.
fn proc_name(line: &str) -> Option<String> {
    let mut words = line.split_whitespace().peekable();
    while let Some(w) = words.peek() {
        let l = w.to_ascii_lowercase();
        if matches!(l.as_str(), "public" | "private" | "friend" | "static") {
            words.next();
        } else {
            break;
        }
    }
    let kw = words.next()?.to_ascii_lowercase();
    if kw != "sub" && kw != "function" {
        return None;
    }
    let rest = words.next()?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// The Sub the caret is in (line 1-based), for Run Sub (F5).
pub fn sub_at(source: &str, line: u32) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = (line.max(1) as usize - 1).min(lines.len().saturating_sub(1));
    loop {
        let l = lines.get(i)?.trim().to_ascii_lowercase();
        if (l.starts_with("end sub") || l.starts_with("end function")) && i + 1 != line as usize {
            return None;
        }
        if let Some(n) = proc_name(lines[i]) {
            return Some(n);
        }
        if i == 0 {
            return None;
        }
        i -= 1;
    }
}

/// The macros side of the app: projects, the running macro and what it
/// shows.
#[derive(Default)]
pub struct MacroState {
    pub projects: Vec<MacroProject>,
    /// The folder the copies live in (`None`: the settings folder).
    pub dir: Option<PathBuf>,
    pub loaded: bool,
    pub run: Option<MacroRun>,
    /// A message box or input box the macro waits for.
    pub prompt: Option<Prompt>,
    /// A run that ended with an error: the error and the project.
    pub error: Option<(RunError, usize)>,
    /// Warnings shown before running a project for the first time.
    pub warned: std::collections::BTreeSet<String>,
    /// The drawing's revision after the run's last changes (to add later
    /// ones to the same undo step).
    pub applied: Option<u64>,
    /// Run Macro (Alt+Shift+F8).
    pub run_dialog: Option<RunDialog>,
    /// The macro editor (Alt+F11).
    pub editor: Option<Editor>,
    /// The Macro Manager's chosen row: (project, module, macro).
    pub chosen: Option<(usize, Option<String>, Option<String>)>,
    /// Renaming a project in the manager: (project, the name typed).
    pub renaming: Option<(usize, String)>,
    /// Scripts panel: the snippet is VBA, and the other language's text
    /// (swapped in when the language changes).
    pub scripts_vba: bool,
    pub other_source: String,
    /// A warning dialog before a run: (project, module, macro).
    pub confirm: Option<(usize, String, String)>,
}

/// A message or input box on screen.
pub enum Prompt {
    MsgBox {
        prompt: String,
        buttons: i32,
        title: String,
    },
    InputBox {
        prompt: String,
        title: String,
        text: String,
    },
}

/// Run Macro's state.
#[derive(Default)]
pub struct RunDialog {
    pub filter: String,
    /// "Project|Module|Macro" chosen.
    pub chosen: Option<(usize, String, String)>,
    /// The search field got the focus.
    pub focused: bool,
}

/// The macro editor's state.
#[derive(Default)]
pub struct Editor {
    pub project: usize,
    pub module: String,
    pub source: String,
    pub dirty: bool,
    /// A line to show (an error's), 1-based.
    pub goto_line: Option<u32>,
    pub find: String,
    pub message: Option<String>,
    /// The caret's line, for F5 (Run Sub).
    pub caret_line: u32,
}

impl MacroState {
    fn folder(&self) -> Option<PathBuf> {
        self.dir.clone().or_else(default_dir)
    }

    /// Load the copies kept in the settings folder (once).
    pub fn ensure_loaded(&mut self) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        if crate::files::WEB {
            return;
        }
        let Some(dir) = self.folder() else {
            return;
        };
        let Ok(rd) = std::fs::read_dir(&dir) else {
            return;
        };
        let mut paths: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("gms")))
            .collect();
        paths.sort();
        for p in paths {
            if let Ok(bytes) = std::fs::read(&p) {
                if let Ok(project) = vp::read_gms(&bytes) {
                    self.projects.push(MacroProject::new(p, project));
                }
            }
        }
    }

    /// Keep a copy of `project` in the settings folder and add it. A project
    /// of the same name is replaced.
    pub fn install(
        &mut self,
        project: vp::Project,
        original: Option<&[u8]>,
    ) -> Result<usize, String> {
        self.ensure_loaded();
        let dir = self.folder().ok_or("no settings folder")?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let stem: String = project
            .name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let stem = if stem.is_empty() {
            "Macros".to_string()
        } else {
            stem
        };
        let file = dir.join(format!("{stem}.gms"));
        let bytes = match original {
            Some(b) => b.to_vec(),
            None => vp::write_gms(&project).map_err(|e| e.to_string())?,
        };
        std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
        let mp = MacroProject::new(file.clone(), project);
        if let Some(i) = self.projects.iter().position(|p| p.file == file) {
            self.projects[i] = mp;
            Ok(i)
        } else {
            self.projects.push(mp);
            Ok(self.projects.len() - 1)
        }
    }

    /// Unload project `i` and delete its copy.
    pub fn remove(&mut self, i: usize) {
        if i < self.projects.len() {
            let p = self.projects.remove(i);
            let _ = std::fs::remove_file(&p.file);
        }
    }

    /// Rename project `i` (its name and its copy's file).
    pub fn rename(&mut self, i: usize, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(tr("macros.name_empty"));
        }
        let dir = self.folder().ok_or("no settings folder")?;
        let p = self.projects.get_mut(i).ok_or("no project")?;
        p.project.name = name.to_string();
        let stem: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let file = dir.join(format!("{stem}.gms"));
        let old = std::mem::replace(&mut p.file, file);
        p.save()?;
        if old != p.file {
            let _ = std::fs::remove_file(old);
        }
        Ok(())
    }

    /// Whether something of the macros blocks the rest of the app: a
    /// message, an input box, an error, Run Macro or a modal form.
    pub fn modal(&self) -> bool {
        self.prompt.is_some()
            || self.error.is_some()
            || self.run_dialog.is_some()
            || self.confirm.is_some()
            || self
                .run
                .as_ref()
                .and_then(|r| r.forms.lock().ok().map(|f| f.modal_visible()))
                .unwrap_or(false)
    }
}

/// Every macro of every project: (project, module, macro).
pub fn all_macros(state: &MacroState) -> Vec<(usize, String, String)> {
    let mut out = Vec::new();
    for (i, p) in state.projects.iter().enumerate() {
        for (m, s) in p.macros() {
            out.push((i, m, s));
        }
    }
    out
}

/// Dots would end the key's last part early (see `i18n::tr`).
fn key_part(s: &str) -> String {
    s.replace('.', " ")
}

/// The command key of a macro for menus and shortcuts.
pub fn command_key(project: &str, module: &str, name: &str) -> String {
    format!("menu.macro.{}|{module}|{name}", key_part(project))
}

/// The key of a project's submenu in Tools > Macros (shown as its name).
pub fn project_menu_key(project: &str) -> String {
    format!("menu.macros.project.{}", key_part(project))
}

/// A macro command's name ("Module.Macro") and project, for
/// Options > Customization.
pub fn command_name(key: &str) -> Option<(String, String)> {
    let rest = key.strip_prefix("menu.macro.")?;
    let mut parts = rest.splitn(3, '|');
    let project = parts.next()?.to_string();
    let module = parts.next()?;
    let name = parts.next()?;
    Some((format!("{module}.{name}"), project))
}

impl App {
    /// The browser version has no macros (they run on a thread of their
    /// own, and the browser build has none): any macro command shows a
    /// message saying so, with the desktop app's downloads. True there.
    pub fn macros_desktop_only(&mut self) -> bool {
        if crate::files::WEB {
            self.dialog = crate::ui::dialogs::Dialog::MacrosDesktopOnly;
        }
        crate::files::WEB
    }

    /// Tools > Macros > Load Macro Project: pick the files and install them.
    pub fn load_macro_project(&mut self) {
        if self.macros_desktop_only() {
            return;
        }
        crate::files::Dialog::new()
            .add_filter(tr("macros.filter"), &["gms", "bas", "cls", "frm"])
            .pick_files(self, |app, paths| app.install_macro_files(&paths));
    }

    /// Install the macro project in `paths`.
    pub fn install_macro_files(&mut self, paths: &[PathBuf]) {
        let mut paths: Vec<PathBuf> = paths.to_vec();
        // A form's .frx goes with its .frm.
        let frms: Vec<PathBuf> = paths
            .iter()
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("frm")))
            .cloned()
            .collect();
        for f in frms {
            let frx = f.with_extension("frx");
            if frx.exists() && !paths.contains(&frx) {
                paths.push(frx);
            }
        }
        let original = if paths.len() == 1
            && paths[0]
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("gms"))
        {
            crate::files::read(&paths[0]).ok()
        } else {
            None
        };
        match read_project(&paths).and_then(|p| self.macros.install(p, original.as_deref())) {
            Ok(i) => {
                let p = &self.macros.projects[i];
                self.status = trf(
                    "macros.loaded",
                    &[("name", p.name()), ("n", &p.macros().len().to_string())],
                );
                self.macros.chosen = Some((i, None, None));
                self.show_macro_manager();
            }
            Err(e) => self.status = trf("macros.load_failed", &[("e", &e)]),
        }
    }

    pub fn show_macro_manager(&mut self) {
        if self.macros_desktop_only() {
            return;
        }
        self.macros.ensure_loaded();
        self.open_docker(crate::app::DockerTab::Macros);
    }

    /// Run `module.name` of project `project`, after showing its warnings
    /// once.
    pub fn request_macro(&mut self, project: usize, module: &str, name: &str) {
        let Some(p) = self.macros.projects.get(project) else {
            return;
        };
        let key = p.file.display().to_string();
        if !p.warnings.is_empty() && !self.macros.warned.contains(&key) {
            self.macros.confirm = Some((project, module.to_string(), name.to_string()));
            return;
        }
        self.run_macro(project, module, name);
    }

    /// Start a macro now.
    pub fn run_macro(&mut self, project: usize, module: &str, name: &str) {
        if self.macros_desktop_only() {
            return;
        }
        if self.macros.run.is_some() {
            self.status = tr("macros.already_running");
            return;
        }
        let Some(p) = self.macros.projects.get(project) else {
            return;
        };
        let program = match p.compile() {
            Ok(prog) => prog,
            Err(e) => {
                self.macros.error = Some((
                    RunError {
                        number: 0,
                        description: trf("macros.compile_error", &[("e", &e.message)]),
                        module: e.module.clone(),
                        procedure: String::new(),
                        line: e.line,
                    },
                    project,
                ));
                return;
            }
        };
        let entry = vba::entry_call(&program, module, name);
        let label = format!("{}.{module}.{name}", p.name());
        let models = p.project.forms.clone();
        match crate::scripting::vba_run::start(self, &program.js, &entry, models, label.clone()) {
            Ok(run) => {
                self.macros.run = Some(run);
                self.macros.applied = None;
                self.status = trf("macros.running", &[("name", &label)]);
            }
            Err(e) => self.status = trf("macros.load_failed", &[("e", &e)]),
        }
    }

    /// Run a VBA snippet from the Scripts panel: its first public Sub
    /// without arguments, or the code as the body of one.
    pub fn run_vba_snippet(&mut self, src: &str) {
        let mut module = vba::Module::new("Snippet", vba::ModuleKind::Standard, src);
        let macros = vba::list_macros(std::slice::from_ref(&module));
        let name = match macros.first() {
            Some((_, n)) => n.clone(),
            None => {
                module.source = format!("Sub Main()\r\n{src}\r\nEnd Sub\r\n");
                "Main".into()
            }
        };
        let project = vp::Project {
            name: "Scripts".into(),
            modules: vec![vp::Module {
                name: "Snippet".into(),
                kind: vp::ModuleKind::Standard,
                source: module.source.clone(),
                has_source: true,
                read_only: false,
                private: false,
            }],
            code_page: 1252,
            ..Default::default()
        };
        let mp = MacroProject::new(PathBuf::new(), project);
        let program = match mp.compile() {
            Ok(p) => p,
            Err(e) => {
                self.scripts_output.push(format!("Error: {e}"));
                return;
            }
        };
        let entry = vba::entry_call(&program, "Snippet", &name);
        match crate::scripting::vba_run::start(
            self,
            &program.js,
            &entry,
            Vec::new(),
            "Scripts".into(),
        ) {
            Ok(run) => {
                self.macros.run = Some(run);
                self.macros.applied = None;
            }
            Err(e) => self.scripts_output.push(format!("Error: {e}")),
        }
    }

    /// Stop the running macro.
    pub fn stop_macro(&mut self) {
        if let Some(run) = &self.macros.run {
            run.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if self.macros.prompt.take().is_some() {
            // The macro waits for the dialog: it sees the stop.
        }
    }

    /// Apply a batch of the macro's changes: the first starts an undo step
    /// named "Macro", the next ones join it while nothing else happened.
    pub fn apply_macro_changes(&mut self, c: Changes) {
        if !c.commands.is_empty() {
            let joined = self.macros.applied == Some(self.engine.revision());
            let r = if joined {
                self.engine.amend_batch("Macro", &c.commands)
            } else {
                self.engine.run_batch("Macro", &c.commands)
            };
            if let Err(e) = r {
                self.status = e.to_string();
            }
            self.macros.applied = Some(self.engine.revision());
            self.raster.borrow_mut().invalidate();
        }
        if let Some(page) = c.page {
            if self.doc().pages.iter().any(|p| p.id == page) {
                self.page = page;
            }
        }
        self.selection = c
            .selection
            .into_iter()
            .filter(|id| self.doc().find_shape(*id).is_some())
            .collect();
        self.scripts_output.extend(c.output);
        if let Some(s) = c.status {
            self.status = s;
        }
    }

    /// Answer the message or input box the macro waits for.
    pub fn answer_macro(&mut self, input: Input) {
        self.macros.prompt = None;
        if let Some(run) = &self.macros.run {
            let _ = run.input.send(input);
        }
    }

    /// Something happened on a macro's form: the macro handles it.
    pub fn macro_form_event(
        &mut self,
        form: u32,
        control: &str,
        event: &str,
        args: Vec<serde_json::Value>,
    ) {
        let snap = Snapshot::of(self);
        if let Some(run) = &self.macros.run {
            let _ = run.input.send(Input::Event(
                FormEvent {
                    form,
                    control: control.to_string(),
                    event: event.to_string(),
                    args,
                },
                Box::new(snap),
            ));
        }
    }

    /// Each frame: what the running macro asks for; when it ends, its last
    /// changes and its error. Whether it still runs.
    pub fn macro_tick(&mut self) -> bool {
        let Some(run) = &self.macros.run else {
            return false;
        };
        let mut reqs = Vec::new();
        while let Ok(r) = run.requests.try_recv() {
            reqs.push(r);
        }
        let finished = run.is_finished();
        for r in reqs {
            match r {
                Request::Flush(c) => self.apply_macro_changes(c),
                Request::MsgBox {
                    prompt,
                    buttons,
                    title,
                } => {
                    self.macros.prompt = Some(Prompt::MsgBox {
                        prompt,
                        buttons,
                        title,
                    })
                }
                Request::InputBox {
                    prompt,
                    title,
                    default,
                } => {
                    self.macros.prompt = Some(Prompt::InputBox {
                        prompt,
                        title,
                        text: default,
                    })
                }
            }
        }
        if !finished {
            return true;
        }
        let Some(run) = self.macros.run.take() else {
            return false;
        };
        let name = run.name.clone();
        let done = run.finish();
        self.apply_macro_changes(done.changes);
        self.macros.prompt = None;
        if done.stopped {
            self.status = trf("macros.stopped", &[("name", &name)]);
        } else if let Some(e) = done.error {
            let project = self
                .macros
                .projects
                .iter()
                .position(|p| name.starts_with(&format!("{}.", p.name())))
                .unwrap_or(usize::MAX);
            if name == "Scripts" {
                self.scripts_output.push(format!(
                    "Error: Run-time error '{}': {} ({}, line {})",
                    e.number, e.description, e.procedure, e.line
                ));
            } else {
                self.macros.error = Some((e, project));
            }
        } else if name != "Scripts" {
            self.status = trf("macros.done", &[("name", &name)]);
        }
        false
    }

    /// Tools > Macros > Run Macro (Alt+Shift+F8).
    pub fn open_run_macro(&mut self) {
        if self.macros_desktop_only() {
            return;
        }
        self.macros.ensure_loaded();
        self.macros.run_dialog = Some(RunDialog::default());
    }

    /// Tools > Macros > Macro Editor (Alt+F11): the chosen project's first
    /// module, else the first project's; with no project, a new one.
    pub fn open_macro_editor(&mut self) {
        if self.macros_desktop_only() {
            return;
        }
        self.macros.ensure_loaded();
        if self.macros.editor.is_some() {
            return;
        }
        if self.macros.projects.is_empty() {
            let project = vp::Project {
                name: "Macros".into(),
                modules: vec![vp::Module {
                    name: "Module1".into(),
                    kind: vp::ModuleKind::Standard,
                    source: "Attribute VB_Name = \"Module1\"\r\nSub Hello()\r\n    MsgBox \"Objects on the page: \" & ActivePage.Shapes.Count\r\nEnd Sub\r\n".into(),
                    has_source: true,
                    read_only: false,
                    private: false,
                }],
                code_page: 1252,
                ..Default::default()
            };
            if let Err(e) = self.macros.install(project, None) {
                self.status = trf("macros.save_failed", &[("e", &e)]);
                return;
            }
        }
        let (p, m) = match &self.macros.chosen {
            Some((p, m, _)) if *p < self.macros.projects.len() => {
                (*p, m.clone().unwrap_or_default())
            }
            _ => (0, String::new()),
        };
        self.edit_macro(p, &m, None);
    }

    /// Export a module (`.bas`, `.cls`, `.frm` with its `.frx`) or the
    /// whole project as a `.gms` file.
    pub fn export_macros(&mut self, project: usize, module: Option<&str>) {
        let Some(p) = self.macros.projects.get(project) else {
            return;
        };
        let files = match module.and_then(|m| p.project.module(m)) {
            Some(m) => vp::export_module(&p.project, m).map_err(|e| e.to_string()),
            None => vp::write_gms(&p.project)
                .map(|bytes| {
                    vec![vp::ExportedFile {
                        name: p
                            .file
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| format!("{}.gms", p.name())),
                        bytes,
                    }]
                })
                .map_err(|e| e.to_string()),
        };
        let files = match files {
            Ok(f) if !f.is_empty() => f,
            Ok(_) => return,
            Err(e) => {
                self.status = trf("macros.export_failed", &[("e", &e)]);
                return;
            }
        };
        let files: Vec<(String, Vec<u8>)> = files.into_iter().map(|f| (f.name, f.bytes)).collect();
        let first = files[0].0.clone();
        let ext = Path::new(&first)
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        // The save dialog, a download, or the system's save picker on
        // Android (#1402).
        crate::files::Dialog::new()
            .add_filter(ext.to_uppercase(), &[ext.as_str()])
            .set_file_name(first)
            .save_file(self, move |app, path| {
                app.write_exported_macros(&path, files)
            });
    }

    /// Write the exported files: the first to `path`, the others beside
    /// it (a form's `.frx` with the name chosen). A document the Android
    /// picker made has nothing beside it: the picker asks for each of the
    /// others in turn.
    fn write_exported_macros(&mut self, path: &Path, files: Vec<(String, Vec<u8>)>) {
        let mut files = std::collections::VecDeque::from(files);
        let Some((_, bytes)) = files.pop_front() else {
            return;
        };
        let shown = path.display().to_string();
        if let Err(e) = crate::files::write(path, &bytes) {
            self.status = trf("macros.export_failed", &[("e", &e.to_string())]);
            return;
        }
        // The form's .frx goes beside its .frm, with the name chosen.
        let named = |name: String| {
            if name.to_ascii_lowercase().ends_with(".frx") {
                path.with_extension("frx")
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(name)
            } else {
                name
            }
        };
        let rest: std::collections::VecDeque<(String, Vec<u8>)> =
            files.into_iter().map(|(n, b)| (named(n), b)).collect();
        if crate::files::is_document_path(path) {
            save_each_macro_file(self, rest, shown);
            return;
        }
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let mut result = Ok(());
        for (name, bytes) in rest {
            if result.is_ok() {
                result = crate::files::write(dir.join(&name), &bytes);
            }
        }
        self.status = match result {
            Ok(()) => trf("macros.exported", &[("f", &shown)]),
            Err(e) => trf("macros.export_failed", &[("e", &e.to_string())]),
        };
    }

    /// Open the macro editor at `module` of project `project`, at `line`.
    pub fn edit_macro(&mut self, project: usize, module: &str, line: Option<u32>) {
        let Some(p) = self.macros.projects.get(project) else {
            return;
        };
        let m = p
            .project
            .module(module)
            .or_else(|| p.project.modules.iter().find(|m| m.has_source));
        let (name, source) = m
            .map(|m| (m.name.clone(), m.source.clone()))
            .unwrap_or_default();
        self.macros.editor = Some(Editor {
            project,
            module: name,
            source,
            goto_line: line,
            ..Default::default()
        });
    }

    /// The editor's text goes back into its module, the project is saved,
    /// and the code is compiled to show its errors.
    pub fn save_macro_editor(&mut self) {
        let Some(ed) = &mut self.macros.editor else {
            return;
        };
        let Some(p) = self.macros.projects.get_mut(ed.project) else {
            return;
        };
        if let Some(m) = p
            .project
            .modules
            .iter_mut()
            .find(|m| m.name.eq_ignore_ascii_case(&ed.module))
        {
            m.source = ed.source.clone();
            m.has_source = true;
        }
        p.warnings = scan(&p.project);
        let saved = p.save();
        ed.dirty = false;
        ed.message = Some(match (saved, p.compile()) {
            (Err(e), _) => trf("macros.save_failed", &[("e", &e)]),
            (Ok(()), Err(e)) => {
                ed.goto_line = Some(e.line);
                trf(
                    "macros.compile_error_at",
                    &[
                        ("m", &e.module),
                        ("l", &e.line.to_string()),
                        ("e", &e.message),
                    ],
                )
            }
            (Ok(()), Ok(_)) => tr("macros.saved"),
        });
    }
}

/// The system's save picker for each of `rest` in turn (Android, #1402);
/// the status line names `shown` once all are written.
fn save_each_macro_file(
    app: &mut App,
    mut rest: std::collections::VecDeque<(String, Vec<u8>)>,
    shown: String,
) {
    let Some((name, bytes)) = rest.pop_front() else {
        app.status = trf("macros.exported", &[("f", &shown)]);
        return;
    };
    let ext = Path::new(&name)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    crate::files::Dialog::new()
        .add_filter(ext.to_uppercase(), &[ext.as_str()])
        .set_file_name(name)
        .save_file(app, move |app, path| {
            match crate::files::write(&path, &bytes) {
                Ok(()) => save_each_macro_file(app, rest, shown),
                Err(e) => app.status = trf("macros.export_failed", &[("e", &e.to_string())]),
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::vba_run::Input;

    fn project(name: &str, code: &str, forms: Vec<vp::Form>) -> vp::Project {
        let mut modules = vec![vp::Module {
            name: "Main".into(),
            kind: vp::ModuleKind::Standard,
            source: code.into(),
            has_source: true,
            read_only: false,
            private: false,
        }];
        for f in &forms {
            modules.push(vp::Module {
                name: f.name.clone(),
                kind: vp::ModuleKind::Form,
                source: String::new(),
                has_source: true,
                read_only: false,
                private: false,
            });
        }
        vp::Project {
            name: name.into(),
            modules,
            forms,
            code_page: 1252,
            ..Default::default()
        }
    }

    fn temp(name: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("tracedraw-macros-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// Run frames until the macro ends, answering with `answer` when it
    /// asks; at most a few seconds.
    fn drive(app: &mut App, mut answer: impl FnMut(&mut App)) {
        let t = std::time::Instant::now();
        while app.macro_tick() {
            answer(app);
            std::thread::sleep(std::time::Duration::from_millis(5));
            assert!(t.elapsed().as_secs() < 20, "the macro did not end");
        }
    }

    #[test]
    fn projects_are_kept_and_loaded_again() {
        let dir = temp("keep");
        let mut st = MacroState {
            dir: Some(dir.clone()),
            ..Default::default()
        };
        let p = project(
            "Tools",
            "Sub One()\r\nEnd Sub\r\nSub Two()\r\nEnd Sub\r\n",
            Vec::new(),
        );
        let i = st.install(p, None).unwrap();
        assert_eq!(st.projects[i].macros().len(), 2);
        st.rename(i, "Renamed Tools").unwrap();
        // A new session loads the copy.
        let mut again = MacroState {
            dir: Some(dir.clone()),
            ..Default::default()
        };
        again.ensure_loaded();
        assert_eq!(again.projects.len(), 1);
        assert_eq!(again.projects[0].name(), "Renamed Tools");
        assert_eq!(
            again.projects[0].macros(),
            vec![
                ("Main".to_string(), "One".to_string()),
                ("Main".to_string(), "Two".to_string())
            ]
        );
        again.remove(0);
        let mut third = MacroState {
            dir: Some(dir),
            ..Default::default()
        };
        third.ensure_loaded();
        assert!(third.projects.is_empty());
    }

    #[test]
    fn warnings_name_what_cannot_work_here() {
        let p = project(
            "W",
            "Private Declare Function FindWindow Lib \"user32\" (ByVal a As String) As Long\r\nSub Go()\r\n  Open \"x\" For Input As #1\r\n  Set o = CreateObject(\"Excel.Application\")\r\n  Set d = CreateObject(\"Scripting.Dictionary\")\r\nEnd Sub\r\n",
            Vec::new(),
        );
        let w = scan(&p).join("\n");
        assert!(w.contains("FindWindow"), "{w}");
        assert!(w.contains("Excel.Application"), "{w}");
        assert!(!w.contains("Scripting.Dictionary"), "{w}");
        assert_eq!(scan(&p).len(), 3, "{w}");
    }

    fn app_with(p: vp::Project) -> App {
        let mut app = App::headless();
        app.macros.dir = Some(temp(&p.name.clone()));
        app.macros.loaded = true;
        app.macros.install(p, None).unwrap();
        app
    }

    #[test]
    fn a_macro_asks_with_msgbox_and_inputbox_and_draws() {
        let code = "Sub Go()\r\n\
            Dim n As String, r As Long\r\n\
            ActiveDocument.Unit = cdrMillimeter\r\n\
            r = MsgBox(\"Draw?\", vbYesNo + vbQuestion, \"Ask\")\r\n\
            If r <> vbYes Then Exit Sub\r\n\
            n = InputBox(\"Name\", \"Ask\", \"box\")\r\n\
            Dim s As Shape\r\n\
            Set s = ActiveLayer.CreateRectangle2(10, 10, 40, 20)\r\n\
            s.Name = n\r\n\
            Debug.Print \"made \" & s.Name\r\n\
            End Sub\r\n";
        let mut app = app_with(project("Ask", code, Vec::new()));
        let before = app
            .doc()
            .all_layers()
            .map(|l| l.shapes.len())
            .sum::<usize>();
        app.run_macro(0, "Main", "Go");
        let mut asked = Vec::new();
        drive(&mut app, |app| match app.macros.prompt.take() {
            Some(Prompt::MsgBox {
                prompt,
                buttons,
                title,
            }) => {
                asked.push(format!("{title}: {prompt} {buttons}"));
                app.answer_macro(Input::Answer(6));
            }
            Some(Prompt::InputBox { prompt, text, .. }) => {
                asked.push(format!("{prompt} [{text}]"));
                app.answer_macro(Input::Text(Some("Frame".into())));
            }
            None => {}
        });
        assert!(app.macros.error.is_none(), "{:?}", app.macros.error);
        assert_eq!(
            asked,
            vec!["Ask: Draw? 36".to_string(), "Name [box]".to_string()]
        );
        let after = app
            .doc()
            .all_layers()
            .map(|l| l.shapes.len())
            .sum::<usize>();
        assert_eq!(after, before + 1);
        assert!(
            app.scripts_output.iter().any(|l| l == "made Frame"),
            "{:?}",
            app.scripts_output
        );
        // The whole run is one undo step.
        assert_eq!(app.engine.undo_label(), Some("Macro"));
        app.undo();
        assert_eq!(
            app.doc()
                .all_layers()
                .map(|l| l.shapes.len())
                .sum::<usize>(),
            before
        );
    }

    #[test]
    fn a_form_button_draws_and_reads_its_text_box() {
        let mut form = vp::Form {
            name: "Box".into(),
            caption: "Make a box".into(),
            width: 200.0,
            height: 100.0,
            enabled: true,
            ..Default::default()
        };
        let mut tb = vp::Control::new(vp::ControlKind::TextBox, "tbName");
        tb.value = "first".into();
        let ok = vp::Control::new(vp::ControlKind::CommandButton, "cbOK");
        form.controls = vec![tb, ok];
        let mut p = project(
            "Forms",
            "Sub Go()\r\nBox.Show\r\nDebug.Print \"closed\"\r\nEnd Sub\r\n",
            vec![form],
        );
        p.modules[1].source = "Private Sub UserForm_Initialize()\r\n\
            Me.Caption = \"Box maker\"\r\n\
            End Sub\r\n\
            Private Sub cbOK_Click()\r\n\
            Dim s As Shape\r\n\
            Set s = ActiveLayer.CreateEllipse2(1, 1, 0.5)\r\n\
            s.Name = tbName.Text\r\n\
            Unload Me\r\n\
            End Sub\r\n"
            .into();
        let mut app = app_with(p);
        app.run_macro(0, "Main", "Go");
        let mut stage = 0;
        drive(&mut app, |app| {
            let Some(run) = &app.macros.run else { return };
            let shown = run
                .forms
                .lock()
                .unwrap()
                .list
                .iter()
                .find(|f| f.visible)
                .map(|f| (f.id, f.caption.clone()));
            if let (0, Some((id, caption))) = (stage, shown) {
                assert_eq!(caption, "Box maker");
                // The user types and clicks OK.
                run.forms
                    .lock()
                    .unwrap()
                    .set_prop(id, "tbName", "Text", &serde_json::json!("Round"))
                    .unwrap();
                app.macro_form_event(id, "tbName", "Change", Vec::new());
                app.macro_form_event(id, "cbOK", "Click", Vec::new());
                stage = 1;
            }
        });
        assert!(app.macros.error.is_none(), "{:?}", app.macros.error);
        let names: Vec<String> = app
            .doc()
            .all_layers()
            .flat_map(|l| l.shapes.iter())
            .filter_map(|s| s.name.clone())
            .collect();
        assert!(names.contains(&"Round".to_string()), "{names:?}");
        assert!(app.scripts_output.iter().any(|l| l == "closed"));
    }

    #[test]
    fn errors_name_the_line_and_stop_ends_a_loop() {
        let code = "Sub Bad()\r\nDim a As Long\r\na = 1 / 0\r\nEnd Sub\r\nSub Forever()\r\nDim i As Long\r\nDo\r\ni = i + 1\r\nLoop\r\nEnd Sub\r\n";
        let mut app = app_with(project("Err", code, Vec::new()));
        app.run_macro(0, "Main", "Bad");
        drive(&mut app, |_| {});
        let (e, p) = app.macros.error.take().expect("an error");
        assert_eq!((e.number, e.line, p), (11, 3, 0), "{e:?}");
        assert_eq!(e.module, "Main");
        app.run_macro(0, "Main", "Forever");
        std::thread::sleep(std::time::Duration::from_millis(200));
        app.stop_macro();
        drive(&mut app, |_| {});
        assert!(app.macros.error.is_none());
        assert!(app.status.contains("Err.Main.Forever"), "{}", app.status);
    }

    #[test]
    fn a_project_written_as_gms_runs_after_loading() {
        let code = "Sub Count()\r\nDebug.Print ActivePage.Shapes.Count + 41\r\nEnd Sub\r\n";
        let bytes = vp::write_gms(&project("Saved", code, Vec::new())).unwrap();
        let dir = temp("gms-file");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("Saved.gms");
        std::fs::write(&file, &bytes).unwrap();
        let mut app = App::headless();
        app.macros.dir = Some(dir.join("kept"));
        app.macros.loaded = true;
        app.install_macro_files(&[file]);
        assert_eq!(app.macros.projects.len(), 1);
        app.run_macro(0, "Main", "Count");
        drive(&mut app, |_| {});
        assert!(app.macros.error.is_none(), "{:?}", app.macros.error);
        let n = app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum::<usize>()
            + 41;
        assert!(
            app.scripts_output.contains(&n.to_string()),
            "{:?}",
            app.scripts_output
        );
    }

    /// Local check of projects on disk (not run by default): every
    /// `.gms` in `TRACEDRAW_MACRO_FILES` (separated by ':') is compiled
    /// and listed; the macros named in `TRACEDRAW_MACRO_RUN`
    /// ("Module.Macro,...") run on a sample drawing, answering each box
    /// with its first button or default text and closing each form.
    #[test]
    #[ignore]
    fn projects_from_the_environment() {
        let Ok(files) = std::env::var("TRACEDRAW_MACRO_FILES") else {
            return;
        };
        let wanted: Vec<String> = std::env::var("TRACEDRAW_MACRO_RUN")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        for f in files.split(':') {
            let bytes = std::fs::read(f).unwrap();
            let project = vp::read_gms(&bytes).unwrap();
            let mut app = App::headless();
            app.macros.dir = Some(temp("env"));
            app.macros.loaded = true;
            crate::scripting::run(
                &mut app,
                "for (let i = 0; i < 3; i++) { const r = ActiveLayer.CreateRectangle(10 + i * 30, 10, 30 + i * 30, 30); r.Fill.ApplyUniformFill(Color.CMYK(0, 100, 100, 0)); r.CreateSelection ? 0 : 0; } ActiveLayer.CreateArtisticText(10, 80, 'Sample text');",
            )
            .unwrap();
            app.select_all();
            let i = app.macros.install(project, Some(&bytes)).unwrap();
            let p = &app.macros.projects[i];
            println!(
                "== {} ({} modules, {} forms)",
                p.name(),
                p.project.modules.len(),
                p.project.forms.len()
            );
            for w in &p.warnings {
                println!("   warning: {w}");
            }
            match p.compile() {
                Ok(_) => println!(
                    "   compiles; macros: {:?}",
                    p.macros()
                        .iter()
                        .map(|(m, n)| format!("{m}.{n}"))
                        .collect::<Vec<_>>()
                ),
                Err(e) => println!("   COMPILE ERROR {e}"),
            }
            let macros = p.macros();
            for (m, n) in macros {
                if !wanted.iter().any(|w| w == &format!("{m}.{n}")) {
                    continue;
                }
                app.select_all();
                app.run_macro(i, &m, &n);
                let mut shown = Vec::new();
                drive(&mut app, |app| {
                    match app.macros.prompt.take() {
                        Some(Prompt::MsgBox {
                            prompt, buttons, ..
                        }) => {
                            shown.push(format!(
                                "MsgBox({buttons}) {}",
                                prompt.lines().next().unwrap_or("")
                            ));
                            let first = match buttons & 0xF {
                                2 => 3,
                                3 | 4 => 6,
                                5 => 4,
                                _ => 1,
                            };
                            app.answer_macro(Input::Answer(first));
                        }
                        Some(Prompt::InputBox { text, .. }) => {
                            shown.push("InputBox".into());
                            app.answer_macro(Input::Text(Some(text)));
                        }
                        None => {}
                    }
                    let Some(run) = &app.macros.run else { return };
                    let visible: Vec<(u32, String)> = run
                        .forms
                        .lock()
                        .unwrap()
                        .list
                        .iter()
                        .filter(|f| f.visible)
                        .map(|f| (f.id, f.caption.clone()))
                        .collect();
                    for (id, caption) in visible {
                        shown.push(format!("form {caption}"));
                        app.macro_form_event(id, "", "QueryClose", vec![serde_json::json!(0)]);
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                });
                let err = app.macros.error.take();
                println!(
                    "   run {m}.{n}: {} {:?} out={:?}",
                    match &err {
                        None => "ok".to_string(),
                        Some((e, _)) => format!(
                            "ERROR {} {} ({}.{} line {})",
                            e.number, e.description, e.module, e.procedure, e.line
                        ),
                    },
                    shown,
                    app.scripts_output.iter().take(5).collect::<Vec<_>>()
                );
                app.scripts_output.clear();
            }
        }
    }
}
