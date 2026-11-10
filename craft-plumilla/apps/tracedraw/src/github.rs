//! Drawings kept in a GitHub repository (File > GitHub): open a `.tdraw`
//! from a repository, save it back as a commit with a message, list the
//! commits that changed it and open an earlier version. Everything goes
//! through GitHub's REST API (`api.github.com`) with the user's own token
//! (a fine-grained personal access token with Contents read and write);
//! public repositories open without one. See `docs/behavior/github.md`.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::net::{HttpRequest, HttpResult};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const API: &str = "https://api.github.com";
/// The largest file the contents API takes and gives (100 MB).
pub const MAX_BYTES: u64 = 100 * 1024 * 1024;

/// File > GitHub's last choices, kept with the preferences. The token is
/// kept only while `remember_token` is on; otherwise it lives in the
/// session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GithubPrefs {
    /// `owner/name`.
    pub repo: String,
    pub branch: String,
    pub path: String,
    pub token: String,
    pub remember_token: bool,
}

impl Default for GithubPrefs {
    fn default() -> Self {
        GithubPrefs {
            repo: String::new(),
            branch: "main".into(),
            path: String::new(),
            token: String::new(),
            remember_token: false,
        }
    }
}

/// Where a drawing open from GitHub lives, and the version of the file
/// it was opened or last saved at (GitHub refuses a save over a newer
/// one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub path: String,
    /// The file's blob id at that version.
    pub sha: String,
}

impl Remote {
    /// `owner/repo`.
    pub fn repo_name(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    /// The file's name, for the tab and the title.
    pub fn file_name(&self) -> String {
        self.path
            .rsplit('/')
            .next()
            .unwrap_or(&self.path)
            .to_string()
    }
}

/// `owner/name` from what the user typed: `owner/name`, a
/// `github.com/owner/name` address (with or without `https://`, `.git`
/// or a trailing slash).
pub fn parse_repo(s: &str) -> Option<(String, String)> {
    let s = s.trim().trim_end_matches('/');
    let s = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
        .unwrap_or(s);
    let s = s.strip_prefix("www.").unwrap_or(s);
    let s = s.strip_prefix("github.com/").unwrap_or(s);
    let s = s.strip_suffix(".git").unwrap_or(s);
    let mut parts = s.split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    let ok = |p: &str| {
        !p.is_empty()
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    (parts.next().is_none() && ok(owner) && ok(name)).then(|| (owner.into(), name.into()))
}

/// A path in the repository, cleaned: no leading or doubled slashes, no
/// `.` or `..` parts. `None` when nothing is left.
pub fn clean_path(p: &str) -> Option<String> {
    let parts: Vec<&str> = p
        .trim()
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    if parts.is_empty() || parts.contains(&"..") {
        return None;
    }
    Some(parts.join("/"))
}

/// Percent-encode a path or query value, keeping `/` when `keep_slash`.
fn encode(s: &str, keep_slash: bool) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        let plain = b.is_ascii_alphanumeric()
            || matches!(b, b'-' | b'_' | b'.' | b'~')
            || (keep_slash && b == b'/');
        if plain {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn headers(token: &str, accept: &str) -> Vec<(String, String)> {
    let mut h = vec![
        ("Accept".to_string(), accept.to_string()),
        ("X-GitHub-Api-Version".to_string(), "2022-11-28".to_string()),
    ];
    if !token.trim().is_empty() {
        h.push((
            "Authorization".to_string(),
            format!("Bearer {}", token.trim()),
        ));
    }
    // A browser sets its own; the desktop says who it is.
    if !crate::files::WEB {
        h.push((
            "User-Agent".to_string(),
            concat!("Craft Plumilla/", env!("CARGO_PKG_VERSION")).to_string(),
        ));
    }
    h
}

fn contents_url(owner: &str, repo: &str, path: &str) -> String {
    format!(
        "{API}/repos/{}/{}/contents/{}",
        encode(owner, false),
        encode(repo, false),
        encode(path, true)
    )
}

/// The file's description (its blob id, and its content when under 1 MB)
/// at `reference` (a branch or a commit).
pub fn meta_request(r: &Remote, reference: &str, token: &str) -> HttpRequest {
    HttpRequest {
        method: "GET",
        url: format!(
            "{}?ref={}",
            contents_url(&r.owner, &r.repo, &r.path),
            encode(reference, false)
        ),
        headers: headers(token, "application/vnd.github+json"),
        body: String::new(),
        bytes: Vec::new(),
    }
}

/// The file's bytes at `reference` (any size the API serves).
pub fn raw_request(r: &Remote, reference: &str, token: &str) -> HttpRequest {
    HttpRequest {
        method: "GET",
        url: format!(
            "{}?ref={}",
            contents_url(&r.owner, &r.repo, &r.path),
            encode(reference, false)
        ),
        headers: headers(token, "application/vnd.github.raw+json"),
        body: String::new(),
        bytes: Vec::new(),
    }
}

/// Save `content` as a commit on the remote's branch; over the version
/// `r.sha` names, or as a new file when it is empty.
pub fn put_request(r: &Remote, content: &[u8], message: &str, token: &str) -> HttpRequest {
    let mut body = json!({
        "message": message,
        "content": base64::engine::general_purpose::STANDARD.encode(content),
        "branch": r.branch,
    });
    if !r.sha.is_empty() {
        body["sha"] = json!(r.sha);
    }
    HttpRequest {
        method: "PUT",
        url: contents_url(&r.owner, &r.repo, &r.path),
        headers: headers(token, "application/vnd.github+json"),
        body: body.to_string(),
        bytes: Vec::new(),
    }
}

/// The commits of the remote's branch that changed its file, newest first.
pub fn history_request(r: &Remote, token: &str) -> HttpRequest {
    HttpRequest {
        method: "GET",
        url: format!(
            "{API}/repos/{}/{}/commits?path={}&sha={}&per_page=50",
            encode(&r.owner, false),
            encode(&r.repo, false),
            encode(&r.path, false),
            encode(&r.branch, false)
        ),
        headers: headers(token, "application/vnd.github+json"),
        body: String::new(),
        bytes: Vec::new(),
    }
}

/// What a file's description says: its blob id and, when GitHub sent it
/// along (files under 1 MB), its text.
#[derive(Debug, Clone, PartialEq)]
pub struct FileMeta {
    pub sha: String,
    pub content: Option<String>,
}

pub fn parse_meta(body: &str) -> Result<FileMeta, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if v.is_array() || v["type"].as_str().is_some_and(|t| t != "file") {
        return Err(tr("github.not_a_file"));
    }
    let sha = v["sha"].as_str().ok_or("no sha")?.to_string();
    let content = match (v["encoding"].as_str(), v["content"].as_str()) {
        (Some("base64"), Some(c)) if !c.is_empty() => {
            let clean: String = c.chars().filter(|c| !c.is_whitespace()).collect();
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(clean)
                .map_err(|e| e.to_string())?;
            Some(String::from_utf8(bytes).map_err(|_| tr("github.not_text"))?)
        }
        _ => None,
    };
    Ok(FileMeta { sha, content })
}

/// The new blob id of the saved file and the commit's address.
pub fn parse_put(body: &str) -> Result<(String, String), String> {
    let v: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let sha = v["content"]["sha"].as_str().ok_or("no sha")?.to_string();
    let url = v["commit"]["html_url"].as_str().unwrap_or("").to_string();
    Ok((sha, url))
}

/// One commit that changed the file.
#[derive(Debug, Clone, PartialEq)]
pub struct Commit {
    pub sha: String,
    /// The message's first line.
    pub message: String,
    pub author: String,
    /// As GitHub writes it (ISO 8601, UTC).
    pub date: String,
}

pub fn parse_commits(body: &str) -> Result<Vec<Commit>, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let list = v.as_array().ok_or("not a list")?;
    Ok(list
        .iter()
        .filter_map(|c| {
            Some(Commit {
                sha: c["sha"].as_str()?.to_string(),
                message: c["commit"]["message"]
                    .as_str()
                    .unwrap_or("")
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string(),
                author: c["commit"]["author"]["name"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                date: c["commit"]["author"]["date"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect())
}

/// Why GitHub said no, in words for the status line.
pub fn error_text(status: u16, body: &str) -> String {
    let message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["message"].as_str().map(str::to_string))
        .unwrap_or_default();
    match status {
        401 => tr("github.bad_token"),
        403 if message.to_ascii_lowercase().contains("rate limit") => tr("github.rate_limit"),
        403 => tr("github.forbidden"),
        404 => tr("github.not_found"),
        409 => tr("github.conflict"),
        // A save over an existing file without its version.
        422 if message.contains("sha") => tr("github.exists"),
        _ => format!("{status} {message}").trim().to_string(),
    }
}

/// A request on its way and what its answer is for.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Open the file at the branch: its description first.
    OpenMeta(Remote),
    /// Its bytes, too large to come with the description.
    OpenRaw(Remote),
    /// Open the file as it was at a commit (a copy, not tied to GitHub).
    VersionMeta(Remote, String),
    VersionRaw(Remote, String),
    Save(Remote),
    History(Remote),
}

/// File > GitHub's state in the session.
#[derive(Default)]
pub struct GithubSession {
    /// The token in use (the remembered one, or the one typed this
    /// session).
    pub token: String,
    pending: Option<(Step, std::sync::mpsc::Receiver<HttpResult>)>,
    /// The last request sent (what the tests look at).
    pub last_request: Option<HttpRequest>,
    /// History's list, for the remote it was asked for.
    pub history: Option<(Remote, Vec<Commit>)>,
    /// A line for the dialogs: what is going on or what went wrong.
    pub status: String,
}

impl GithubSession {
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }
}

/// The form of File > GitHub > Open and Save.
#[derive(Clone, PartialEq, Default)]
pub struct GithubForm {
    pub repo: String,
    pub branch: String,
    pub path: String,
    pub token: String,
    pub remember_token: bool,
    /// Save only: the commit message.
    pub message: String,
}

impl std::fmt::Debug for GithubForm {
    // The token stays out of any log.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GithubForm")
            .field("repo", &self.repo)
            .field("branch", &self.branch)
            .field("path", &self.path)
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}

impl GithubForm {
    /// The remote the form names (no version yet), or why not.
    pub fn remote(&self) -> Result<Remote, String> {
        let (owner, repo) = parse_repo(&self.repo).ok_or_else(|| tr("github.bad_repo"))?;
        let path = clean_path(&self.path).ok_or_else(|| tr("github.bad_path"))?;
        let branch = self.branch.trim();
        Ok(Remote {
            owner,
            repo,
            branch: if branch.is_empty() { "main" } else { branch }.to_string(),
            path,
            sha: String::new(),
        })
    }
}

impl App {
    /// The form, filled from the last choices (and the open drawing's
    /// remote, when it has one).
    pub fn github_form(&self, saving: bool) -> GithubForm {
        let p = &self.settings.github;
        let mut f = GithubForm {
            repo: p.repo.clone(),
            branch: p.branch.clone(),
            path: p.path.clone(),
            token: self.github.token.clone(),
            remember_token: p.remember_token,
            message: String::new(),
        };
        if let Some(r) = &self.remote {
            f.repo = r.repo_name();
            f.branch = r.branch.clone();
            f.path = r.path.clone();
        } else if saving && f.path.trim().is_empty() {
            f.path = format!("{}.tdraw", self.document_title().trim_end_matches('*'));
        }
        if saving {
            f.message = trf(
                "github.default_message",
                &[("name", f.path.rsplit('/').next().unwrap_or(&f.path))],
            );
        }
        f
    }

    /// Keep the form's choices (and the token, as the user asked).
    fn github_remember(&mut self, f: &GithubForm) {
        self.github.token = f.token.trim().to_string();
        let p = &mut self.settings.github;
        p.repo = f.repo.trim().to_string();
        p.branch = f.branch.trim().to_string();
        p.path = f.path.trim().to_string();
        p.remember_token = f.remember_token;
        p.token = if f.remember_token {
            self.github.token.clone()
        } else {
            String::new()
        };
        self.settings.save();
    }

    fn github_send(&mut self, step: Step, req: HttpRequest) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.github.last_request = Some(req.clone());
        self.github.status = tr("github.working");
        // Tests answer by hand (`github_answer`).
        if !cfg!(test) {
            crate::net::spawn(req, MAX_BYTES + 1024 * 1024, tx);
        } else {
            drop(tx);
        }
        self.github.pending = Some((step, rx));
    }

    /// File > GitHub > Open: fetch the file the form names.
    pub fn github_open(&mut self, f: &GithubForm) -> Result<(), String> {
        let r = f.remote()?;
        self.github_remember(f);
        let req = meta_request(&r, &r.branch, &self.github.token);
        self.github_send(Step::OpenMeta(r), req);
        Ok(())
    }

    /// File > GitHub > Save (and Save on a drawing from GitHub): commit the
    /// drawing to the file the form names, over the version it was opened
    /// at when it is that file.
    pub fn github_save(&mut self, f: &GithubForm) -> Result<(), String> {
        let mut r = f.remote()?;
        let message = f.message.trim();
        if message.is_empty() {
            return Err(tr("github.need_message"));
        }
        if let Some(cur) = &self.remote {
            if cur.owner == r.owner
                && cur.repo == r.repo
                && cur.branch == r.branch
                && cur.path == r.path
            {
                r.sha = cur.sha.clone();
            }
        }
        self.github_remember(f);
        self.flush_tracked_time();
        let bytes = tracedraw_text::fonts()
            .with_carried_fonts(self.engine.document())
            .to_json()
            .map_err(|e| e.to_string())?
            .into_bytes();
        if bytes.len() as u64 > MAX_BYTES {
            return Err(tr("github.too_large"));
        }
        let req = put_request(&r, &bytes, message, &self.github.token);
        self.github_send(Step::Save(r), req);
        Ok(())
    }

    /// File > GitHub > History: the commits that changed the drawing's file.
    pub fn github_history(&mut self) {
        let Some(r) = self.remote.clone() else {
            return;
        };
        let req = history_request(&r, &self.github.token);
        self.github_send(Step::History(r), req);
    }

    /// History's Open: the file as it was at `commit`, as a new drawing.
    pub fn github_open_version(&mut self, r: &Remote, commit: &str) {
        let req = meta_request(r, commit, &self.github.token);
        self.github_send(Step::VersionMeta(r.clone(), commit.to_string()), req);
    }

    /// Each frame: take a finished answer. Whether a request is still out.
    pub fn poll_github(&mut self) -> bool {
        let Some((_, rx)) = &self.github.pending else {
            return false;
        };
        match rx.try_recv() {
            Ok(result) => {
                if let Some((step, _)) = self.github.pending.take() {
                    self.github_answer(step, result);
                }
                self.github.pending.is_some()
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => true,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => !cfg!(test),
        }
    }

    /// The step's answer.
    pub fn github_answer(&mut self, step: Step, result: HttpResult) {
        self.github.pending = None;
        let resp = match result {
            Ok(r) => r,
            Err(e) => return self.github_failed(e),
        };
        if !(200..300).contains(&resp.status) {
            return self.github_failed(error_text(resp.status, &resp.body));
        }
        match step {
            Step::OpenMeta(mut r) => match parse_meta(&resp.body) {
                Ok(meta) => {
                    r.sha = meta.sha;
                    match meta.content {
                        Some(text) => self.github_opened(r, None, &text),
                        None => {
                            let req = raw_request(&r, &r.branch, &self.github.token);
                            self.github_send(Step::OpenRaw(r), req);
                        }
                    }
                }
                Err(e) => self.github_failed(e),
            },
            Step::OpenRaw(r) => self.github_opened(r, None, &resp.body),
            Step::VersionMeta(r, commit) => match parse_meta(&resp.body) {
                Ok(meta) => match meta.content {
                    Some(text) => self.github_opened(r, Some(commit), &text),
                    None => {
                        let req = raw_request(&r, &commit, &self.github.token);
                        self.github_send(Step::VersionRaw(r, commit), req);
                    }
                },
                Err(e) => self.github_failed(e),
            },
            Step::VersionRaw(r, commit) => self.github_opened(r, Some(commit), &resp.body),
            Step::Save(mut r) => match parse_put(&resp.body) {
                Ok((sha, _url)) => {
                    r.sha = sha;
                    let name = r.file_name();
                    let repo = r.repo_name();
                    self.remote = Some(r);
                    self.engine.mark_saved();
                    self.github.status.clear();
                    self.status = trf("github.saved", &[("name", &name), ("repo", &repo)]);
                    if matches!(self.dialog, crate::ui::dialogs::Dialog::GithubSave(_)) {
                        self.dialog = crate::ui::dialogs::Dialog::None;
                    }
                }
                Err(e) => self.github_failed(e),
            },
            Step::History(r) => match parse_commits(&resp.body) {
                Ok(list) => {
                    self.github.status.clear();
                    self.github.history = Some((r, list));
                }
                Err(e) => self.github_failed(e),
            },
        }
    }

    fn github_failed(&mut self, e: String) {
        self.github.status = e.clone();
        self.status = trf("github.failed", &[("e", &e)]);
    }

    /// A file came: open it as a drawing (tied to GitHub unless it is an
    /// earlier version).
    fn github_opened(&mut self, r: Remote, commit: Option<String>, text: &str) {
        match tracedraw_core::Document::from_json(text) {
            Ok(mut doc) => {
                let name = r.file_name();
                let stem = name.strip_suffix(".tdraw").unwrap_or(&name).to_string();
                doc.title = match &commit {
                    Some(c) => format!("{stem} @ {}", &c[..c.len().min(7)]),
                    None => stem,
                };
                self.open_document(doc, None);
                let repo = r.repo_name();
                self.remote = commit.is_none().then_some(r);
                self.github.status.clear();
                self.status = trf("github.opened", &[("name", &name), ("repo", &repo)]);
                if matches!(
                    self.dialog,
                    crate::ui::dialogs::Dialog::GithubOpen(_)
                        | crate::ui::dialogs::Dialog::GithubHistory
                ) {
                    self.dialog = crate::ui::dialogs::Dialog::None;
                }
            }
            Err(_) => self.github_failed(tr("github.not_a_drawing")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::HttpResponse;

    fn remote() -> Remote {
        Remote {
            owner: "me".into(),
            repo: "art".into(),
            branch: "main".into(),
            path: "logos/My Logo.tdraw".into(),
            sha: String::new(),
        }
    }

    fn ok(body: String) -> HttpResult {
        Ok(HttpResponse { status: 200, body })
    }

    #[test]
    fn repositories_and_paths_are_read_as_typed() {
        let want = Some(("me".to_string(), "art".to_string()));
        for s in [
            "me/art",
            " me/art ",
            "https://github.com/me/art",
            "github.com/me/art/",
            "https://www.github.com/me/art.git",
        ] {
            assert_eq!(parse_repo(s), want, "{s}");
        }
        for s in ["me", "me/art/x", "/art", "me/a rt", ""] {
            assert_eq!(parse_repo(s), None, "{s}");
        }
        assert_eq!(
            clean_path("/a//b/./c.tdraw").as_deref(),
            Some("a/b/c.tdraw")
        );
        assert_eq!(clean_path("../x"), None);
        assert_eq!(clean_path(" / "), None);
    }

    #[test]
    fn requests_name_the_file_and_carry_the_token() {
        let r = remote();
        let m = meta_request(&r, "main", "tok");
        assert_eq!(m.method, "GET");
        assert_eq!(
            m.url,
            "https://api.github.com/repos/me/art/contents/logos/My%20Logo.tdraw?ref=main"
        );
        assert!(m
            .headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer tok"));
        // No token, no Authorization (public repositories).
        assert!(!meta_request(&r, "main", " ")
            .headers
            .iter()
            .any(|(k, _)| k == "Authorization"));
        let raw = raw_request(&r, "abc", "");
        assert!(raw
            .headers
            .iter()
            .any(|(k, v)| k == "Accept" && v == "application/vnd.github.raw+json"));
        assert!(raw.url.ends_with("?ref=abc"));
        // A save: the content in base64, over the version when known.
        let mut over = r.clone();
        over.sha = "blob1".into();
        let put = put_request(&over, b"{}", "Update", "tok");
        assert_eq!(put.method, "PUT");
        let v: Value = serde_json::from_str(&put.body).expect("json");
        assert_eq!(v["content"], "e30=");
        assert_eq!(v["sha"], "blob1");
        assert_eq!(v["branch"], "main");
        assert_eq!(v["message"], "Update");
        let new: Value =
            serde_json::from_str(&put_request(&r, b"{}", "Add", "").body).expect("json");
        assert!(new.get("sha").is_none());
        let h = history_request(&r, "");
        assert!(h
            .url
            .contains("commits?path=logos%2FMy%20Logo.tdraw&sha=main"));
    }

    #[test]
    fn answers_are_read() {
        let meta =
            parse_meta(r#"{"type":"file","sha":"s1","encoding":"base64","content":"e30=\n"}"#)
                .expect("meta");
        assert_eq!(
            meta,
            FileMeta {
                sha: "s1".into(),
                content: Some("{}".into())
            }
        );
        // Over 1 MB: no content, the bytes come by a second request.
        let big = parse_meta(r#"{"type":"file","sha":"s2","encoding":"none","content":""}"#)
            .expect("meta");
        assert_eq!(big.content, None);
        assert!(parse_meta("[]").is_err());
        assert!(parse_meta(r#"{"type":"dir","sha":"x"}"#).is_err());
        assert_eq!(
            parse_put(r#"{"content":{"sha":"s3"},"commit":{"html_url":"u"}}"#),
            Ok(("s3".into(), "u".into()))
        );
        let commits = parse_commits(
            r#"[{"sha":"c1","commit":{"message":"First\n\nmore","author":{"name":"Ana","date":"2026-10-14T10:00:00Z"}}}]"#,
        )
        .expect("commits");
        assert_eq!(commits[0].message, "First");
        assert_eq!(commits[0].author, "Ana");
        assert!(error_text(409, "{}").contains(&tr("github.conflict")));
        assert_eq!(error_text(404, "{}"), tr("github.not_found"));
        assert_eq!(error_text(500, r#"{"message":"boom"}"#), "500 boom");
    }

    fn drawing_json() -> String {
        let doc = tracedraw_core::Document::new("x", tracedraw_core::Size::new(100.0, 50.0));
        doc.to_json().expect("json")
    }

    fn form(path: &str) -> GithubForm {
        GithubForm {
            repo: "me/art".into(),
            branch: "main".into(),
            path: path.into(),
            token: "tok".into(),
            remember_token: false,
            message: "Update".into(),
        }
    }

    #[test]
    fn open_edit_save_and_conflict() {
        let mut app = App::headless();
        let docs = app.docs.len();
        app.github_open(&form("a.tdraw")).expect("form");
        assert!(app.github.busy());
        // The token is kept for the session only.
        assert_eq!(app.github.token, "tok");
        assert!(app.settings.github.token.is_empty());
        let content = base64::engine::general_purpose::STANDARD.encode(drawing_json());
        let (step, _) = app.github.pending.take().expect("pending");
        app.github_answer(
            step,
            ok(format!(
                r#"{{"type":"file","sha":"v1","encoding":"base64","content":"{content}"}}"#
            )),
        );
        assert_eq!(app.docs.len(), docs + 1);
        let r = app.remote.clone().expect("remote");
        assert_eq!((r.sha.as_str(), r.path.as_str()), ("v1", "a.tdraw"));
        assert_eq!(app.doc().title, "a");
        // Save: over v1; the answer gives v2.
        app.github_save(&form("a.tdraw")).expect("save");
        let put: Value = serde_json::from_str(&app.github.last_request.clone().expect("sent").body)
            .expect("json");
        assert_eq!(put["sha"], "v1");
        let (step, _) = app.github.pending.take().expect("pending");
        app.github_answer(
            step,
            ok(r#"{"content":{"sha":"v2"},"commit":{"html_url":"u"}}"#.into()),
        );
        assert_eq!(app.remote.as_ref().map(|r| r.sha.as_str()), Some("v2"));
        assert!(!app.engine.is_dirty());
        // Someone else saved meanwhile: GitHub says 409, the drawing stays
        // tied to v2 and the reason is shown.
        app.github_save(&form("a.tdraw")).expect("save");
        let (step, _) = app.github.pending.take().expect("pending");
        app.github_answer(
            step,
            Ok(HttpResponse {
                status: 409,
                body: "{}".into(),
            }),
        );
        assert_eq!(app.remote.as_ref().map(|r| r.sha.as_str()), Some("v2"));
        assert!(app.github.status.contains(&tr("github.conflict")));
        // No message, no commit.
        let mut silent = form("a.tdraw");
        silent.message = " ".into();
        assert!(app.github_save(&silent).is_err());
    }

    #[test]
    fn large_files_and_earlier_versions() {
        let mut app = App::headless();
        app.github_open(&form("big.tdraw")).expect("form");
        let (step, _) = app.github.pending.take().expect("pending");
        app.github_answer(
            step,
            ok(r#"{"type":"file","sha":"b1","encoding":"none","content":""}"#.into()),
        );
        // A second request for the bytes.
        let raw = app.github.last_request.clone().expect("raw");
        assert!(raw
            .headers
            .iter()
            .any(|(_, v)| v == "application/vnd.github.raw+json"));
        let (step, _) = app.github.pending.take().expect("pending");
        app.github_answer(step, ok(drawing_json()));
        assert_eq!(app.remote.as_ref().map(|r| r.sha.as_str()), Some("b1"));
        // History, then an earlier version: a copy not tied to GitHub.
        app.github_history();
        let (step, _) = app.github.pending.take().expect("pending");
        app.github_answer(
            step,
            ok(r#"[{"sha":"c0ffee1234","commit":{"message":"Old","author":{"name":"A","date":"d"}}}]"#.into()),
        );
        let (r, list) = app.github.history.clone().expect("history");
        app.github_open_version(&r, &list[0].sha);
        assert!(app
            .github
            .last_request
            .clone()
            .expect("sent")
            .url
            .ends_with("?ref=c0ffee1234"));
        let content = base64::engine::general_purpose::STANDARD.encode(drawing_json());
        let (step, _) = app.github.pending.take().expect("pending");
        app.github_answer(
            step,
            ok(format!(
                r#"{{"type":"file","sha":"old","encoding":"base64","content":"{content}"}}"#
            )),
        );
        assert!(app.remote.is_none());
        assert_eq!(app.doc().title, "big @ c0ffee1");
        // Not a drawing.
        app.github_open(&form("notes.tdraw")).expect("form");
        let (step, _) = app.github.pending.take().expect("pending");
        let text = base64::engine::general_purpose::STANDARD.encode("hello");
        app.github_answer(
            step,
            ok(format!(
                r#"{{"type":"file","sha":"n","encoding":"base64","content":"{text}"}}"#
            )),
        );
        assert_eq!(app.github.status, tr("github.not_a_drawing"));
    }

    /// Against GitHub: the project's own sample drawing opens without a
    /// token. Run by hand (`cargo test -- --ignored`), it needs the network.
    #[test]
    #[ignore]
    fn live_open_of_a_public_drawing() {
        let r = Remote {
            owner: "tracedraw".into(),
            repo: "tracedraw".into(),
            branch: "main".into(),
            path: "sample.tdraw".into(),
            sha: String::new(),
        };
        let resp = crate::net::send(&meta_request(&r, "main", ""), MAX_BYTES).expect("answer");
        assert_eq!(resp.status, 200, "{}", resp.body);
        let meta = parse_meta(&resp.body).expect("meta");
        let text = match meta.content {
            Some(t) => t,
            None => {
                crate::net::send(&raw_request(&r, "main", ""), MAX_BYTES)
                    .expect("raw")
                    .body
            }
        };
        tracedraw_core::Document::from_json(&text).expect("a drawing");
        let resp = crate::net::send(&history_request(&r, ""), MAX_BYTES).expect("answer");
        assert!(!parse_commits(&resp.body).expect("commits").is_empty());
    }
}
