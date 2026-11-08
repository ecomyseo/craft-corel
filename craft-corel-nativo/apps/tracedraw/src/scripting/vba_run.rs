//! Running a compiled VBA macro on a worker thread, so it can stop for the
//! user: MsgBox and InputBox wait for the app's dialogs, a form shown with
//! `Show` waits for the user (its controls' events run the macro's code on
//! this thread), and Stop ends it. The changes it makes go to the app in
//! batches ([`Changes`]) at each of those pauses and at the end, so the
//! drawing shows them while a form or a message is up. See
//! docs/behavior/vba.md, "Running macros".

use super::{dispatch, vg, Host, HostInner, Trust, COMMON};
use crate::app::App;
use crate::macro_forms::{kind_name, Forms};
use boa_engine::{
    js_string, native_function::NativeFunction, Context, JsData, JsResult, JsString, JsValue,
    Source,
};
use serde_json::{json, Value};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracedraw_core::{Command, PageId, ShapeId};

/// What the app sees of a drawing when a macro starts or resumes: taken on
/// the app's thread and handed to the macro's.
pub struct Snapshot(HostInner);

impl Snapshot {
    pub fn of(app: &App) -> Snapshot {
        let mut inner = super::host_for(app, Trust::User);
        inner.vg = true;
        Snapshot(inner)
    }
}

/// Changes made by the macro since the last batch, for the app to apply.
#[derive(Debug, Default)]
pub struct Changes {
    pub commands: Vec<Command>,
    pub selection: Vec<ShapeId>,
    pub page: Option<PageId>,
    /// Lines printed (`Debug.Print`).
    pub output: Vec<String>,
    /// The status line it left (`Application.Status`).
    pub status: Option<String>,
}

/// What the macro asks the app for.
pub enum Request {
    /// Apply these changes now (the macro waits for the user next).
    Flush(Changes),
    MsgBox {
        prompt: String,
        buttons: i32,
        title: String,
    },
    InputBox {
        prompt: String,
        title: String,
        default: String,
    },
}

/// A control's (or the form's) event, from the app to the macro.
#[derive(Debug, Clone, PartialEq)]
pub struct FormEvent {
    pub form: u32,
    /// Empty for the form itself.
    pub control: String,
    pub event: String,
    pub args: Vec<Value>,
}

/// What the app answers.
pub enum Input {
    /// The button chosen in a MsgBox (vbOK = 1 ...).
    Answer(i32),
    /// The text typed in an InputBox; `None` for Cancel.
    Text(Option<String>),
    /// What the user did on a form, with the drawing as it is now.
    Event(FormEvent, Box<Snapshot>),
}

/// A run-time error: its number, description and where it happened.
#[derive(Debug, Clone, PartialEq)]
pub struct RunError {
    pub number: i32,
    pub description: String,
    pub module: String,
    pub procedure: String,
    pub line: u32,
}

impl RunError {
    /// Read "Run-time error 'n': description (Module.Proc, line l)".
    pub fn parse(text: &str) -> RunError {
        let t = text.trim();
        let t = t.strip_prefix("Error: ").unwrap_or(t);
        let mut e = RunError {
            number: 0,
            description: t.to_string(),
            module: String::new(),
            procedure: String::new(),
            line: 0,
        };
        if let Some(rest) = t.strip_prefix("Run-time error '") {
            if let Some((n, rest)) = rest.split_once("': ") {
                e.number = n.trim().parse().unwrap_or(0);
                let mut desc = rest.to_string();
                if let Some(open) = rest.rfind(" (") {
                    let loc = rest[open + 2..].trim_end_matches(')');
                    if let Some((mp, line)) = loc.split_once(", line ") {
                        if let Some((m, p)) = mp.split_once('.') {
                            e.module = m.to_string();
                            e.procedure = p.to_string();
                            e.line = line.trim().parse().unwrap_or(0);
                            desc = rest[..open].to_string();
                        }
                    }
                }
                e.description = desc;
            }
        }
        e
    }
}

/// How a run ended.
pub struct Finished {
    pub changes: Changes,
    pub error: Option<RunError>,
    pub stopped: bool,
}

/// A macro running on its thread.
pub struct MacroRun {
    pub requests: Receiver<Request>,
    pub input: Sender<Input>,
    pub stop: Arc<AtomicBool>,
    pub forms: Arc<Mutex<Forms>>,
    pub started: web_time::Instant,
    /// "Project.Module.Macro", for the status line and messages.
    pub name: String,
    handle: std::thread::JoinHandle<Finished>,
}

impl MacroRun {
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub fn finish(self) -> Finished {
        match self.handle.join() {
            Ok(f) => f,
            Err(_) => Finished {
                changes: Changes::default(),
                error: Some(RunError::parse("the macro's thread failed")),
                stopped: false,
            },
        }
    }
}

/// The macro thread's links to the app (context data of its engine).
struct RunState {
    requests: Sender<Request>,
    input: Receiver<Input>,
    stop: Arc<AtomicBool>,
    forms: Arc<Mutex<Forms>>,
    models: Vec<tracedraw_vba_project::Form>,
    /// Commands of the host already sent to the app.
    flushed: Cell<usize>,
    stopped: Cell<bool>,
    /// The macros folder (settings, paths).
    folder: Option<std::path::PathBuf>,
}

impl boa_engine::gc::Finalize for RunState {}
// The run state holds no garbage-collected values.
unsafe impl boa_engine::gc::Trace for RunState {
    boa_engine::gc::empty_trace!();
}
impl JsData for RunState {}

fn arg_str(args: &[JsValue], i: usize, ctx: &mut Context) -> JsResult<String> {
    match args.get(i) {
        Some(v) if !v.is_undefined() && !v.is_null() => {
            Ok(v.to_string(ctx)?.to_std_string_escaped())
        }
        _ => Ok(String::new()),
    }
}

fn arg_num(args: &[JsValue], i: usize, ctx: &mut Context) -> JsResult<f64> {
    match args.get(i) {
        Some(v) if !v.is_undefined() && !v.is_null() => v.to_number(ctx),
        _ => Ok(0.0),
    }
}

fn js_err(msg: impl Into<String>) -> boa_engine::JsError {
    boa_engine::JsNativeError::error()
        .with_message(msg.into())
        .into()
}

fn text(s: impl Into<String>) -> JsValue {
    JsValue::from(JsString::from(s.into().as_str()))
}

fn stop_value() -> JsValue {
    text("stop")
}

/// The changes since the last batch, taken from the host.
fn take_changes(ctx: &mut Context, all: bool) -> Changes {
    let Some(st) = ctx.get_data::<RunState>() else {
        return Changes::default();
    };
    let from = st.flushed.get();
    let Some(host) = ctx.get_data::<Host>() else {
        return Changes::default();
    };
    let mut h = host.0.borrow_mut();
    let commands: Vec<Command> = h
        .commands
        .get(from..)
        .map(<[Command]>::to_vec)
        .unwrap_or_default();
    if let Some(st) = ctx.get_data::<RunState>() {
        st.flushed.set(h.commands.len());
    }
    let _ = all;
    Changes {
        commands,
        selection: h.selection.clone(),
        page: Some(h.page),
        output: std::mem::take(&mut h.output),
        status: h.status.take(),
    }
}

/// Send the changes so far, before waiting for the user.
fn flush(ctx: &mut Context) {
    let changes = take_changes(ctx, false);
    if let Some(st) = ctx.get_data::<RunState>() {
        let _ = st.requests.send(Request::Flush(changes));
    }
}

/// The drawing as the app has it now replaces the host's (after an event:
/// what the macro sent was applied, and the user may have changed it).
fn resume_with(ctx: &mut Context, snap: Snapshot) {
    if let Some(host) = ctx.get_data::<Host>() {
        let mut h = host.0.borrow_mut();
        let fresh = snap.0;
        h.doc = fresh.doc;
        h.page = fresh.page;
        h.layer = fresh.layer;
        h.selection = fresh.selection;
        h.next_id = fresh.next_id;
        h.file = fresh.file;
        h.view = fresh.view;
        h.commands.clear();
    }
    if let Some(st) = ctx.get_data::<RunState>() {
        st.flushed.set(0);
    }
}

/// Wait for the app's answer; `None` when the macro is stopped.
fn wait_input(ctx: &Context) -> Option<Input> {
    let st = ctx.get_data::<RunState>()?;
    loop {
        if st.stop.load(Ordering::Relaxed) {
            st.stopped.set(true);
            return None;
        }
        match st.input.recv_timeout(Duration::from_millis(40)) {
            Ok(i) => return Some(i),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                st.stopped.set(true);
                return None;
            }
        }
    }
}

/// `__mb(prompt, buttons, title)`: the app's message box; the button.
fn native_msgbox(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let prompt = arg_str(args, 0, ctx)?;
    let buttons = arg_num(args, 1, ctx)? as i32;
    let title = arg_str(args, 2, ctx)?;
    flush(ctx);
    if let Some(st) = ctx.get_data::<RunState>() {
        let _ = st.requests.send(Request::MsgBox {
            prompt,
            buttons,
            title,
        });
    }
    loop {
        match wait_input(ctx) {
            None => return Ok(stop_value()),
            Some(Input::Answer(n)) => return Ok(JsValue::from(n)),
            Some(_) => continue,
        }
    }
}

/// `__ib(prompt, title, default)`: the app's input box; the text ("" for
/// Cancel).
fn native_inputbox(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let prompt = arg_str(args, 0, ctx)?;
    let title = arg_str(args, 1, ctx)?;
    let default = arg_str(args, 2, ctx)?;
    flush(ctx);
    if let Some(st) = ctx.get_data::<RunState>() {
        let _ = st.requests.send(Request::InputBox {
            prompt,
            title,
            default,
        });
    }
    loop {
        match wait_input(ctx) {
            None => return Ok(stop_value()),
            Some(Input::Text(t)) => return Ok(text(t.unwrap_or_default())),
            Some(_) => continue,
        }
    }
}

/// The settings macros keep with SaveSetting: `settings.json` in the
/// macros folder, by program, section and key (any case).
fn settings_path(ctx: &Context) -> Option<std::path::PathBuf> {
    ctx.get_data::<RunState>()?
        .folder
        .as_ref()
        .map(|d| d.join("settings.json"))
}

fn find_ci<'a>(m: &'a serde_json::Map<String, Value>, k: &str) -> Option<&'a String> {
    m.keys().find(|x| x.eq_ignore_ascii_case(k))
}

/// `__setting(op, app, section, key, value)`: "get" (the value or null),
/// "set", "delete" (key null: the section), "all" (pairs).
fn native_setting(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let op = arg_str(args, 0, ctx)?;
    let app = arg_str(args, 1, ctx)?;
    let section = arg_str(args, 2, ctx)?;
    let key = match args.get(3) {
        Some(v) if !v.is_null() && !v.is_undefined() => {
            Some(v.to_string(ctx)?.to_std_string_escaped())
        }
        _ => None,
    };
    let value = arg_str(args, 4, ctx).unwrap_or_default();
    let Some(path) = settings_path(ctx) else {
        return Ok(JsValue::null());
    };
    let mut all: serde_json::Map<String, Value> = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let akey = find_ci(&all, &app).cloned().unwrap_or(app.clone());
    let mut sections = all
        .get(&akey)
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let skey = find_ci(&sections, &section)
        .cloned()
        .unwrap_or(section.clone());
    let mut keys = sections
        .get(&skey)
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let found = key.as_deref().and_then(|k| find_ci(&keys, k).cloned());
    let out = match op.as_str() {
        "get" => {
            return Ok(match found.and_then(|k| keys.get(&k).cloned()) {
                Some(Value::String(s)) => text(s),
                Some(v) => text(v.to_string()),
                None => JsValue::null(),
            })
        }
        "all" => {
            let pairs: Vec<Value> = keys
                .iter()
                .map(|(k, v)| {
                    json!([
                        k,
                        v.as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| v.to_string())
                    ])
                })
                .collect();
            return Ok(text(Value::Array(pairs).to_string()));
        }
        "set" => {
            let k = found.unwrap_or(key.unwrap_or_default());
            keys.insert(k, Value::String(value));
            true
        }
        "delete" => match (key, found) {
            (None, _) => {
                sections.remove(&skey);
                all.insert(akey.clone(), Value::Object(sections.clone()));
                let _ = std::fs::write(&path, Value::Object(all).to_string());
                return Ok(JsValue::null());
            }
            (Some(_), Some(k)) => keys.remove(&k).is_some(),
            (Some(_), None) => false,
        },
        _ => false,
    };
    if out {
        sections.insert(skey, Value::Object(keys));
        all.insert(akey, Value::Object(sections));
        if let Some(d) = path.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(&path, Value::Object(all).to_string());
    }
    Ok(JsValue::null())
}

/// `__folder()`: the macros folder (with a separator at the end), for
/// the paths macros ask the host for.
fn native_folder(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let f = ctx
        .get_data::<RunState>()
        .and_then(|s| s.folder.clone())
        .map(|d| format!("{}{}", d.display(), std::path::MAIN_SEPARATOR))
        .unwrap_or_default();
    Ok(text(f))
}

/// `__stop()`: whether the user stopped the macro.
fn native_stop(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let Some(st) = ctx.get_data::<RunState>() else {
        return Ok(JsValue::from(false));
    };
    let s = st.stop.load(Ordering::Relaxed);
    if s {
        st.stopped.set(true);
    }
    Ok(JsValue::from(s))
}

fn with_forms<T>(
    ctx: &Context,
    f: impl FnOnce(&mut Forms, &[tracedraw_vba_project::Form]) -> T,
) -> Option<T> {
    let st = ctx.get_data::<RunState>()?;
    let mut forms = st.forms.lock().ok()?;
    Some(f(&mut forms, &st.models))
}

/// `__fnew(module)`: a form for that module; `{id, controls: [{name, kind}]}`.
fn native_form_new(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let module = arg_str(args, 0, ctx)?;
    let out = with_forms(ctx, |forms, models| {
        let model = models.iter().find(|m| m.name.eq_ignore_ascii_case(&module));
        let f = forms.create(&module, model);
        json!({
            "id": f.id,
            "controls": f.controls.iter().map(|c| json!({"name": c.name, "kind": kind_name(c.kind)})).collect::<Vec<_>>(),
        })
    })
    .ok_or_else(|| js_err("forms are not available here"))?;
    Ok(text(out.to_string()))
}

/// `__fget(id, control, property)`: the value as JSON.
fn native_form_get(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = arg_num(args, 0, ctx)? as u32;
    let ctl = arg_str(args, 1, ctx)?;
    let prop = arg_str(args, 2, ctx)?;
    let v = with_forms(ctx, |forms, _| forms.get_prop(id, &ctl, &prop))
        .ok_or_else(|| js_err("forms are not available here"))?
        .map_err(js_err)?;
    Ok(text(v.to_string()))
}

/// `__fset(id, control, property, json)`: `{changed, click, off}`.
fn native_form_set(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = arg_num(args, 0, ctx)? as u32;
    let ctl = arg_str(args, 1, ctx)?;
    let prop = arg_str(args, 2, ctx)?;
    let v: Value = serde_json::from_str(&arg_str(args, 3, ctx)?).unwrap_or(Value::Null);
    let o = with_forms(ctx, |forms, _| forms.set_prop(id, &ctl, &prop, &v))
        .ok_or_else(|| js_err("forms are not available here"))?
        .map_err(js_err)?;
    Ok(text(
        json!({"changed": o.changed, "click": o.click, "off": o.off}).to_string(),
    ))
}

/// `__fcall(id, control, method, jsonArgs)`: the result as JSON.
fn native_form_call(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = arg_num(args, 0, ctx)? as u32;
    let ctl = arg_str(args, 1, ctx)?;
    let method = arg_str(args, 2, ctx)?;
    let a: Vec<Value> = serde_json::from_str(&arg_str(args, 3, ctx)?).unwrap_or_default();
    let v = with_forms(ctx, |forms, _| forms.call(id, &ctl, &method, &a))
        .ok_or_else(|| js_err("forms are not available here"))?
        .map_err(js_err)?;
    Ok(text(v.to_string()))
}

/// `__fshow(id, modal)`, `__fhide(id)`, `__funload(id)`.
fn native_form_show(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = arg_num(args, 0, ctx)? as u32;
    let modal = args.get(1).is_some_and(|v| v.to_boolean());
    flush(ctx);
    with_forms(ctx, |forms, _| {
        if let Some(f) = forms.get_mut(id) {
            f.visible = true;
            f.modal = modal;
        }
    });
    Ok(JsValue::undefined())
}

fn native_form_hide(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = arg_num(args, 0, ctx)? as u32;
    with_forms(ctx, |forms, _| {
        if let Some(f) = forms.get_mut(id) {
            f.visible = false;
        }
    });
    Ok(JsValue::undefined())
}

fn native_form_unload(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = arg_num(args, 0, ctx)? as u32;
    with_forms(ctx, |forms, _| forms.remove(id));
    Ok(JsValue::undefined())
}

/// `__fwait(id)`: send the changes so far, then wait for the next event of
/// a form on screen. Returns the event as JSON; null when form `id` (any
/// form for 0) is no longer on screen; "stop" when the macro is stopped.
fn native_form_wait(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = arg_num(args, 0, ctx)? as u32;
    flush(ctx);
    loop {
        let open = with_forms(ctx, |forms, _| {
            if id == 0 {
                forms.any_visible()
            } else {
                forms.get(id).is_some_and(|f| f.visible)
            }
        })
        .unwrap_or(false);
        if !open {
            return Ok(JsValue::null());
        }
        let st = ctx
            .get_data::<RunState>()
            .ok_or_else(|| js_err("forms are not available here"))?;
        if st.stop.load(Ordering::Relaxed) {
            st.stopped.set(true);
            return Ok(stop_value());
        }
        match st.input.recv_timeout(Duration::from_millis(40)) {
            Ok(Input::Event(ev, snap)) => {
                resume_with(ctx, *snap);
                let e = json!({
                    "form": ev.form,
                    "control": ev.control,
                    "event": ev.event,
                    "args": ev.args,
                });
                return Ok(text(e.to_string()));
            }
            Ok(_) => continue,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                st.stopped.set(true);
                return Ok(stop_value());
            }
        }
    }
}

/// The host side of the VBA runtime: message boxes, forms and their
/// controls, and the check that ends a stopped macro.
pub(super) const GLUE: &str = include_str!("vba_glue.js");

/// Make every statement check, now and then, whether the user stopped the
/// macro (`__chk` throws `End`). The lines stay the same.
pub fn with_stop_checks(js: &str) -> String {
    js.replace("__f.l=", "__chk();__f.l=")
}

/// Start `entry` (JavaScript from `tracedraw_vba::entry_call`) of
/// `program` (compiled VBA) on a worker thread, with the project's form
/// designs. Desktop only.
pub fn start(
    app: &App,
    program: &str,
    entry: &str,
    models: Vec<tracedraw_vba_project::Form>,
    name: String,
) -> Result<MacroRun, String> {
    let (req_tx, req_rx) = channel();
    let (in_tx, in_rx) = channel();
    let stop = Arc::new(AtomicBool::new(false));
    let forms = Arc::new(Mutex::new(Forms::default()));
    let snap = Snapshot::of(app);
    let program = with_stop_checks(program);
    let entry = format!("{entry};\n__pump(0,true);\n");
    let (stop2, forms2) = (stop.clone(), forms.clone());
    let folder = if crate::files::WEB {
        None
    } else {
        app.macros.dir.clone().or_else(crate::macros::default_dir)
    };
    let handle = std::thread::Builder::new()
        .name("vba-macro".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            run_thread(
                snap, program, entry, models, req_tx, in_rx, stop2, forms2, folder,
            )
        })
        .map_err(|e| e.to_string())?;
    Ok(MacroRun {
        requests: req_rx,
        input: in_tx,
        stop,
        forms,
        started: web_time::Instant::now(),
        name,
        handle,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_thread(
    snap: Snapshot,
    program: String,
    entry: String,
    models: Vec<tracedraw_vba_project::Form>,
    requests: Sender<Request>,
    input: Receiver<Input>,
    stop: Arc<AtomicBool>,
    forms: Arc<Mutex<Forms>>,
    folder: Option<std::path::PathBuf>,
) -> Finished {
    let mut ctx = Context::default();
    ctx.insert_data(Host(RefCell::new(snap.0)));
    ctx.insert_data(RunState {
        requests,
        input,
        stop,
        forms,
        models,
        flushed: Cell::new(0),
        stopped: Cell::new(false),
        folder,
    });
    let result = (|| -> Result<(), String> {
        let natives: [(
            &str,
            usize,
            boa_engine::native_function::NativeFunctionPointer,
        ); 14] = [
            ("__setting", 5, native_setting),
            ("__folder", 0, native_folder),
            ("__td", 2, dispatch),
            ("__mb", 3, native_msgbox),
            ("__ib", 3, native_inputbox),
            ("__stop", 0, native_stop),
            ("__fnew", 1, native_form_new),
            ("__fget", 3, native_form_get),
            ("__fset", 4, native_form_set),
            ("__fcall", 4, native_form_call),
            ("__fshow", 2, native_form_show),
            ("__fhide", 1, native_form_hide),
            ("__funload", 1, native_form_unload),
            ("__fwait", 1, native_form_wait),
        ];
        for (name, n, f) in natives {
            ctx.register_global_callable(JsString::from(name), n, NativeFunction::from_fn_ptr(f))
                .map_err(|e| e.to_string())?;
        }
        let _ = js_string!("");
        for (what, src) in [
            ("common", COMMON),
            ("object model", vg::PRELUDE),
            ("VBA runtime", tracedraw_vba::RUNTIME_JS),
            ("forms", GLUE),
        ] {
            ctx.eval(Source::from_bytes(src))
                .map_err(|e| format!("{what}: {e}"))?;
        }
        ctx.eval(Source::from_bytes(&program))
            .map_err(|e| e.to_string())?;
        ctx.eval(Source::from_bytes(&entry))
            .map_err(|e| e.to_string())?;
        Ok(())
    })();
    let stopped = ctx
        .get_data::<RunState>()
        .is_some_and(|s| s.stopped.get() || s.stop.load(Ordering::Relaxed));
    // Forms still open when it ends (stopped, or an error) go away.
    if let Some(st) = ctx.get_data::<RunState>() {
        if let Ok(mut f) = st.forms.lock() {
            f.list.clear();
        }
    }
    let changes = take_changes(&mut ctx, true);
    Finished {
        changes,
        error: result
            .err()
            .filter(|_| !stopped)
            .map(|e| RunError::parse(&e)),
        stopped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_are_read_back() {
        let e = RunError::parse("Error: Run-time error '11': Division by zero (Tools.Go, line 7)");
        assert_eq!(e.number, 11);
        assert_eq!(e.description, "Division by zero");
        assert_eq!(
            (e.module.as_str(), e.procedure.as_str(), e.line),
            ("Tools", "Go", 7)
        );
        let e = RunError::parse("SyntaxError: bad");
        assert_eq!(e.number, 0);
        assert_eq!(e.description, "SyntaxError: bad");
    }
}
