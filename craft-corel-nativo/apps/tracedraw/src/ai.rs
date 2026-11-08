//! AI Drawing: a request in words becomes a program for the scripting
//! object model (`scripting.rs`), written by an AI service the user sets
//! up with their own key, and run as one undo step.
//!
//! There is one AI service (Tools > Options > AI > Service). Its text
//! model writes drawings; when the service can also make pictures, its
//! image model serves AI Generate and Remove Background (see
//! [`AiPrefs::image_prefs`]).
//!
//! Two kinds of service are spoken to: the Messages API and the chat
//! completions API that many services and local model servers offer. The
//! request carries a description of the object model, the user's words
//! and a description of the active page (`describe.rs`); the reply's code
//! block runs as an untrusted script (no file access, limits). A script
//! that fails is sent back once with its error for a corrected one.
//! Nothing is sent until the user asks; the key stays in memory unless the
//! user asks to remember it. See docs/behavior/ai-drawing.md.

use crate::app::App;
use crate::i18n::{tr, trf};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// The kinds of service AI Drawing talks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    /// The Messages API.
    #[default]
    Anthropic,
    /// The chat completions API, offered by many services and by local
    /// model servers.
    OpenAiCompatible,
}

impl Provider {
    pub const ALL: [Provider; 2] = [Provider::Anthropic, Provider::OpenAiCompatible];

    /// i18n key of the name shown in the list.
    pub fn key(self) -> &'static str {
        match self {
            Provider::Anthropic => "ai.provider_anthropic",
            Provider::OpenAiCompatible => "ai.provider_openai_compatible",
        }
    }

    /// The address requests go to when none is set.
    pub fn default_endpoint(self) -> &'static str {
        match self {
            Provider::Anthropic => "https://api.anthropic.com/v1/messages",
            Provider::OpenAiCompatible => "https://api.openai.com/v1/chat/completions",
        }
    }

    /// The model asked for when none is set (none for services whose
    /// models vary).
    pub fn default_model(self) -> &'static str {
        match self {
            Provider::Anthropic => "claude-sonnet-5-5",
            Provider::OpenAiCompatible => "",
        }
    }

    /// Requests need a key (local model servers often take none).
    pub fn needs_key(self) -> bool {
        matches!(self, Provider::Anthropic)
    }

    /// Models offered before the service's own list is fetched.
    pub fn preset_models(self) -> &'static [&'static str] {
        match self {
            Provider::Anthropic => &[
                "claude-sonnet-5-5",
                "claude-opus-5-5",
                "claude-haiku-5-5",
                "claude-fable-5-1",
            ],
            Provider::OpenAiCompatible => &[],
        }
    }
}

/// Ready-made connections: the i18n key of the name, the kind of service
/// and its address. The address stays editable after one is picked.
pub const PRESETS: [(&str, Provider, &str); 8] = [
    ("aipreset.anthropic", Provider::Anthropic, ""),
    ("aipreset.openai", Provider::OpenAiCompatible, ""),
    (
        "aipreset.gemini",
        Provider::OpenAiCompatible,
        "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
    ),
    (
        "aipreset.ollama",
        Provider::OpenAiCompatible,
        "http://localhost:11434/v1/chat/completions",
    ),
    (
        "aipreset.lm_studio",
        Provider::OpenAiCompatible,
        "http://localhost:1234/v1/chat/completions",
    ),
    (
        "aipreset.llama_cpp",
        Provider::OpenAiCompatible,
        "http://localhost:8080/v1/chat/completions",
    ),
    (
        "aipreset.vllm",
        Provider::OpenAiCompatible,
        "http://localhost:8000/v1/chat/completions",
    ),
    (
        "aipreset.opencode",
        Provider::OpenAiCompatible,
        "https://opencode.ai/inference/openai/v1/chat/completions",
    ),
];

/// The host of the OpenCode service (its Console Inference API, at
/// `/inference/<family>/...`; the Zen and Go models are served there),
/// which wants every request to name the session it belongs to.
const OPENCODE_HOST: &str = "https://opencode.ai/";

/// True when requests go to the OpenCode services.
fn is_opencode(prefs: &AiPrefs) -> bool {
    prefs
        .endpoint()
        .to_ascii_lowercase()
        .starts_with(OPENCODE_HOST)
}

/// One id for this run of the program, sent as the session of every
/// request to the OpenCode services so they are counted together.
fn session_id() -> &'static str {
    use std::hash::{BuildHasher, Hasher};
    static ID: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    ID.get_or_init(|| {
        let nanos = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(nanos);
        format!("tracedraw-{:016x}{:08x}", h.finish(), nanos as u32)
    })
}

/// The preset the connection matches, if any.
pub fn preset_of(prefs: &AiPrefs) -> Option<usize> {
    PRESETS.iter().position(|(_, p, e)| {
        *p == prefs.provider && prefs.endpoint.trim().trim_end_matches('/') == *e
    })
}

/// The presets that only write text (no pictures): the model servers on
/// this computer (Ollama, LM Studio, llama.cpp and vLLM) and OpenCode.
const CHAT_ONLY_PRESETS: [&str; 5] = [
    "aipreset.ollama",
    "aipreset.lm_studio",
    "aipreset.llama_cpp",
    "aipreset.vllm",
    "aipreset.opencode",
];

/// Google's API host: chat at its OpenAI-compatible address, pictures
/// at the Gemini API base (`ImageProvider::Gemini`'s usual address).
const GEMINI_HOST: &str = "https://generativelanguage.googleapis.com/";
/// The base address of OpenAI's API.
pub const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

/// What the service offers for pictures, besides drawings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageSupport {
    /// Drawings only (the Messages API, local chat-only servers).
    None,
    /// The OpenAI Images API at OpenAI's address, with the same key.
    OpenAi,
    /// Gemini's image API with the same key.
    Gemini,
    /// An OpenAI-compatible address of the user's: images at the same
    /// base, when an image model is given.
    Custom { base: String },
}

/// True for an address that is one of the chat-only local presets.
fn is_chat_only_preset(prefs: &AiPrefs) -> bool {
    preset_of(prefs)
        .and_then(|i| PRESETS.get(i))
        .is_some_and(|(k, _, _)| CHAT_ONLY_PRESETS.contains(k))
}

/// The base of an OpenAI-compatible address: without
/// `/chat/completions` and a trailing slash.
pub fn api_base(endpoint: &str) -> String {
    let e = endpoint.trim().trim_end_matches('/');
    e.strip_suffix("/chat/completions")
        .unwrap_or(e)
        .trim_end_matches('/')
        .to_string()
}

/// The connection, kept with the preferences. The key is kept here only
/// while `remember_key` is on; otherwise it lives in the session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiPrefs {
    pub provider: Provider,
    /// Empty: the provider's usual address.
    pub endpoint: String,
    /// Empty: the provider's usual model.
    pub model: String,
    pub key: String,
    pub remember_key: bool,
    /// The prompt bar above the palette shows while a service is set up.
    pub show_bar: bool,
    /// The model that makes pictures, when the service can (empty: the
    /// service's usual one; at an address of the user's, no pictures).
    pub image_model: String,
}

impl Default for AiPrefs {
    fn default() -> Self {
        AiPrefs {
            provider: Provider::default(),
            endpoint: String::new(),
            model: String::new(),
            key: String::new(),
            remember_key: false,
            show_bar: true,
            image_model: String::new(),
        }
    }
}

impl AiPrefs {
    /// The address requests go to.
    pub fn endpoint(&self) -> String {
        let e = self.endpoint.trim();
        if e.is_empty() {
            self.provider.default_endpoint().to_string()
        } else {
            e.to_string()
        }
    }

    /// The model asked for.
    pub fn model(&self) -> String {
        let m = self.model.trim();
        if m.is_empty() {
            self.provider.default_model().to_string()
        } else {
            m.to_string()
        }
    }

    /// The address of the service's list of models: `models` next to
    /// `messages` or `chat/completions`.
    pub fn models_endpoint(&self) -> String {
        let e = self.endpoint();
        let e = e.trim_end_matches('/');
        let base = e
            .strip_suffix("/chat/completions")
            .or_else(|| e.strip_suffix("/messages"))
            .unwrap_or(e);
        format!("{base}/models")
    }

    /// What the service offers for pictures.
    pub fn image_support(&self) -> ImageSupport {
        if self.provider == Provider::Anthropic {
            return ImageSupport::None;
        }
        let base = api_base(&self.endpoint());
        if base == OPENAI_API_BASE {
            ImageSupport::OpenAi
        } else if base.starts_with(GEMINI_HOST) {
            ImageSupport::Gemini
        } else if is_chat_only_preset(self) {
            ImageSupport::None
        } else {
            ImageSupport::Custom { base }
        }
    }

    /// The service can make pictures (the Image model field shows).
    pub fn can_make_pictures(&self) -> bool {
        self.image_support() != ImageSupport::None
    }

    /// The image calls of this service: kind, address and model (the key
    /// is the service's, passed with each request). None when the service
    /// makes drawings only, or is an address of the user's without an
    /// image model.
    pub fn image_prefs(&self) -> Option<crate::ai_image::ImagePrefs> {
        use crate::ai_image::{ImagePrefs, ImageProvider};
        let model = self.image_model.trim().to_string();
        let (provider, endpoint) = match self.image_support() {
            ImageSupport::None => return None,
            ImageSupport::OpenAi => (ImageProvider::OpenAi, String::new()),
            ImageSupport::Gemini => (ImageProvider::Gemini, String::new()),
            ImageSupport::Custom { base } => {
                if model.is_empty() {
                    return None;
                }
                (ImageProvider::OpenAi, base)
            }
        };
        Some(ImagePrefs {
            provider,
            endpoint,
            model,
            key: String::new(),
            remember_key: false,
        })
    }
}

/// The key a session starts with: the remembered one, if any.
pub fn remembered_key(prefs: &AiPrefs) -> String {
    if prefs.remember_key {
        prefs.key.clone()
    } else {
        String::new()
    }
}

pub use crate::net::{HttpRequest, HttpResponse, HttpResult};

/// Who says a message of the conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

impl Role {
    fn name(self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

/// An earlier request and the reply that drew it, sent along with the
/// next request so "make it bigger" means something.
#[derive(Debug, Clone, PartialEq)]
pub struct Turn {
    pub request: String,
    pub reply: String,
}

/// Earlier turns sent with a request.
pub const MAX_HISTORY: usize = 3;
/// Objects of the active page described to the service.
pub const MAX_OBJECTS: usize = 300;
/// Longest reply taken as a script, in bytes.
const MAX_SCRIPT: usize = 256 * 1024;
/// Tokens the Messages API may answer with.
const MAX_TOKENS: u32 = 8000;
/// The undo step of a drawing made by AI Drawing.
const UNDO_LABEL: &str = "AI Drawing";

/// Where a request stands.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AiStatus {
    #[default]
    Idle,
    /// Sent; waiting for the reply.
    Waiting,
    /// The first script failed; a corrected one was asked for.
    Retrying,
    /// Drawn, with this many changes.
    Done(usize),
    /// The service listed this many models.
    Listed(usize),
    /// Nothing drawn, and why.
    Failed(String),
}

/// A request on its way.
struct Pending {
    rx: std::sync::mpsc::Receiver<HttpResult>,
    job: Job,
}

/// What a request on its way is for.
enum Job {
    Draw {
        /// The messages sent, to continue the conversation on a retry.
        messages: Vec<(Role, String)>,
        /// The user's words, for the history.
        request: String,
        /// This is already the corrected attempt.
        retried: bool,
    },
    /// The service's list of models.
    Models,
}

/// AI Drawing's state for the session (the docker's fields and the
/// conversation).
#[derive(Default)]
pub struct AiSession {
    /// What to draw, in the user's words.
    pub request: String,
    /// The key in use (also in the preferences when remembered).
    pub key: String,
    pub status: AiStatus,
    /// The last script received.
    pub script: String,
    /// What the last script printed.
    pub output: Vec<String>,
    pub history: Vec<Turn>,
    /// The models the service listed (empty until asked).
    pub models: Vec<String>,
    pending: Option<Pending>,
    /// Tests: answers requests at once instead of the network.
    pub fake: Option<fn(&HttpRequest) -> HttpResult>,
    /// Tests: the requests sent, newest last.
    pub sent: Vec<HttpRequest>,
}

impl AiSession {
    /// A request is on its way.
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }
}

/// What the service is told about the job and the object model.
pub const SYSTEM_PROMPT: &str = r#"You are the drawing assistant of Craft Corel, a vector illustration editor. You draw by writing one JavaScript program for Craft Corel's object model. Reply with the whole program in a single ```javascript code block and nothing else.

Units and axes: lengths in millimetres; the origin is the bottom-left corner of the page; x grows to the right and y upwards. Angles in degrees, counter-clockwise. Text sizes in points.

Colours: Color.RGB(r, g, b) with 0 to 255; Color.CMYK(c, m, y, k) with 0 to 100; Color.Gray(v) with 0 black to 100 white.

Creating objects (each returns a Shape; new objects go on the active layer, in front of the others; they have no fill and a thin black outline, text is black without outline):
  ActiveLayer.CreateRectangle(x, y, width, height, cornerRadius)   x, y: bottom-left corner
  ActiveLayer.CreateEllipse(x, y, width, height)                    the bounding box from its bottom-left corner
  ActiveLayer.CreatePolygon(x, y, width, height, sides)
  ActiveLayer.CreateStar(x, y, width, height, points, sharpness)    sharpness 0 to 1
  ActiveLayer.CreateLineSegment(x1, y1, x2, y2)
  ActiveLayer.CreateCurve([[x, y], [x, y], ...], closed)            straight segments through the points
  ActiveLayer.CreatePath("M 10 10 C 20 40, 40 40, 50 10 Z")        SVG path data (M L H V C S Q T A Z) in page coordinates, y up
  ActiveLayer.CreateArtisticText(x, y, text, fontFamily, sizePt)    x, y: start of the baseline
  ActiveLayer.CreateParagraphText(x, y, width, height, text, fontFamily, sizePt)

Shape methods and properties:
  s.Fill.ApplyUniformFill(color)   s.Fill.ApplyFountainFill(fromColor, toColor, angle)   s.Fill.ApplyNoFill()
  s.Outline.SetProperties(widthMm, color)   s.Outline.SetNoOutline()
  s.Move(dx, dy)   s.SetPosition(x, y) (bottom-left of its bounding box)   s.SetSize(width, height)
  s.Rotate(angle) (about its centre)   s.Stretch(sx, sy)   s.Skew(angleX, angleY)
  s.Duplicate(dx, dy) returns the copy   s.Delete()   s.OrderToFront()   s.OrderToBack()
  s.SetOpacity(transparencyPercent) (0 opaque, 100 invisible)   s.AddDropShadow(dx, dy, opacityPercent)
  s.Name and s.Text (text objects) can be read and set; s.id, s.PositionX, s.PositionY, s.SizeWidth, s.SizeHeight, s.CenterX, s.CenterY can be read

Several objects: new ShapeRange([a.id, b.id]).Group() groups them and returns the group. ActivePage.Shapes.All and ActiveDocument.Selection are ShapeRanges with Count, Item(i) (from 1), forEach(f), Move(dx, dy), Delete(), AlignLeft(), AlignRight(), AlignTop(), AlignBottom(), AlignCentersHorizontally(), AlignCentersVertically().
Existing objects: new Shape(id) with an id from the description of the page; ActiveLayer.Shapes.FindShape(name).
Pages and layers: ActivePage.SizeWidth, ActivePage.SizeHeight, ActivePage.SetSize(width, height), ActivePage.CreateLayer(name) returns a layer with the same Create methods.
print(text) shows a short message to the user.

Rules: keep the drawing on the page unless asked otherwise, in proportion and tidy; use common font families such as Arial, Times New Roman or Georgia; change existing objects through their ids; delete only what the request asks to; files cannot be read or written."#;

/// The user's message: the request, then (for the newest request) the
/// page as it is.
pub fn user_message(request: &str, page: Option<&Value>) -> String {
    match page {
        Some(p) => format!("Request: {request}\n\nThe page as it is now (JSON):\n{p}"),
        None => format!("Request: {request}"),
    }
}

/// The message asking for a corrected script.
fn retry_message(error: &str) -> String {
    format!(
        "The program failed with this error: {error}\nNothing was drawn. Reply with the whole corrected program."
    )
}

/// The headers that carry the key (and, for the Messages API, its
/// version), with the JSON content type.
fn headers(prefs: &AiPrefs, key: &str) -> Vec<(String, String)> {
    let key = key.trim();
    let mut headers = vec![("content-type".to_string(), "application/json".to_string())];
    match prefs.provider {
        Provider::Anthropic => {
            headers.push(("x-api-key".to_string(), key.to_string()));
            headers.push(("anthropic-version".to_string(), "2023-06-01".to_string()));
            // The service answers a browser only when it is told the key
            // is the user's own, used from their browser.
            if crate::files::WEB {
                headers.push((
                    "anthropic-dangerous-direct-browser-access".to_string(),
                    "true".to_string(),
                ));
            }
        }
        Provider::OpenAiCompatible => {
            if !key.is_empty() {
                headers.push(("authorization".to_string(), format!("Bearer {key}")));
            }
            if is_opencode(prefs) {
                headers.push(("x-opencode-session".to_string(), session_id().to_string()));
            }
        }
    }
    headers
}

/// The HTTP request for `messages` (oldest first, starting and ending
/// with the user's) to the service `prefs` describe.
pub fn build_request(prefs: &AiPrefs, key: &str, messages: &[(Role, String)]) -> HttpRequest {
    build_request_with(prefs, key, SYSTEM_PROMPT, messages)
}

/// [`build_request`] with other instructions (`system`).
pub fn build_request_with(
    prefs: &AiPrefs,
    key: &str,
    system: &str,
    messages: &[(Role, String)],
) -> HttpRequest {
    let msgs: Vec<Value> = messages
        .iter()
        .map(|(role, text)| json!({"role": role.name(), "content": text}))
        .collect();
    let body = match prefs.provider {
        Provider::Anthropic => json!({
            "model": prefs.model(),
            "max_tokens": MAX_TOKENS,
            "system": system,
            "messages": msgs,
        }),
        Provider::OpenAiCompatible => {
            let mut all = vec![json!({"role": "system", "content": system})];
            all.extend(msgs);
            json!({"model": prefs.model(), "messages": all})
        }
    };
    HttpRequest {
        method: "POST",
        url: prefs.endpoint(),
        headers: headers(prefs, key),
        body: body.to_string(),
        bytes: Vec::new(),
    }
}

/// The request for the service's list of models.
pub fn build_models_request(prefs: &AiPrefs, key: &str) -> HttpRequest {
    HttpRequest {
        method: "GET",
        url: prefs.models_endpoint(),
        headers: headers(prefs, key),
        body: String::new(),
        bytes: Vec::new(),
    }
}

/// The message for a response that is an error: the service's own words
/// with the status.
fn http_error(r: &HttpResponse, v: Option<&Value>) -> String {
    let message = v
        .and_then(|v| v.pointer("/error/message").and_then(|m| m.as_str()))
        .map(str::to_string)
        .unwrap_or_else(|| r.body.chars().take(200).collect());
    trf(
        "ai.http_error",
        &[
            ("status", &r.status.to_string()),
            ("message", message.trim()),
        ],
    )
}

/// The models a service listed (`data[].id` in both kinds of service),
/// or why there are none.
pub fn parse_models(r: &HttpResponse) -> Result<Vec<String>, String> {
    let v: Option<Value> = serde_json::from_str(&r.body).ok();
    if !(200..300).contains(&r.status) {
        return Err(http_error(r, v.as_ref()));
    }
    let ids: Vec<String> = v
        .as_ref()
        .and_then(|v| v.get("data"))
        .and_then(|d| d.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if ids.is_empty() {
        return Err(tr("ai.no_models"));
    }
    Ok(ids)
}

/// The text of a reply, or why there is none (as shown to the user).
pub fn parse_reply(provider: Provider, r: &HttpResponse) -> Result<String, String> {
    let v: Option<Value> = serde_json::from_str(&r.body).ok();
    if !(200..300).contains(&r.status) {
        return Err(http_error(r, v.as_ref()));
    }
    let Some(v) = v else {
        return Err(tr("ai.unreadable_reply"));
    };
    let (text, cut) = match provider {
        Provider::Anthropic => {
            let text: String = v
                .get("content")
                .and_then(|c| c.as_array())
                .map(|parts| {
                    parts
                        .iter()
                        .filter(|p| p.get("type").and_then(|t| t.as_str()) == Some("text"))
                        .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                        .collect()
                })
                .unwrap_or_default();
            let cut = v.get("stop_reason").and_then(|s| s.as_str()) == Some("max_tokens");
            (text, cut)
        }
        Provider::OpenAiCompatible => {
            let text = v
                .pointer("/choices/0/message/content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            let cut = v
                .pointer("/choices/0/finish_reason")
                .and_then(|s| s.as_str())
                == Some("length");
            (text, cut)
        }
    };
    if cut {
        return Err(tr("ai.cut_short"));
    }
    if text.trim().is_empty() {
        return Err(tr("ai.no_script"));
    }
    Ok(text)
}

/// The program in a reply: its JavaScript code block (or its first code
/// block), or the whole reply when it has no block but reads as code.
pub fn extract_script(reply: &str) -> Option<String> {
    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut rest = reply;
    while let Some(start) = rest.find("```") {
        let after = &rest[start + 3..];
        let Some(nl) = after.find('\n') else {
            break;
        };
        let lang = after[..nl].trim().to_ascii_lowercase();
        let body = &after[nl + 1..];
        match body.find("```") {
            Some(end) => {
                blocks.push((lang, body[..end].to_string()));
                rest = &body[end + 3..];
            }
            // A reply cut off inside its block: take what came.
            None => {
                blocks.push((lang, body.to_string()));
                break;
            }
        }
    }
    let code = blocks
        .iter()
        .find(|(lang, _)| matches!(lang.as_str(), "javascript" | "js" | ""))
        .or(blocks.first())
        .map(|(_, body)| body.trim().to_string())
        .or_else(|| {
            let t = reply.trim();
            let looks_like_code =
                t.contains("ActiveLayer") || t.contains("ActivePage") || t.contains("Create");
            looks_like_code.then(|| t.to_string())
        })?;
    (!code.is_empty()).then_some(code)
}

/// Send `req` and deliver the answer on `tx` (up to 8 MB of answer).
fn spawn(req: HttpRequest, tx: std::sync::mpsc::Sender<HttpResult>) {
    crate::net::spawn(req, 8 * 1024 * 1024, tx);
}

impl App {
    /// AI Drawing's Draw: send the request with the page as it is.
    pub fn ai_draw(&mut self) {
        if self.ai.busy() {
            return;
        }
        let request = self.ai.request.trim().to_string();
        let prefs = self.settings.ai.clone();
        let problem = if !self.ai_features_on() {
            Some("aiimg.off")
        } else if request.is_empty() {
            Some("ai.need_request")
        } else if prefs.provider.needs_key() && self.ai.key.trim().is_empty() {
            Some("ai.need_key")
        } else if prefs.model().is_empty() {
            Some("ai.need_model")
        } else if !self.has_document() {
            Some("ai.need_document")
        } else {
            None
        };
        if let Some(key) = problem {
            self.ai.status = AiStatus::Failed(tr(key));
            return;
        }
        let page = crate::describe::active_page(self, MAX_OBJECTS);
        let mut messages = Vec::new();
        for t in &self.ai.history {
            messages.push((Role::User, user_message(&t.request, None)));
            messages.push((Role::Assistant, t.reply.clone()));
        }
        messages.push((Role::User, user_message(&request, Some(&page))));
        self.ai_send(messages, request, false);
    }

    /// Send `messages`; the reply is taken in by [`App::poll_ai`] (at once
    /// with a test transport).
    fn ai_send(&mut self, messages: Vec<(Role, String)>, request: String, retried: bool) {
        let http = build_request(&self.settings.ai, &self.ai.key, &messages);
        self.ai.status = if retried {
            AiStatus::Retrying
        } else {
            AiStatus::Waiting
        };
        self.ai_start(
            http,
            Job::Draw {
                messages,
                request,
                retried,
            },
        );
    }

    /// Fetch the service's list of models for the model lists.
    pub fn ai_fetch_models(&mut self) {
        if self.ai.busy() {
            return;
        }
        if self.settings.ai.provider.needs_key() && self.ai.key.trim().is_empty() {
            self.ai.status = AiStatus::Failed(tr("ai.need_key"));
            return;
        }
        let http = build_models_request(&self.settings.ai, &self.ai.key);
        self.ai.status = AiStatus::Waiting;
        self.ai_start(http, Job::Models);
    }

    /// Send `http` for `job`.
    fn ai_start(&mut self, http: HttpRequest, job: Job) {
        let (tx, rx) = std::sync::mpsc::channel();
        match self.ai.fake {
            Some(fake) => {
                let _ = tx.send(fake(&http));
            }
            None => spawn(http.clone(), tx),
        }
        self.ai.sent.push(http);
        if self.ai.sent.len() > 8 {
            self.ai.sent.remove(0);
        }
        self.ai.pending = Some(Pending { rx, job });
        if self.ai.fake.is_some() {
            self.poll_ai();
        }
    }

    /// A service is set up: a model, and a key where the service needs
    /// one (a service at an address of the user's, such as a local model
    /// server, may take none).
    pub fn ai_ready(&self) -> bool {
        let p = &self.settings.ai;
        if p.model().is_empty() {
            return false;
        }
        let has_key = !self.ai.key.trim().is_empty();
        match p.provider {
            Provider::Anthropic => has_key,
            Provider::OpenAiCompatible => has_key || !p.endpoint.trim().is_empty(),
        }
    }

    /// The models to choose from: the service's list once fetched, else
    /// the usual ones, with the model in use first when it is in neither.
    pub fn ai_model_choices(&self) -> Vec<String> {
        let mut list: Vec<String> = if self.ai.models.is_empty() {
            self.settings
                .ai
                .provider
                .preset_models()
                .iter()
                .map(|m| m.to_string())
                .collect()
        } else {
            self.ai.models.clone()
        };
        let current = self.settings.ai.model();
        if !current.is_empty() && !list.contains(&current) {
            list.insert(0, current);
        }
        list
    }

    /// Stop waiting for the reply (it is dropped when it comes).
    pub fn ai_cancel(&mut self) {
        if self.ai.pending.take().is_some() {
            self.ai.status = AiStatus::Idle;
        }
    }

    /// Forget the earlier requests: the next one starts afresh.
    pub fn ai_new_conversation(&mut self) {
        self.ai.history.clear();
        self.ai.script.clear();
        self.ai.output.clear();
        if !self.ai.busy() {
            self.ai.status = AiStatus::Idle;
        }
    }

    /// Take in a reply that arrived. True while a request is still on its
    /// way (the frame loop then looks again shortly).
    pub fn poll_ai(&mut self) -> bool {
        let Some(p) = &self.ai.pending else {
            return false;
        };
        let result = match p.rx.try_recv() {
            Ok(r) => r,
            Err(std::sync::mpsc::TryRecvError::Empty) => return true,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(tr("ai.connection_lost")),
        };
        let Some(p) = self.ai.pending.take() else {
            return false;
        };
        self.ai_receive(result, p);
        self.ai.busy()
    }

    fn ai_receive(&mut self, result: HttpResult, p: Pending) {
        let (messages, request, retried) = match p.job {
            Job::Models => {
                match result.and_then(|r| parse_models(&r)) {
                    Ok(models) => {
                        self.ai.status = AiStatus::Listed(models.len());
                        self.ai.models = models;
                    }
                    Err(e) => self.ai.status = AiStatus::Failed(e),
                }
                return;
            }
            Job::Draw {
                messages,
                request,
                retried,
            } => (messages, request, retried),
        };
        let provider = self.settings.ai.provider;
        let reply = match result.and_then(|r| parse_reply(provider, &r)) {
            Ok(text) => text,
            Err(e) => {
                self.ai.status = AiStatus::Failed(e);
                return;
            }
        };
        let Some(script) = extract_script(&reply) else {
            self.ai.status = AiStatus::Failed(tr("ai.no_script"));
            return;
        };
        if script.len() > MAX_SCRIPT {
            self.ai.status = AiStatus::Failed(tr("ai.script_too_long"));
            return;
        }
        self.ai.script = script.clone();
        match crate::scripting::run_untrusted(self, &script, UNDO_LABEL) {
            Ok((output, changes)) => {
                self.ai.output = output;
                self.ai.status = AiStatus::Done(changes);
                // The request is done: the field is free for the next.
                if self.ai.request.trim() == request {
                    self.ai.request.clear();
                }
                self.ai.history.push(Turn { request, reply });
                while self.ai.history.len() > MAX_HISTORY {
                    self.ai.history.remove(0);
                }
                self.status = trf("status.ai_drawn", &[("n", &changes.to_string())]);
            }
            Err(e) if !retried => {
                let mut messages = messages;
                messages.push((Role::Assistant, reply));
                messages.push((Role::User, retry_message(&e)));
                self.ai_send(messages, request, true);
            }
            Err(e) => {
                self.ai.output.clear();
                self.ai.status = AiStatus::Failed(trf("ai.script_failed", &[("e", &e)]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anthropic_reply(text: &str) -> HttpResult {
        Ok(HttpResponse {
            status: 200,
            body: json!({
                "content": [{"type": "text", "text": text}],
                "stop_reason": "end_turn",
            })
            .to_string(),
        })
    }

    fn app() -> App {
        let mut app = App::headless();
        app.settings.ai = AiPrefs::default();
        app.ai.key = "sk-test".into();
        app
    }

    fn shapes(app: &App) -> usize {
        app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum()
    }

    #[test]
    fn requests_are_built_for_both_kinds_of_service() {
        let mut prefs = AiPrefs::default();
        let messages = vec![
            (Role::User, "Request: a".to_string()),
            (Role::Assistant, "```js\nx\n```".to_string()),
            (Role::User, "Request: b".to_string()),
        ];
        let r = build_request(&prefs, " k1 ", &messages);
        assert_eq!(r.url, "https://api.anthropic.com/v1/messages");
        let header = |r: &HttpRequest, name: &str| {
            r.headers
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
        };
        assert_eq!(header(&r, "x-api-key").as_deref(), Some("k1"));
        assert_eq!(
            header(&r, "anthropic-version").as_deref(),
            Some("2023-06-01")
        );
        let body: Value = serde_json::from_str(&r.body).expect("json");
        assert_eq!(body["model"], "claude-sonnet-5-5");
        assert_eq!(body["system"], SYSTEM_PROMPT);
        assert_eq!(body["messages"].as_array().map(Vec::len), Some(3));
        assert_eq!(body["messages"][1]["role"], "assistant");
        assert!(body["max_tokens"].as_u64().is_some());
        // Chat completions: the system message first, a bearer key only
        // when there is one, the address and model as set.
        prefs.provider = Provider::OpenAiCompatible;
        prefs.endpoint = " http://localhost:11434/v1/chat/completions ".into();
        prefs.model = "llama3".into();
        let r = build_request(&prefs, "", &messages);
        assert_eq!(r.url, "http://localhost:11434/v1/chat/completions");
        assert!(header(&r, "authorization").is_none());
        let body: Value = serde_json::from_str(&r.body).expect("json");
        assert_eq!(body["model"], "llama3");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"].as_array().map(Vec::len), Some(4));
        let r = build_request(&prefs, "k2", &messages);
        assert_eq!(header(&r, "authorization").as_deref(), Some("Bearer k2"));
        assert!(header(&r, "x-opencode-session").is_none());
    }

    #[test]
    fn opencode_requests_name_their_session() {
        let messages = vec![(Role::User, "Request: a".to_string())];
        let header = |r: &HttpRequest, name: &str| {
            r.headers
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
        };
        let mut ids = Vec::new();
        for key in ["aipreset.opencode"] {
            let (_, provider, endpoint) = PRESETS
                .iter()
                .find(|(k, _, _)| *k == key)
                .copied()
                .expect("preset");
            let prefs = AiPrefs {
                provider,
                endpoint: endpoint.to_string(),
                model: "glm-5.1".into(),
                ..AiPrefs::default()
            };
            assert_eq!(preset_of(&prefs).map(|i| PRESETS[i].0), Some(key));
            let r = build_request(&prefs, "k", &messages);
            assert_eq!(r.url, endpoint);
            assert_eq!(header(&r, "authorization").as_deref(), Some("Bearer k"));
            let id = header(&r, "x-opencode-session").expect("session header");
            assert!(id.starts_with("tracedraw-") && id.len() > 20, "{id}");
            ids.push(id);
        }
        assert_eq!(
            AiPrefs {
                provider: Provider::OpenAiCompatible,
                endpoint: PRESETS[7].2.into(),
                ..AiPrefs::default()
            }
            .image_support(),
            ImageSupport::None
        );
        // One session for the whole run, the same on every request.
        let again = build_request(
            &AiPrefs {
                provider: Provider::OpenAiCompatible,
                endpoint: PRESETS[7].2.into(),
                ..AiPrefs::default()
            },
            "k",
            &messages,
        );
        assert_eq!(header(&again, "x-opencode-session"), ids.first().cloned());
    }

    #[test]
    fn replies_and_errors_are_read() {
        let ok = anthropic_reply("```javascript\nprint(1)\n```").expect("reply");
        assert!(parse_reply(Provider::Anthropic, &ok)
            .expect("text")
            .contains("print(1)"));
        let chat = HttpResponse {
            status: 200,
            body: json!({"choices": [{"message": {"content": "hi"}, "finish_reason": "stop"}]})
                .to_string(),
        };
        assert_eq!(
            parse_reply(Provider::OpenAiCompatible, &chat).as_deref(),
            Ok("hi")
        );
        // The service's own message for an error.
        let denied = HttpResponse {
            status: 401,
            body: json!({"type": "error", "error": {"type": "authentication_error", "message": "invalid x-api-key"}})
                .to_string(),
        };
        let e = parse_reply(Provider::Anthropic, &denied).expect_err("an error");
        assert!(e.contains("401") && e.contains("invalid x-api-key"), "{e}");
        // Not JSON at all.
        let page = HttpResponse {
            status: 502,
            body: "<html>Bad gateway</html>".into(),
        };
        assert!(parse_reply(Provider::OpenAiCompatible, &page)
            .expect_err("an error")
            .contains("502"));
        // Cut short, and empty.
        let cut = HttpResponse {
            status: 200,
            body: json!({"content": [{"type": "text", "text": "```js\nActiveLayer."}], "stop_reason": "max_tokens"})
                .to_string(),
        };
        assert!(parse_reply(Provider::Anthropic, &cut).is_err());
        let empty = HttpResponse {
            status: 200,
            body: json!({"choices": [{"message": {"content": ""}}]}).to_string(),
        };
        assert!(parse_reply(Provider::OpenAiCompatible, &empty).is_err());
    }

    #[test]
    fn the_program_is_taken_from_its_code_block() {
        let reply =
            "Here you go:\n```javascript\nActiveLayer.CreateEllipse(0, 0, 5, 5);\n```\nEnjoy.";
        assert_eq!(
            extract_script(reply).as_deref(),
            Some("ActiveLayer.CreateEllipse(0, 0, 5, 5);")
        );
        // The JavaScript block wins over others; a bare block counts.
        let two = "```text\nnot this\n```\n```js\nprint(2)\n```";
        assert_eq!(extract_script(two).as_deref(), Some("print(2)"));
        assert_eq!(
            extract_script("```\nprint(3)\n```").as_deref(),
            Some("print(3)")
        );
        // A reply that is only code, and one that is not.
        assert_eq!(
            extract_script("ActiveLayer.CreateRectangle(0, 0, 1, 1);").as_deref(),
            Some("ActiveLayer.CreateRectangle(0, 0, 1, 1);")
        );
        assert_eq!(extract_script("Sorry, I cannot help with that."), None);
        assert_eq!(extract_script("```js\n\n```"), None);
        // Unterminated block.
        assert_eq!(
            extract_script("```js\nprint(4)").as_deref(),
            Some("print(4)")
        );
    }

    #[test]
    fn a_request_draws_through_the_service_as_one_undo_step() {
        let mut app = app();
        let before = shapes(&app);
        app.ai.fake = Some(|req| {
            let body: Value = serde_json::from_str(&req.body).expect("json");
            let last = body["messages"]
                .as_array()
                .and_then(|m| m.last())
                .and_then(|m| m["content"].as_str())
                .unwrap_or("");
            assert!(last.starts_with("Request: a red circle"), "{last}");
            assert!(last.contains("\"width_mm\""), "the page goes along");
            anthropic_reply(
                "```javascript\nconst c = ActiveLayer.CreateEllipse(80, 120, 50, 50);\nc.Fill.ApplyUniformFill(Color.RGB(255, 0, 0));\nprint('circle');\n```",
            )
        });
        app.ai.request = "a red circle".into();
        app.ai_draw();
        assert_eq!(app.ai.status, AiStatus::Done(2));
        assert_eq!(app.ai.output, vec!["circle".to_string()]);
        assert_eq!(shapes(&app), before + 1);
        assert_eq!(app.ai.history.len(), 1);
        assert!(app.ai.script.contains("CreateEllipse"));
        // One undo step takes it all back.
        app.undo();
        assert_eq!(shapes(&app), before);
        // The next request carries the earlier one, without its page.
        app.ai.fake = Some(|req| {
            let body: Value = serde_json::from_str(&req.body).expect("json");
            let m = body["messages"].as_array().cloned().unwrap_or_default();
            assert_eq!(m.len(), 3, "{m:?}");
            assert_eq!(m[0]["content"], "Request: a red circle");
            anthropic_reply("```js\nActiveLayer.CreateRectangle(0, 0, 10, 10);\n```")
        });
        app.ai.request = "now a square".into();
        app.ai_draw();
        assert_eq!(app.ai.status, AiStatus::Done(1));
        app.ai_new_conversation();
        assert!(app.ai.history.is_empty());
    }

    #[test]
    fn a_failing_script_is_sent_back_once_and_nothing_half_drawn_stays() {
        let mut app = app();
        let before = shapes(&app);
        // Draws one object, then fails: the corrected script is asked for
        // with the error, and the half-drawn object does not stay.
        app.ai.fake = Some(|req| {
            let body: Value = serde_json::from_str(&req.body).expect("json");
            let m = body["messages"].as_array().cloned().unwrap_or_default();
            if m.len() == 1 {
                anthropic_reply(
                    "```js\nActiveLayer.CreateRectangle(0, 0, 5, 5);\nnoSuchFunction();\n```",
                )
            } else {
                let last = m.last().and_then(|m| m["content"].as_str()).unwrap_or("");
                assert!(last.contains("noSuchFunction"), "{last}");
                assert_eq!(m[1]["role"], "assistant");
                anthropic_reply("```js\nActiveLayer.CreateRectangle(0, 0, 5, 5);\n```")
            }
        });
        app.ai.request = "a square".into();
        app.ai_draw();
        assert_eq!(app.ai.status, AiStatus::Done(1));
        assert_eq!(shapes(&app), before + 1);
        assert_eq!(app.ai.sent.len(), 2);
        // A script that fails twice draws nothing and says why.
        app.ai.fake = Some(|_| {
            anthropic_reply("```js\nActiveLayer.CreateRectangle(0, 0, 5, 5);\nboom();\n```")
        });
        app.ai.request = "another".into();
        app.ai_draw();
        assert!(
            matches!(&app.ai.status, AiStatus::Failed(e) if e.contains("boom")),
            "{:?}",
            app.ai.status
        );
        assert_eq!(shapes(&app), before + 1);
    }

    #[test]
    fn scripts_from_the_service_cannot_reach_files_or_run_forever() {
        let mut app = app();
        let dir = std::env::temp_dir().join(format!("tracedraw-ai-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let target = dir.join("x.svg");
        let src = format!(
            "ActiveDocument.Export('{}', 'svg');",
            target.display().to_string().replace('\\', "/")
        );
        let e = crate::scripting::run_untrusted(&mut app, &src, "AI Drawing").expect_err("refused");
        assert!(e.contains("file"), "{e}");
        assert!(!target.exists());
        let e = crate::scripting::run_untrusted(
            &mut app,
            "ActiveLayer.ImportBitmap('/etc/passwd');",
            "AI Drawing",
        )
        .expect_err("refused");
        assert!(e.contains("file"), "{e}");
        // An endless loop stops at the limit.
        let e = crate::scripting::run_untrusted(&mut app, "while (true) {}", "AI Drawing")
            .expect_err("stopped");
        assert!(e.to_lowercase().contains("loop"), "{e}");
        // The user's own scripts still export.
        assert!(crate::scripting::run(&mut app, &src).is_ok());
        assert!(target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn requests_need_words_a_key_and_a_model() {
        let mut app = app();
        app.ai.fake = Some(|_| panic!("nothing is sent"));
        app.ai.request = "  ".into();
        app.ai_draw();
        assert!(matches!(app.ai.status, AiStatus::Failed(_)));
        app.ai.request = "a tree".into();
        app.ai.key.clear();
        app.ai_draw();
        assert_eq!(app.ai.status, AiStatus::Failed(tr("ai.need_key")));
        // A local server takes no key but needs a model name.
        app.settings.ai.provider = Provider::OpenAiCompatible;
        app.ai_draw();
        assert_eq!(app.ai.status, AiStatus::Failed(tr("ai.need_model")));
        assert!(app.ai.sent.is_empty());
    }

    #[test]
    fn errors_of_the_service_and_the_network_are_shown() {
        let mut app = app();
        app.ai.request = "a tree".into();
        app.ai.fake = Some(|_| Err("connection refused".into()));
        app.ai_draw();
        assert_eq!(app.ai.status, AiStatus::Failed("connection refused".into()));
        app.ai.fake = Some(|_| {
            Ok(HttpResponse {
                status: 429,
                body: json!({"error": {"message": "slow down"}}).to_string(),
            })
        });
        app.ai_draw();
        assert!(matches!(&app.ai.status, AiStatus::Failed(e) if e.contains("slow down")));
        app.ai.fake = Some(|_| anthropic_reply("I would rather not."));
        app.ai_draw();
        assert_eq!(app.ai.status, AiStatus::Failed(tr("ai.no_script")));
        assert!(!app.ai.busy());
    }

    /// The desktop transport against a server on this computer: the
    /// request as sent, a reply, an error reply with its body, and no
    /// server at all.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_desktop_transport_posts_and_reads_error_replies() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a port");
        let addr = listener.local_addr().expect("address");
        let server = std::thread::spawn(move || {
            let mut seen = Vec::new();
            for (status, body) in [
                (200, r#"{"ok":true}"#),
                (401, r#"{"error":{"message":"bad key"}}"#),
            ] {
                let Ok((mut s, _)) = listener.accept() else {
                    break;
                };
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let n = s.read(&mut chunk).unwrap_or(0);
                    buf.extend_from_slice(&chunk[..n]);
                    let text = String::from_utf8_lossy(&buf).to_string();
                    let done = text.find("\r\n\r\n").is_some_and(|end| {
                        let len = text[..end]
                            .lines()
                            .find_map(|l| {
                                let l = l.to_ascii_lowercase();
                                l.strip_prefix("content-length:")
                                    .and_then(|v| v.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        buf.len() >= end + 4 + len
                    });
                    if done || n == 0 {
                        break;
                    }
                }
                seen.push(String::from_utf8_lossy(&buf).to_string());
                let _ = write!(
                    s,
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
            seen
        });
        let req = HttpRequest {
            method: "POST",
            url: format!("http://{addr}/v1/chat/completions"),
            headers: vec![
                ("content-type".into(), "application/json".into()),
                ("authorization".into(), "Bearer k".into()),
            ],
            body: r#"{"a":1}"#.into(),
            bytes: Vec::new(),
        };
        let r = crate::net::send(&req, 8 * 1024 * 1024).expect("answered");
        assert_eq!((r.status, r.body.as_str()), (200, r#"{"ok":true}"#));
        let r = crate::net::send(&req, 8 * 1024 * 1024).expect("answered");
        assert_eq!(r.status, 401);
        assert!(parse_reply(Provider::OpenAiCompatible, &r)
            .expect_err("an error")
            .contains("bad key"));
        let seen = server.join().expect("server");
        assert!(
            seen[0].starts_with("POST /v1/chat/completions"),
            "{}",
            seen[0]
        );
        assert!(seen[0]
            .to_ascii_lowercase()
            .contains("authorization: bearer k"));
        assert!(seen[0].ends_with(r#"{"a":1}"#));
        // Nobody listening.
        let closed = std::net::TcpListener::bind("127.0.0.1:0").expect("a port");
        let gone = closed.local_addr().expect("address");
        drop(closed);
        let dead = HttpRequest {
            url: format!("http://{gone}/"),
            ..req
        };
        assert!(crate::net::send(&dead, 8 * 1024 * 1024).is_err());
    }

    /// A real service refuses a made-up key with its own message (needs
    /// the network: `cargo test -p tracedraw -- --ignored the_real`).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore]
    fn the_real_service_refuses_a_made_up_key() {
        let prefs = AiPrefs::default();
        let req = build_request(
            &prefs,
            "not-a-key",
            &[(Role::User, user_message("a dot", None))],
        );
        let r = crate::net::send(&req, 8 * 1024 * 1024).expect("the service answers");
        assert_eq!(r.status, 401, "{}", r.body);
        let e = parse_reply(prefs.provider, &r).expect_err("refused");
        assert!(e.contains("401"), "{e}");
    }

    #[test]
    fn paths_in_svg_syntax_draw_curves() {
        let mut app = app();
        let before = shapes(&app);
        let out = crate::scripting::run(
            &mut app,
            "const p = ActiveLayer.CreatePath('M 10 10 C 20 40, 40 40, 50 10 Z'); print(p.SizeWidth > 39 && p.SizeWidth < 41);",
        )
        .expect("ran");
        assert_eq!(out, vec!["true".to_string()]);
        assert_eq!(shapes(&app), before + 1);
        assert!(crate::scripting::run(&mut app, "ActiveLayer.CreatePath('Q nonsense');").is_err());
    }

    #[test]
    fn the_reference_names_only_methods_the_object_model_has() {
        for name in [
            "CreateRectangle",
            "CreateEllipse",
            "CreatePolygon",
            "CreateStar",
            "CreateLineSegment",
            "CreateCurve",
            "CreatePath",
            "CreateArtisticText",
            "CreateParagraphText",
            "ApplyUniformFill",
            "ApplyFountainFill",
            "ApplyNoFill",
            "SetProperties",
            "SetNoOutline",
            "SetPosition",
            "SetSize",
            "Rotate",
            "Stretch",
            "Skew",
            "Duplicate",
            "OrderToFront",
            "OrderToBack",
            "SetOpacity",
            "AddDropShadow",
            "FindShape",
            "CreateLayer",
            "AlignCentersHorizontally",
        ] {
            assert!(SYSTEM_PROMPT.contains(name), "{name} described");
            let src = format!("print(typeof ActiveLayer.{name} + typeof new Shape(0).{name} + typeof new Shape(0).Fill.{name} + typeof new Shape(0).Outline.{name} + typeof new ShapeRange([]).{name} + typeof ActivePage.{name} + typeof ActiveLayer.Shapes.{name});");
            let mut app = App::headless();
            let out = crate::scripting::run(&mut app, &src).expect("ran");
            assert!(
                out.first().is_some_and(|o| o.contains("function")),
                "{name} exists: {out:?}"
            );
        }
    }

    /// Against the real Messages API: lists the models and asks for a
    /// small drawing. Run with the key in `TRACEDRAW_ANTHROPIC_KEY`;
    /// without it, only checks that the service answers the request with
    /// its own "no key" error, read as the program shows it.
    #[test]
    #[ignore]
    fn live_claude_api() {
        let send = |req: HttpRequest| {
            let (tx, rx) = std::sync::mpsc::channel();
            crate::net::spawn(req, 8 * 1024 * 1024, tx);
            rx.recv_timeout(std::time::Duration::from_secs(120))
                .expect("an answer")
                .expect("a response")
        };
        let prefs = AiPrefs {
            provider: Provider::Anthropic,
            ..AiPrefs::default()
        };
        let key = std::env::var("TRACEDRAW_ANTHROPIC_KEY").unwrap_or_default();
        let listed = parse_models(&send(build_models_request(&prefs, &key)));
        if key.is_empty() {
            let e = listed.expect_err("no key");
            eprintln!("without a key: {e}");
            assert!(e.contains("401"), "{e}");
            return;
        }
        let models = listed.expect("the models");
        eprintln!("models: {models:?}");
        assert!(models.iter().any(|m| m.starts_with("claude-")));
        let reply = send(build_request(
            &prefs,
            &key,
            &[(
                Role::User,
                "Draw one red circle in the middle of the page.".into(),
            )],
        ));
        let text = parse_reply(Provider::Anthropic, &reply).expect("a reply");
        eprintln!("reply: {}", text.chars().take(400).collect::<String>());
        assert!(!text.trim().is_empty());
    }
}
