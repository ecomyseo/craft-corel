//! AI images: pictures made, remixed or cut out by the AI service the
//! user sets up with their own key (Tools > Options > AI > Service), when
//! that service can make pictures. Craft Corel has no AI of its own;
//! nothing is sent until the user asks.
//!
//! Two kinds of image API are spoken to: Google's Gemini API
//! (`generateContent` with an image model) and the OpenAI Images API
//! (`images/generations` and `images/edits`), which other services and
//! local servers also offer at an address of the user's. Which one, and
//! where, follows from the one service (`AiPrefs::image_prefs`). Replies carry
//! the pictures as base64; they become bitmap objects on the page. See
//! docs/behavior/ai-generate.md.

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::net::{HttpRequest, HttpResponse, HttpResult};
use tracedraw_core::document::{Shape, ShapeKind};
use tracedraw_core::{Command, Rect, ShapeId};

/// The kinds of image service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageProvider {
    /// Google's Gemini API.
    #[default]
    Gemini,
    /// The OpenAI Images API (also offered by other services).
    OpenAi,
}

impl ImageProvider {
    pub fn key(self) -> &'static str {
        match self {
            ImageProvider::Gemini => "aiimg.provider_gemini",
            ImageProvider::OpenAi => "aiimg.provider_openai",
        }
    }

    /// The service's base address when none is set.
    pub fn default_endpoint(self) -> &'static str {
        match self {
            ImageProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            ImageProvider::OpenAi => "https://api.openai.com/v1",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            ImageProvider::Gemini => "gemini-2.5-flash-image",
            ImageProvider::OpenAi => "gpt-image-1",
        }
    }

    /// Models offered before the service's own list is fetched.
    pub fn preset_models(self) -> &'static [&'static str] {
        match self {
            ImageProvider::Gemini => &["gemini-2.5-flash-image", "gemini-3-pro-image-preview"],
            ImageProvider::OpenAi => &["gpt-image-1", "gpt-image-1-mini", "dall-e-3"],
        }
    }
}

/// An image service: derived from the one AI service
/// (`AiPrefs::image_prefs`, with no key: the service's key goes with each
/// request), and the shape of the separate image service older versions
/// saved (read once and folded into the one service, see
/// `settings::migrate_ai`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ImagePrefs {
    pub provider: ImageProvider,
    /// Empty: the provider's usual address.
    pub endpoint: String,
    /// Empty: the provider's usual model.
    pub model: String,
    pub key: String,
    pub remember_key: bool,
}

impl ImagePrefs {
    pub fn endpoint(&self) -> String {
        let e = self.endpoint.trim().trim_end_matches('/');
        if e.is_empty() {
            self.provider.default_endpoint().to_string()
        } else {
            e.to_string()
        }
    }

    pub fn model(&self) -> String {
        let m = self.model.trim();
        if m.is_empty() {
            self.provider.default_model().to_string()
        } else {
            m.to_string()
        }
    }

    /// A service at an address of the user's may take no key.
    pub fn needs_key(&self) -> bool {
        self.endpoint.trim().is_empty()
    }
}

/// The picture's shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Aspect {
    #[default]
    Square,
    Landscape43,
    Landscape32,
    Wide169,
    Portrait34,
    Portrait23,
    Tall916,
    Landscape54,
    Portrait45,
    Ultrawide219,
    /// The reference image's own shape.
    MatchSource,
}

impl Aspect {
    pub const ALL: [Aspect; 10] = [
        Aspect::Square,
        Aspect::Landscape54,
        Aspect::Landscape43,
        Aspect::Landscape32,
        Aspect::Wide169,
        Aspect::Ultrawide219,
        Aspect::Portrait45,
        Aspect::Portrait34,
        Aspect::Portrait23,
        Aspect::Tall916,
    ];

    /// The ratio nearest to a `w` by `h` picture.
    pub fn nearest(w: u32, h: u32) -> Aspect {
        let r = (w.max(1) as f64 / h.max(1) as f64).ln();
        Aspect::ALL
            .into_iter()
            .min_by(|a, b| {
                let d = |x: Aspect| (x.value().ln() - r).abs();
                d(*a).total_cmp(&d(*b))
            })
            .unwrap_or_default()
    }

    /// Width over height.
    fn value(self) -> f64 {
        let (w, h) = self.ratio().split_once(':').unwrap_or(("1", "1"));
        w.parse::<f64>().unwrap_or(1.0) / h.parse::<f64>().unwrap_or(1.0).max(1e-9)
    }

    pub fn key(self) -> &'static str {
        match self {
            Aspect::Square => "aiimg.aspect_square",
            Aspect::Landscape43 => "aiimg.aspect_43",
            Aspect::Landscape32 => "aiimg.aspect_32",
            Aspect::Wide169 => "aiimg.aspect_169",
            Aspect::Portrait34 => "aiimg.aspect_34",
            Aspect::Portrait23 => "aiimg.aspect_23",
            Aspect::Tall916 => "aiimg.aspect_916",
            Aspect::Landscape54 => "aiimg.aspect_54",
            Aspect::Portrait45 => "aiimg.aspect_45",
            Aspect::Ultrawide219 => "aiimg.aspect_219",
            Aspect::MatchSource => "aiimg.aspect_match",
        }
    }

    /// The ratio as the Gemini API names it.
    pub fn ratio(self) -> &'static str {
        match self {
            Aspect::Square => "1:1",
            Aspect::Landscape43 => "4:3",
            Aspect::Landscape32 => "3:2",
            Aspect::Wide169 => "16:9",
            Aspect::Portrait34 => "3:4",
            Aspect::Portrait23 => "2:3",
            Aspect::Tall916 => "9:16",
            Aspect::Landscape54 => "5:4",
            Aspect::Portrait45 => "4:5",
            Aspect::Ultrawide219 => "21:9",
            Aspect::MatchSource => "1:1",
        }
    }

    fn landscape(self) -> Option<bool> {
        match self {
            Aspect::Square | Aspect::MatchSource => None,
            Aspect::Landscape43
            | Aspect::Landscape32
            | Aspect::Wide169
            | Aspect::Landscape54
            | Aspect::Ultrawide219 => Some(true),
            _ => Some(false),
        }
    }

    /// The nearest size the OpenAI Images API offers for `model`.
    pub fn openai_size(self, model: &str) -> &'static str {
        let dalle3 = model.starts_with("dall-e-3");
        if model.starts_with("dall-e-2") {
            return "1024x1024";
        }
        match (self.landscape(), dalle3) {
            (None, _) => "1024x1024",
            (Some(true), false) => "1536x1024",
            (Some(false), false) => "1024x1536",
            (Some(true), true) => "1792x1024",
            (Some(false), true) => "1024x1792",
        }
    }
}

/// Style presets: the i18n key of the name and the words added to the
/// prompt.
pub const STYLES: [(&str, &str); 6] = [
    (
        "aiimg.style_photo",
        "Photorealistic, natural light, sharp detail.",
    ),
    (
        "aiimg.style_line_art",
        "Black line art on white, clean even lines, no shading.",
    ),
    (
        "aiimg.style_watercolor",
        "Watercolour painting on textured paper, soft washes.",
    ),
    (
        "aiimg.style_illustration",
        "Flat editorial illustration, bold shapes.",
    ),
    (
        "aiimg.style_3d",
        "3D render, soft studio lighting, smooth materials.",
    ),
    (
        "aiimg.style_pop_art",
        "Pop art, bold outlines, halftone dots, saturated colours.",
    ),
];

/// Styles of Create vectors: the i18n key of the name and the words
/// added to the prompt.
pub const VECTOR_STYLES: [(&str, &str); 6] = [
    (
        "aiimg.vstyle_flat",
        "Flat illustration with clean geometric shapes.",
    ),
    (
        "aiimg.vstyle_line",
        "Line art: even black strokes, no fills.",
    ),
    (
        "aiimg.vstyle_comic",
        "Comic book style: bold black outlines, flat cel colours.",
    ),
    (
        "aiimg.vstyle_icon",
        "A simple icon: few shapes, thick strokes, centred.",
    ),
    (
        "aiimg.vstyle_hand",
        "Hand drawn: slightly irregular organic lines and shapes.",
    ),
    (
        "aiimg.vstyle_kawaii",
        "Kawaii poster: cute rounded characters, pastel colours, playful shapes.",
    ),
];

/// Ready-made remix requests (the workflows remixing is good at): the
/// i18n key of the name and the prompt it fills in.
/// Image > AI Art Styles: the picture repainted in a style, keeping
/// what it shows (i18n key, request).
pub const ART_STYLES: [(&str, &str); 8] = [
    ("aiimg.art_oil", "Repaint this picture as an oil painting with thick, visible brush strokes and rich colour. Keep its subject and composition exactly."),
    ("aiimg.art_watercolor", "Repaint this picture as a loose watercolour painting on textured paper, with soft washes and bleeding edges. Keep its subject and composition exactly."),
    ("aiimg.art_pencil", "Redraw this picture as a detailed graphite pencil sketch with shading and cross-hatching on white paper. Keep its subject and composition exactly."),
    ("aiimg.art_ink", "Redraw this picture as a black ink drawing with bold outlines and hatching. Keep its subject and composition exactly."),
    ("aiimg.art_pastel", "Repaint this picture as a soft chalk pastel drawing on toned paper. Keep its subject and composition exactly."),
    ("aiimg.art_pop", "Repaint this picture as pop art with flat bright colours, thick outlines and halftone dots. Keep its subject and composition exactly."),
    ("aiimg.art_woodcut", "Redraw this picture as a woodcut print with carved lines in two colours. Keep its subject and composition exactly."),
    ("aiimg.art_mosaic", "Recreate this picture as a stained glass mosaic with lead lines between coloured pieces. Keep its subject and composition exactly."),
];

pub const REMIX_IDEAS: [(&str, &str); 6] = [
    ("aiimg.remix_mockup", "Use this design to decorate a photorealistic mockup of a mug, a notebook and a folded standing menu on a cafe table."),
    ("aiimg.remix_restore", "Restore this old photo as if it was taken today with a high-resolution camera: repair scratches and tears, fill in missing areas, clean and sharpen it, keep the faces unchanged."),
    ("aiimg.remix_colorize", "Colorize this sketch as a natural watercolour painting, complete the unfinished parts and add a soft summer landscape background."),
    ("aiimg.remix_background", "Replace the background with a peaceful sunlit forest, keep the subject exactly as it is."),
    ("aiimg.remix_season", "Turn this scene into winter: snow on the ground, cold light, winter clothing, keep the composition."),
    ("aiimg.remix_recolor", "Recolour the design with a light beige background, yellow and red accents and deep blue details, keep every shape."),
];

/// Prompts Inspire Me picks from.
pub const IDEAS: [&str; 12] = [
    "A lighthouse on a rocky coast at dawn, seagulls, calm sea",
    "A cosy reading corner with plants and warm lamp light",
    "A fox in a snowy forest, looking over its shoulder",
    "A retro bicycle with a basket of flowers against a pastel wall",
    "A mountain cabin under a starry sky with the northern lights",
    "A bowl of ramen seen from above on a wooden table",
    "A hot air balloon flying above layered mountains and forested hills, soft clouds",
    "A friendly robot watering plants in a greenhouse",
    "A cheerful yeti caricature dancing in the Himalayas, big glasses, red t-shirt",
    "An astronaut woman with a striking helmet visor on a purple planet",
    "Folk art heart of flowers, cats and birds in bright colours",
    "A futuristic treehouse in a redwood forest, glass walls, morning mist",
];

/// The instructions of Create vectors with the drawing service: the
/// reply is an SVG document.
pub const SVG_PROMPT: &str = "You are a vector illustrator. Reply with one complete, valid SVG document and nothing else: an <svg> element with xmlns=\"http://www.w3.org/2000/svg\" and a viewBox, made only of path, rect, circle, ellipse, polygon, polyline, line and g elements with solid fill and stroke colours (linear and radial gradients allowed). No text, no images, no scripts, no external references, no CSS classes. Use clean, closed shapes in a pleasing composition and keep it under 400 elements.";

/// Colour palette presets: the i18n key of the name, the words added to
/// the prompt and the colours of its swatch.
pub const PALETTES: [(&str, &str, [u32; 3]); 8] = [
    (
        "aiimg.palette_full",
        "Use the full colour spectrum.",
        [0xE53935, 0x43A047, 0x1E88E5],
    ),
    (
        "aiimg.palette_gray",
        "Only shades of grey.",
        [0x111111, 0x888888, 0xEEEEEE],
    ),
    (
        "aiimg.palette_warm",
        "A warm palette of reds, oranges and yellows.",
        [0xC62828, 0xEF6C00, 0xFDD835],
    ),
    (
        "aiimg.palette_cool",
        "A cool palette of blues, teals and violets.",
        [0x1A237E, 0x00897B, 0x7E57C2],
    ),
    (
        "aiimg.palette_pastel",
        "Soft pastel colours.",
        [0xF8BBD0, 0xB3E5FC, 0xC8E6C9],
    ),
    (
        "aiimg.palette_neon",
        "Vivid neon colours on dark tones.",
        [0xFF00E5, 0x00E5FF, 0x76FF03],
    ),
    (
        "aiimg.palette_earth",
        "Earthy browns, olives and sand.",
        [0x5D4037, 0x827717, 0xD7CCC8],
    ),
    (
        "aiimg.palette_mono_blue",
        "Shades of a single blue.",
        [0x0D47A1, 0x42A5F5, 0xBBDEFB],
    ),
];

/// The words that make a picture fit for tracing into vectors.
const VECTOR_PROMPT: &str = "Flat vector illustration: clean simple shapes, a few solid colours, no gradients, no textures, no shadows, on a plain white background.";
/// Remove Background, for services that can answer with transparency.
const CUTOUT_PROMPT: &str = "Remove the background completely and keep only the main subject, unchanged, at the same size and position, on a transparent background.";
/// Remove Background, for services that cannot: a flat key colour is
/// asked for and keyed out here.
const CHROMA_PROMPT: &str = "Keep the main subject exactly as it is, at the same size and position, and replace everything else with a flat, even, pure green background (#00FF00) with no shadows or gradients.";

/// The longest prompt sent, in characters.
pub const MAX_PROMPT: usize = 4000;
/// Pictures asked for at a time.
pub const MAX_COUNT: u8 = 4;
/// Results kept in the docker.
const MAX_RESULTS: usize = 12;
/// Largest reply taken, in bytes (pictures come as base64).
const MAX_REPLY: u64 = 64 * 1024 * 1024;
/// The undo steps.
const LABEL_GENERATE: &str = "AI Generate";
const LABEL_CUTOUT: &str = "Remove Background";

/// The full prompt: the user's words, then the style and palette.
pub fn compose_prompt(
    prompt: &str,
    style: Option<usize>,
    palette: Option<usize>,
    vector: bool,
) -> String {
    let mut out = String::new();
    if vector {
        out.push_str(VECTOR_PROMPT);
        out.push(' ');
    }
    out.push_str(prompt.trim());
    if let Some((_, words)) = style.and_then(|i| STYLES.get(i)) {
        if !vector {
            out.push_str("\n\nStyle: ");
            out.push_str(words);
        }
    }
    if let Some((_, words, _)) = palette.and_then(|i| PALETTES.get(i)) {
        out.push_str("\n\nColours: ");
        out.push_str(words);
    }
    out.chars().take(MAX_PROMPT).collect()
}

fn headers(prefs: &ImagePrefs, key: &str, json: bool) -> Vec<(String, String)> {
    let key = key.trim();
    let mut h = Vec::new();
    if json {
        h.push(("content-type".to_string(), "application/json".to_string()));
    }
    if !key.is_empty() {
        match prefs.provider {
            ImageProvider::Gemini => h.push(("x-goog-api-key".to_string(), key.to_string())),
            ImageProvider::OpenAi => h.push(("authorization".to_string(), format!("Bearer {key}"))),
        }
    }
    h
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// The request for a new picture (Gemini makes one per request; the
/// OpenAI API `count`).
pub fn build_generate(
    prefs: &ImagePrefs,
    key: &str,
    prompt: &str,
    aspect: Aspect,
    count: u8,
) -> HttpRequest {
    match prefs.provider {
        ImageProvider::Gemini => gemini_request(prefs, key, prompt, None, Some(aspect)),
        ImageProvider::OpenAi => {
            let model = prefs.model();
            let mut body = json!({
                "model": model,
                "prompt": prompt,
                "n": count.clamp(1, MAX_COUNT),
                "size": aspect.openai_size(&model),
            });
            if model.starts_with("dall-e") {
                body["response_format"] = json!("b64_json");
            }
            HttpRequest {
                method: "POST",
                url: format!("{}/images/generations", prefs.endpoint()),
                headers: headers(prefs, key, true),
                body: body.to_string(),
                bytes: Vec::new(),
            }
        }
    }
}

/// The request for a changed picture: `png` with `prompt`; `cutout` asks
/// for a transparent background where the service can give one.
pub fn build_edit(
    prefs: &ImagePrefs,
    key: &str,
    prompt: &str,
    png: &[u8],
    cutout: bool,
) -> HttpRequest {
    match prefs.provider {
        ImageProvider::Gemini => gemini_request(prefs, key, prompt, Some(png), None),
        ImageProvider::OpenAi => {
            let boundary = "tracedraw-ai-image-boundary-7f3a9c";
            let mut fields = vec![
                ("model", prefs.model()),
                ("prompt", prompt.to_string()),
                ("n", "1".to_string()),
            ];
            if cutout {
                fields.push(("background", "transparent".to_string()));
                fields.push(("output_format", "png".to_string()));
            }
            let mut body = Vec::new();
            for (name, value) in fields {
                body.extend_from_slice(
                    format!(
                        "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
                    )
                    .as_bytes(),
                );
            }
            body.extend_from_slice(
                format!(
                    "--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"image.png\"\r\nContent-Type: image/png\r\n\r\n"
                )
                .as_bytes(),
            );
            body.extend_from_slice(png);
            body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
            let mut h = headers(prefs, key, false);
            h.push((
                "content-type".to_string(),
                format!("multipart/form-data; boundary={boundary}"),
            ));
            HttpRequest {
                method: "POST",
                url: format!("{}/images/edits", prefs.endpoint()),
                headers: h,
                body: String::new(),
                bytes: body,
            }
        }
    }
}

fn gemini_request(
    prefs: &ImagePrefs,
    key: &str,
    prompt: &str,
    png: Option<&[u8]>,
    aspect: Option<Aspect>,
) -> HttpRequest {
    let mut parts = Vec::new();
    if let Some(png) = png {
        parts.push(json!({"inline_data": {"mime_type": "image/png", "data": b64(png)}}));
    }
    parts.push(json!({"text": prompt}));
    let mut config = json!({"responseModalities": ["TEXT", "IMAGE"]});
    if let Some(a) = aspect {
        config["imageConfig"] = json!({"aspectRatio": a.ratio()});
    }
    let body = json!({
        "contents": [{"role": "user", "parts": parts}],
        "generationConfig": config,
    });
    HttpRequest {
        method: "POST",
        url: format!(
            "{}/models/{}:generateContent",
            prefs.endpoint(),
            prefs.model()
        ),
        headers: headers(prefs, key, true),
        body: body.to_string(),
        bytes: Vec::new(),
    }
}

/// The request for new pictures that follow a reference picture: Gemini
/// takes it with the prompt; the OpenAI API edits it.
pub fn build_reference(
    prefs: &ImagePrefs,
    key: &str,
    prompt: &str,
    png: &[u8],
    aspect: Aspect,
) -> HttpRequest {
    match prefs.provider {
        ImageProvider::Gemini => gemini_request(prefs, key, prompt, Some(png), Some(aspect)),
        ImageProvider::OpenAi => build_edit(prefs, key, prompt, png, false),
    }
}

/// The SVG document in a reply: an ```svg block, else the first
/// `<svg ...>...</svg>`.
pub fn extract_svg(reply: &str) -> Option<String> {
    let start = reply.find("<svg")?;
    let end = reply[start..].rfind("</svg>")? + start + "</svg>".len();
    let svg = reply.get(start..end)?.trim().to_string();
    (svg.len() <= 2 * 1024 * 1024).then_some(svg)
}

/// The request for the service's models.
pub fn build_models(prefs: &ImagePrefs, key: &str) -> HttpRequest {
    HttpRequest {
        method: "GET",
        url: format!("{}/models", prefs.endpoint()),
        headers: headers(prefs, key, false),
        body: String::new(),
        bytes: Vec::new(),
    }
}

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

/// The image models a service lists.
pub fn parse_models(provider: ImageProvider, r: &HttpResponse) -> Result<Vec<String>, String> {
    let v: Option<Value> = serde_json::from_str(&r.body).ok();
    if !(200..300).contains(&r.status) {
        return Err(http_error(r, v.as_ref()));
    }
    let v = v.ok_or_else(|| tr("ai.unreadable_reply"))?;
    let ids: Vec<String> = match provider {
        ImageProvider::Gemini => v
            .get("models")
            .and_then(|m| m.as_array())
            .map(|l| {
                l.iter()
                    .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
                    .map(|n| n.trim_start_matches("models/").to_string())
                    .filter(|n| n.contains("image"))
                    .collect()
            })
            .unwrap_or_default(),
        ImageProvider::OpenAi => v
            .get("data")
            .and_then(|d| d.as_array())
            .map(|l| {
                l.iter()
                    .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
                    .filter(|i| i.contains("image") || i.starts_with("dall-e"))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    };
    if ids.is_empty() {
        return Err(tr("ai.no_models"));
    }
    Ok(ids)
}

/// The pictures of a reply (as the bytes of their files), or why there
/// are none.
pub fn parse_images(provider: ImageProvider, r: &HttpResponse) -> Result<Vec<Vec<u8>>, String> {
    let v: Option<Value> = serde_json::from_str(&r.body).ok();
    if !(200..300).contains(&r.status) {
        return Err(http_error(r, v.as_ref()));
    }
    let v = v.ok_or_else(|| tr("ai.unreadable_reply"))?;
    let mut encoded: Vec<&str> = Vec::new();
    let mut said = String::new();
    match provider {
        ImageProvider::Gemini => {
            if let Some(reason) = v
                .pointer("/promptFeedback/blockReason")
                .and_then(|b| b.as_str())
            {
                return Err(trf("aiimg.refused", &[("why", reason)]));
            }
            for c in v
                .get("candidates")
                .and_then(|c| c.as_array())
                .into_iter()
                .flatten()
            {
                for p in c
                    .pointer("/content/parts")
                    .and_then(|p| p.as_array())
                    .into_iter()
                    .flatten()
                {
                    let data = p
                        .get("inlineData")
                        .or_else(|| p.get("inline_data"))
                        .and_then(|d| d.get("data"))
                        .and_then(|d| d.as_str());
                    if let Some(d) = data {
                        encoded.push(d);
                    } else if let Some(t) = p.get("text").and_then(|t| t.as_str()) {
                        said.push_str(t);
                    }
                }
            }
        }
        ImageProvider::OpenAi => {
            for d in v
                .get("data")
                .and_then(|d| d.as_array())
                .into_iter()
                .flatten()
            {
                if let Some(s) = d.get("b64_json").and_then(|s| s.as_str()) {
                    encoded.push(s);
                } else if d.get("url").is_some() {
                    return Err(tr("aiimg.url_reply"));
                }
            }
        }
    }
    let images: Vec<Vec<u8>> = encoded
        .into_iter()
        .filter_map(|s| {
            base64::engine::general_purpose::STANDARD
                .decode(s.trim())
                .ok()
        })
        .collect();
    if images.is_empty() {
        let said = said.trim();
        return Err(if said.is_empty() {
            tr("aiimg.no_image")
        } else {
            trf(
                "aiimg.no_image_said",
                &[("text", &said.chars().take(300).collect::<String>())],
            )
        });
    }
    Ok(images)
}

/// A picture file (PNG, JPEG or WebP) as PNG with its size.
pub fn to_png(bytes: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return None;
    }
    Some((crate::bitmap_fx::encode(&img)?, w, h))
}

/// Make the pure green a service painted behind the subject transparent:
/// full green goes, colours far from it stay, and edges in between keep
/// part of their opacity with the green taken out of them.
pub fn chroma_key(img: &mut image::RgbaImage) {
    const LOW: f32 = 40.0;
    const HIGH: f32 = 120.0;
    for p in img.pixels_mut() {
        let [r, g, b, a] = p.0;
        let others = r.max(b) as f32;
        let green = g as f32 - others;
        if green <= LOW {
            continue;
        }
        let keep = ((HIGH - green) / (HIGH - LOW)).clamp(0.0, 1.0);
        // Take the green spill out of what stays.
        let g2 = (g as f32).min(others).round() as u8;
        p.0 = [r, g2, b, (a as f32 * keep).round() as u8];
    }
}

/// The picture already has see-through parts.
fn has_transparency(img: &image::RgbaImage) -> bool {
    img.pixels().any(|p| p.0[3] < 250)
}

/// A picture the service made.
#[derive(Debug, Clone, PartialEq)]
pub struct GenImage {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// A reference picture: the source of Remix, an example for Create.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceImage {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// The bitmap it came from, for a remix to go over it.
    pub shape: Option<ShapeId>,
}

/// How Create vectors makes its curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VectorEngine {
    /// A flat picture from the image service, traced into curves.
    #[default]
    Trace,
    /// An SVG drawing written by the drawing service.
    Svg,
}

/// What the docker makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Create,
    Remix,
    Vector,
    /// Draw and change the drawing's objects from a request in words,
    /// through the drawing service (see `ai.rs`).
    Draw,
}

/// Where the requests stand.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ImageStatus {
    #[default]
    Idle,
    /// `done` of `total` answers are in.
    Waiting {
        done: usize,
        total: usize,
    },
    /// This many pictures were made.
    Done(usize),
    Listed(usize),
    Failed(String),
}

enum Job {
    Generate { vector: bool },
    Svg,
    Remix { source: Option<ShapeId> },
    Cutout { shape: ShapeId, chroma: bool },
    Models,
}

struct Pending {
    rx: std::sync::mpsc::Receiver<HttpResult>,
    job: Job,
}

/// The image service's state for the session.
pub struct ImageSession {
    pub mode: Mode,
    pub prompt: String,
    pub remix_prompt: String,
    pub aspect: Aspect,
    pub style: Option<usize>,
    pub palette: Option<usize>,
    pub vector_style: Option<usize>,
    pub vector_engine: VectorEngine,
    pub count: u8,
    /// The reference picture (Remix needs one; Create may use one).
    pub source: Option<SourceImage>,
    /// Pictures go into the drawing as they come (else only to Results).
    pub auto_place: bool,
    /// The settings are folded away to show the results.
    pub folded: bool,
    /// The information page shows instead of the settings.
    pub info: bool,
    pub status: ImageStatus,
    /// Newest first.
    pub results: Vec<GenImage>,
    pub models: Vec<String>,
    pending: Vec<Pending>,
    total: usize,
    made: usize,
    errors: Vec<String>,
    /// Tests: answers requests at once instead of the network.
    pub fake: Option<fn(&HttpRequest) -> HttpResult>,
    /// Tests: the requests sent, newest last.
    pub sent: Vec<HttpRequest>,
}

impl Default for ImageSession {
    fn default() -> Self {
        ImageSession {
            mode: Mode::default(),
            prompt: String::new(),
            remix_prompt: String::new(),
            aspect: Aspect::default(),
            style: None,
            palette: None,
            vector_style: None,
            vector_engine: VectorEngine::Trace,
            count: 1,
            source: None,
            auto_place: true,
            folded: false,
            info: false,
            status: ImageStatus::default(),
            results: Vec::new(),
            models: Vec::new(),
            pending: Vec::new(),
            total: 0,
            made: 0,
            errors: Vec::new(),
            fake: None,
            sent: Vec::new(),
        }
    }
}

impl ImageSession {
    pub fn busy(&self) -> bool {
        !self.pending.is_empty()
    }
}

impl App {
    /// AI features are on (Tools > Options > AI).
    pub fn ai_features_on(&self) -> bool {
        self.settings.ai_enabled
    }

    /// The image calls of the AI service, when it can make pictures.
    pub fn ai_image_prefs(&self) -> Option<ImagePrefs> {
        self.settings.ai.image_prefs()
    }

    /// The AI service makes pictures (it can, has a model and its key
    /// where one is needed) and AI features are on.
    pub fn ai_image_ready(&self) -> bool {
        let Some(p) = self.ai_image_prefs() else {
            return false;
        };
        self.ai_features_on()
            && !p.model().is_empty()
            && (!p.needs_key() || !self.ai.key.trim().is_empty())
    }

    /// Why the picture commands are greyed, for their hover tips: AI
    /// turned off, a service that makes drawings only, or one not set up.
    /// None when they can run.
    pub fn ai_image_tip(&self) -> Option<String> {
        if !self.ai_features_on() {
            Some(tr("aiimg.off"))
        } else if !self.settings.ai.can_make_pictures() {
            Some(tr("aiui.drawings_only_tip"))
        } else if !self.ai_image_ready() {
            Some(tr("aiui.set_up_tip"))
        } else {
            None
        }
    }

    /// The drawing service (AI Drawing) is set up and AI features are on.
    pub fn ai_drawing_ready(&self) -> bool {
        self.ai_features_on() && self.ai_ready()
    }

    /// The one bitmap selected, for Remix and Remove Background.
    pub fn ai_source_bitmap(&self) -> Option<(ShapeId, tracedraw_core::SharedBytes)> {
        let sel = self.selected_shapes();
        let [s] = sel.as_slice() else {
            return None;
        };
        match &s.kind {
            ShapeKind::Bitmap { png, .. } => Some((s.id, png.clone())),
            _ => None,
        }
    }

    fn ai_image_problem(&self) -> Option<String> {
        if !self.ai_features_on() {
            Some(tr("aiimg.off"))
        } else if !self.settings.ai.can_make_pictures() {
            Some(tr("aiui.drawings_only_tip"))
        } else if !self.ai_image_ready() {
            Some(tr("aiimg.not_set_up"))
        } else if !self.has_document() {
            Some(tr("ai.need_document"))
        } else {
            None
        }
    }

    fn ai_image_send(&mut self, http: HttpRequest, job: Job) {
        let (tx, rx) = std::sync::mpsc::channel();
        match self.ai_image.fake {
            Some(fake) => {
                let _ = tx.send(fake(&http));
            }
            None => crate::net::spawn(http.clone(), MAX_REPLY, tx),
        }
        self.ai_image.sent.push(http);
        if self.ai_image.sent.len() > 8 {
            self.ai_image.sent.remove(0);
        }
        self.ai_image.pending.push(Pending { rx, job });
    }

    fn ai_image_begin(&mut self, total: usize) {
        self.ai_image.total = total;
        self.ai_image.made = 0;
        self.ai_image.errors.clear();
        self.ai_image.status = ImageStatus::Waiting { done: 0, total };
    }

    /// The docker's Generate, for its mode.
    pub fn ai_generate(&mut self) {
        match self.ai_image.mode {
            Mode::Create => self.ai_create(false),
            Mode::Vector => self.ai_create(true),
            Mode::Remix => self.ai_remix(),
            Mode::Draw => self.ai_draw(),
        }
    }

    /// The reference picture from the selection: the selected bitmap as
    /// it is, or the selected objects drawn as a picture.
    pub fn ai_source_from_selection(&mut self) {
        if let Some((id, png)) = self.ai_source_bitmap() {
            if let Some(img) = crate::bitmap_fx::decode(&png) {
                self.ai_image.source = Some(SourceImage {
                    png: png.to_vec(),
                    width: img.width(),
                    height: img.height(),
                    shape: Some(id),
                });
                return;
            }
        }
        // Objects of any kind: drawn at a size the services take.
        let Some(b) = self.selection_bounds() else {
            self.ai_image.status = ImageStatus::Failed(tr("aiimg.need_selection"));
            return;
        };
        let longer_in = b.width().max(b.height()).max(0.1) / 25.4;
        let dpi = (1024.0 / longer_in).clamp(10.0, 1200.0);
        match self.render_selection_png(dpi, false) {
            Some((png, w, h, _)) => {
                self.ai_image.source = Some(SourceImage {
                    png,
                    width: w,
                    height: h,
                    shape: None,
                })
            }
            None => self.ai_image.status = ImageStatus::Failed(tr("aiimg.need_selection")),
        }
    }

    /// The reference picture from a file's bytes.
    pub fn ai_source_from_bytes(&mut self, bytes: &[u8]) {
        match to_png(bytes) {
            Some((png, width, height)) => {
                self.ai_image.source = Some(SourceImage {
                    png,
                    width,
                    height,
                    shape: None,
                })
            }
            None => self.ai_image.status = ImageStatus::Failed(tr("aiimg.unreadable_image")),
        }
    }

    /// Pick a picture file as the reference picture.
    pub fn ai_source_from_file(&mut self) {
        crate::files::Dialog::new()
            .add_filter(
                tr("aiimg.picture_files"),
                &["png", "jpg", "jpeg", "webp", "bmp", "gif", "tif", "tiff"],
            )
            .pick_file(self, |app, path| match crate::files::read(&path) {
                Ok(bytes) => app.ai_source_from_bytes(&bytes),
                Err(e) => app.ai_image.status = ImageStatus::Failed(e.to_string()),
            });
    }

    /// The aspect ratio asked for: the reference picture's when matching.
    fn ai_aspect(&self) -> Aspect {
        match (self.ai_image.aspect, &self.ai_image.source) {
            (Aspect::MatchSource, Some(src)) => Aspect::nearest(src.width, src.height),
            (Aspect::MatchSource, None) => Aspect::Square,
            (a, _) => a,
        }
    }

    /// Create image (or vectors): pictures from the prompt, with the
    /// reference picture when there is one, placed on the page.
    pub fn ai_create(&mut self, vector: bool) {
        if self.ai_image.busy() {
            return;
        }
        let words = self.ai_image.prompt.trim().to_string();
        if vector && self.ai_image.vector_engine == VectorEngine::Svg {
            self.ai_create_svg(words);
            return;
        }
        if let Some(p) = self.ai_image_problem() {
            self.ai_image.status = ImageStatus::Failed(p);
            return;
        }
        if words.is_empty() {
            self.ai_image.status = ImageStatus::Failed(tr("aiimg.need_prompt"));
            return;
        }
        let s = &self.ai_image;
        let style = if vector { None } else { s.style };
        let mut prompt = compose_prompt(&words, style, s.palette, vector);
        if let Some((_, w)) = s
            .vector_style
            .filter(|_| vector)
            .and_then(|i| VECTOR_STYLES.get(i))
        {
            prompt.push_str("\n\nStyle: ");
            prompt.push_str(w);
        }
        if s.source.is_some() {
            prompt = format!("Use the attached picture as the reference.\n\n{prompt}");
        }
        let count = s.count.clamp(1, MAX_COUNT);
        let Some(prefs) = self.ai_image_prefs() else {
            return;
        };
        let key = self.ai.key.clone();
        let aspect = self.ai_aspect();
        let reference = self.ai_image.source.as_ref().map(|s| s.png.clone());
        // Gemini answers one picture a request, and so do edits.
        let requests = match (prefs.provider, &reference) {
            (ImageProvider::Gemini, _) | (_, Some(_)) => count as usize,
            (ImageProvider::OpenAi, None) => 1,
        };
        self.ai_image_begin(requests);
        self.ai_image.results.clear();
        for _ in 0..requests {
            let http = match &reference {
                Some(png) => build_reference(&prefs, &key, &prompt, png, aspect),
                None => build_generate(&prefs, &key, &prompt, aspect, count),
            };
            self.ai_image_send(http, Job::Generate { vector });
        }
        self.poll_ai_image();
    }

    /// Create vectors with the drawing service: an SVG drawing imported
    /// as editable shapes.
    fn ai_create_svg(&mut self, words: String) {
        let problem = if !self.ai_features_on() {
            Some(tr("aiimg.off"))
        } else if !self.ai_drawing_ready() {
            Some(tr("aiimg.svg_needs_drawing"))
        } else if !self.has_document() {
            Some(tr("ai.need_document"))
        } else if words.is_empty() {
            Some(tr("aiimg.need_prompt"))
        } else {
            None
        };
        if let Some(p) = problem {
            self.ai_image.status = ImageStatus::Failed(p);
            return;
        }
        let mut request = format!("Draw: {words}");
        if let Some((_, w)) = self
            .ai_image
            .vector_style
            .and_then(|i| VECTOR_STYLES.get(i))
        {
            request.push_str("\nStyle: ");
            request.push_str(w);
        }
        if let Some((_, w, _)) = self.ai_image.palette.and_then(|i| PALETTES.get(i)) {
            request.push_str("\nColours: ");
            request.push_str(w);
        }
        let a = self.ai_aspect();
        request.push_str(&format!("\nAspect ratio: {}", a.ratio()));
        let http = crate::ai::build_request_with(
            &self.settings.ai,
            &self.ai.key,
            SVG_PROMPT,
            &[(crate::ai::Role::User, request)],
        );
        self.ai_image_begin(1);
        self.ai_image.results.clear();
        self.ai_image_send(http, Job::Svg);
        self.poll_ai_image();
    }

    /// Remix image: the reference picture (or the selected bitmap) changed
    /// as the prompt says; the result goes over the bitmap it came from.
    pub fn ai_remix(&mut self) {
        if self.ai_image.busy() {
            return;
        }
        if let Some(p) = self.ai_image_problem() {
            self.ai_image.status = ImageStatus::Failed(p);
            return;
        }
        if self.ai_image.source.is_none() {
            self.ai_source_from_selection();
            if self.ai_image.source.is_none() {
                self.ai_image.status = ImageStatus::Failed(tr("aiimg.need_bitmap"));
                return;
            }
        }
        let Some(src) = self.ai_image.source.clone() else {
            return;
        };
        let words = self.ai_image.remix_prompt.trim().to_string();
        if words.is_empty() {
            self.ai_image.status = ImageStatus::Failed(tr("aiimg.need_prompt"));
            return;
        }
        let prompt = compose_prompt(&words, self.ai_image.style, self.ai_image.palette, false);
        let count = self.ai_image.count.clamp(1, MAX_COUNT) as usize;
        let Some(prefs) = self.ai_image_prefs() else {
            return;
        };
        self.ai_image_begin(count);
        self.ai_image.results.clear();
        for _ in 0..count {
            let http = build_edit(&prefs, &self.ai.key, &prompt, &src.png, false);
            self.ai_image_send(http, Job::Remix { source: src.shape });
        }
        self.poll_ai_image();
    }

    /// Image > AI Art Styles: remix the selection with the request of
    /// style `i` (the AI Generate docker shows it and the results).
    pub fn ai_art_style(&mut self, i: usize) {
        let Some((_, words)) = ART_STYLES.get(i) else {
            return;
        };
        self.ai_image.mode = Mode::Remix;
        self.ai_image.info = false;
        self.ai_image.remix_prompt = (*words).to_string();
        self.ai_image.style = None;
        self.ai_image.palette = None;
        // The picture picked now, not one chosen before.
        self.ai_image.source = None;
        self.open_ai_generate();
        self.ai_remix();
    }

    /// Image > Remove Background: the selected bitmap's subject kept,
    /// the rest made transparent, as one undo step.
    pub fn ai_remove_background(&mut self) {
        if self.ai_image.busy() {
            return;
        }
        if let Some(p) = self.ai_image_problem() {
            self.status = p.clone();
            self.ai_image.status = ImageStatus::Failed(p);
            return;
        }
        let Some((shape, png)) = self.ai_source_bitmap() else {
            self.status = tr("aiimg.need_bitmap");
            return;
        };
        let Some(prefs) = self.ai_image_prefs() else {
            return;
        };
        let chroma = prefs.provider == ImageProvider::Gemini;
        let prompt = if chroma { CHROMA_PROMPT } else { CUTOUT_PROMPT };
        let http = build_edit(&prefs, &self.ai.key, prompt, &png, !chroma);
        self.ai_image_begin(1);
        self.status = tr("aiimg.removing_background");
        self.ai_image_send(http, Job::Cutout { shape, chroma });
        self.poll_ai_image();
    }

    /// Fetch the service's image models (nothing when it makes drawings
    /// only).
    pub fn ai_image_fetch_models(&mut self) {
        if self.ai_image.busy() {
            return;
        }
        let Some(prefs) = self.ai_image_prefs() else {
            return;
        };
        if prefs.needs_key() && self.ai.key.trim().is_empty() {
            self.ai_image.status = ImageStatus::Failed(tr("ai.need_key"));
            return;
        }
        let http = build_models(&prefs, &self.ai.key);
        self.ai_image_begin(1);
        self.ai_image_send(http, Job::Models);
        self.poll_ai_image();
    }

    /// Stop waiting (answers are dropped when they come).
    pub fn ai_image_cancel(&mut self) {
        if !self.ai_image.pending.is_empty() {
            self.ai_image.pending.clear();
            self.ai_image.status = ImageStatus::Idle;
        }
    }

    /// Take in answers that arrived. True while some are still on their
    /// way.
    pub fn poll_ai_image(&mut self) -> bool {
        let mut i = 0;
        while i < self.ai_image.pending.len() {
            let got = match self.ai_image.pending[i].rx.try_recv() {
                Ok(r) => Some(r),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Some(Err(tr("ai.connection_lost")))
                }
            };
            match got {
                Some(result) => {
                    let p = self.ai_image.pending.remove(i);
                    self.ai_image_receive(result, p.job);
                }
                None => i += 1,
            }
        }
        self.ai_image.busy()
    }

    fn ai_image_receive(&mut self, result: HttpResult, job: Job) {
        let provider = self
            .ai_image_prefs()
            .map(|p| p.provider)
            .unwrap_or_default();
        let done = self.ai_image.total - self.ai_image.pending.len();
        if let Job::Models = job {
            match result.and_then(|r| parse_models(provider, &r)) {
                Ok(m) => {
                    self.ai_image.status = ImageStatus::Listed(m.len());
                    self.ai_image.models = m;
                }
                Err(e) => self.ai_image.status = ImageStatus::Failed(e),
            }
            return;
        }
        if let Job::Svg = job {
            let svg = result
                .and_then(|r| crate::ai::parse_reply(self.settings.ai.provider, &r))
                .and_then(|reply| extract_svg(&reply).ok_or_else(|| tr("aiimg.no_svg")));
            match svg {
                Ok(svg) => self.ai_place_svg(&svg),
                Err(e) => self.ai_image.errors.push(e),
            }
            self.ai_image_finish(done);
            return;
        }
        let images = result
            .and_then(|r| parse_images(provider, &r))
            .and_then(|files| {
                let pngs: Vec<GenImage> = files
                    .iter()
                    .filter_map(|f| to_png(f))
                    .map(|(png, width, height)| GenImage { png, width, height })
                    .collect();
                if pngs.is_empty() {
                    Err(tr("aiimg.unreadable_image"))
                } else {
                    Ok(pngs)
                }
            });
        match images {
            Err(e) => self.ai_image.errors.push(e),
            Ok(images) => match job {
                Job::Generate { vector } => {
                    for img in images {
                        if self.ai_image.auto_place || vector {
                            self.ai_place_image(&img, None, vector);
                        } else {
                            self.ai_image.made += 1;
                        }
                        self.ai_keep_result(img);
                    }
                }
                Job::Remix { source } => {
                    if let Some(img) = images.into_iter().next() {
                        if self.ai_image.auto_place {
                            self.ai_place_image(&img, source, false);
                        } else {
                            self.ai_image.made += 1;
                        }
                        self.ai_keep_result(img);
                    }
                }
                Job::Cutout { shape, chroma } => {
                    if let Some(img) = images.into_iter().next() {
                        self.ai_apply_cutout(shape, img, chroma);
                    }
                }
                Job::Models | Job::Svg => {}
            },
        }
        self.ai_image_finish(done);
    }

    fn ai_image_finish(&mut self, done: usize) {
        self.ai_image.status = if self.ai_image.busy() {
            ImageStatus::Waiting {
                done,
                total: self.ai_image.total,
            }
        } else if self.ai_image.made == 0 {
            ImageStatus::Failed(
                self.ai_image
                    .errors
                    .first()
                    .cloned()
                    .unwrap_or_else(|| tr("aiimg.no_image")),
            )
        } else {
            ImageStatus::Done(self.ai_image.made)
        };
    }

    fn ai_keep_result(&mut self, img: GenImage) {
        self.ai_image.results.push(img);
        self.ai_image.results.truncate(MAX_RESULTS);
    }

    /// An SVG drawing as one group of editable shapes, centred on the page
    /// within four fifths of it, as one undo step.
    pub fn ai_place_svg(&mut self, svg: &str) {
        use tracedraw_core::geometry::Shape as _;
        let Some(layer) = self.active_layer() else {
            return;
        };
        let parsed =
            tracedraw_io::svg_import::parse(svg, &mut tracedraw_core::id::IdSource::default());
        let shapes = match parsed {
            Ok(p) if !p.shapes.is_empty() => p.shapes,
            Ok(_) => {
                self.ai_image.errors.push(tr("aiimg.no_svg"));
                return;
            }
            Err(e) => {
                self.ai_image
                    .errors
                    .push(trf("aiimg.bad_svg", &[("e", &e)]));
                return;
            }
        };
        let gid = self.engine.new_shape_id();
        let children: Vec<Shape> = shapes
            .into_iter()
            .map(|s| {
                let id = self.engine.new_shape_id();
                crate::app::reid_pub(s, id, &mut self.engine)
            })
            .collect();
        let mut group = Shape::new(gid, ShapeKind::Group { children });
        group.name = Some(tr("aiimg.vector_name"));
        let b = group.page_path().bounding_box();
        if b.width() > 1e-9 && b.height() > 1e-9 {
            let page = self.page_size();
            let k = (page.width * 0.8 / b.width()).min(page.height * 0.8 / b.height());
            let c = b.center();
            group.transform = tracedraw_core::Affine::translate(tracedraw_core::Vec2::new(
                page.width / 2.0,
                page.height / 2.0,
            )) * tracedraw_core::Affine::scale(k)
                * tracedraw_core::Affine::translate(-c.to_vec2());
        }
        if let Err(e) = self.engine.run_batch(
            LABEL_GENERATE,
            &[Command::AddShape {
                layer,
                shape: group,
            }],
        ) {
            self.ai_image.errors.push(e.to_string());
            return;
        }
        self.ai_image.made += 1;
        self.select(vec![gid]);
        self.status = trf(
            "status.ai_image_placed",
            &[("n", &self.ai_image.made.to_string())],
        );
    }

    /// Put a picture on the page: over `over` (a remix, same place and
    /// size) or centred on the page within four fifths of it. `vector`
    /// traces it into shapes.
    pub fn ai_place_image(&mut self, img: &GenImage, over: Option<ShapeId>, vector: bool) {
        let Some(layer) = self.active_layer() else {
            return;
        };
        let (rect, transform) = match over.and_then(|id| self.doc().find_shape(id).cloned()) {
            Some(src) => match &src.kind {
                ShapeKind::Bitmap { rect, .. } => (*rect, src.transform),
                _ => (Rect::ZERO, tracedraw_core::Affine::IDENTITY),
            },
            None => (Rect::ZERO, tracedraw_core::Affine::IDENTITY),
        };
        let rect = if rect.area() > 0.0 {
            // A remix keeps the source's height and its own proportions.
            let w = rect.height() * img.width as f64 / img.height.max(1) as f64;
            Rect::new(rect.x0, rect.y0, rect.x0 + w, rect.y1)
        } else {
            let page = self.page_size();
            let scale =
                (page.width * 0.8 / img.width as f64).min(page.height * 0.8 / img.height as f64);
            let (w, h) = (img.width as f64 * scale, img.height as f64 * scale);
            let (x, y) = ((page.width - w) / 2.0, (page.height - h) / 2.0);
            Rect::new(x, y, x + w, y + h)
        };
        let id = self.engine.new_shape_id();
        let mut shape = Shape::new(
            id,
            ShapeKind::Bitmap {
                rect,
                width_px: img.width,
                height_px: img.height,
                png: img.png.as_slice().into(),
                fx: None,
            },
        );
        shape.transform = transform;
        shape.fill = tracedraw_core::Fill::None;
        shape.stroke = None;
        shape.name = Some(tr("aiimg.object_name"));
        if let Err(e) = self
            .engine
            .run_batch(LABEL_GENERATE, &[Command::AddShape { layer, shape }])
        {
            self.ai_image.errors.push(e.to_string());
            return;
        }
        self.ai_image.made += 1;
        self.select(vec![id]);
        if vector {
            let mut settings = self
                .settings
                .tracing
                .settings(crate::trace::Preset::Clipart);
            settings.delete_original = true;
            self.trace_bitmaps(&settings);
            let traced = self.selection.clone();
            let mut cmds = Vec::new();
            for id in traced {
                cmds.push(Command::SetShapeName {
                    shape: id,
                    name: Some(tr("aiimg.vector_name")),
                });
            }
            let _ = self.engine.run_batch(LABEL_GENERATE, &cmds);
        }
        self.status = trf(
            "status.ai_image_placed",
            &[("n", &self.ai_image.made.to_string())],
        );
    }

    fn ai_apply_cutout(&mut self, shape: ShapeId, img: GenImage, chroma: bool) {
        let Some(src) = self.doc().find_shape(shape).cloned() else {
            self.ai_image.errors.push(tr("aiimg.source_gone"));
            return;
        };
        let ShapeKind::Bitmap { rect, .. } = &src.kind else {
            self.ai_image.errors.push(tr("aiimg.source_gone"));
            return;
        };
        let Some(mut out) = crate::bitmap_fx::decode(&img.png) else {
            self.ai_image.errors.push(tr("aiimg.unreadable_image"));
            return;
        };
        if chroma || !has_transparency(&out) {
            chroma_key(&mut out);
        }
        let Some(png) = crate::bitmap_fx::encode(&out) else {
            return;
        };
        let cmd = Command::SetShapeKind {
            shape,
            kind: ShapeKind::Bitmap {
                rect: *rect,
                width_px: out.width(),
                height_px: out.height(),
                png: png.into(),
                fx: None,
            },
        };
        match self.engine.run_batch(LABEL_CUTOUT, &[cmd]) {
            Ok(_) => {
                self.ai_image.made += 1;
                self.status = tr("status.background_removed");
            }
            Err(e) => self.ai_image.errors.push(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn png(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
        let img = RgbaImage::from_fn(w, h, |x, y| Rgba(f(x, y)));
        crate::bitmap_fx::encode(&img).expect("png")
    }

    fn gemini_reply(png: &[u8]) -> HttpResult {
        Ok(HttpResponse {
            status: 200,
            body: json!({"candidates": [{"content": {"parts": [
                {"text": "Here it is"},
                {"inlineData": {"mimeType": "image/png", "data": b64(png)}}
            ]}}]})
            .to_string(),
        })
    }

    /// The one AI service at the preset of `provider` (Gemini or OpenAI),
    /// with a key.
    fn ready_app(provider: ImageProvider) -> App {
        let mut app = App::headless();
        app.settings.ai_enabled = true;
        let preset = match provider {
            ImageProvider::Gemini => "aipreset.gemini",
            ImageProvider::OpenAi => "aipreset.openai",
        };
        let (_, kind, endpoint) = crate::ai::PRESETS
            .iter()
            .find(|(k, _, _)| *k == preset)
            .copied()
            .expect("preset");
        app.settings.ai.provider = kind;
        app.settings.ai.endpoint = endpoint.to_string();
        app.ai.key = "k".into();
        app
    }

    #[test]
    fn pictures_come_from_the_one_service_when_it_can_make_them() {
        let mut app = ready_app(ImageProvider::Gemini);
        let p = app.ai_image_prefs().expect("gemini makes pictures");
        assert_eq!(p.provider, ImageProvider::Gemini);
        assert_eq!(p.model(), "gemini-2.5-flash-image");
        // Gemini's chat address is its OpenAI-compatible one; pictures go
        // to its own API.
        let r = build_generate(&p, &app.ai.key, "a cat", Aspect::Square, 1);
        assert!(r.url.starts_with(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-image"
        ));
        assert!(app.ai_image_ready() && app.ai_image_tip().is_none());
        let mut app2 = ready_app(ImageProvider::OpenAi);
        let p = app2.ai_image_prefs().expect("openai makes pictures");
        assert_eq!(
            (p.provider, p.model()),
            (ImageProvider::OpenAi, "gpt-image-1".into())
        );
        assert_eq!(p.endpoint(), "https://api.openai.com/v1");
        app2.settings.ai.image_model = "dall-e-3".into();
        assert_eq!(
            app2.ai_image_prefs().map(|p| p.model()),
            Some("dall-e-3".into())
        );
        // Anthropic and the local chat-only servers make drawings only.
        app.settings.ai = crate::ai::AiPrefs::default();
        assert!(app.ai_image_prefs().is_none() && !app.ai_image_ready());
        assert_eq!(app.ai_image_tip(), Some(tr("aiui.drawings_only_tip")));
        for key in [
            "aipreset.ollama",
            "aipreset.lm_studio",
            "aipreset.llama_cpp",
            "aipreset.vllm",
        ] {
            let (_, kind, endpoint) = crate::ai::PRESETS
                .iter()
                .find(|(k, _, _)| *k == key)
                .copied()
                .expect("preset");
            app.settings.ai.provider = kind;
            app.settings.ai.endpoint = endpoint.to_string();
            app.settings.ai.image_model = "anything".into();
            assert!(!app.settings.ai.can_make_pictures(), "{key}");
            assert!(app.ai_image_prefs().is_none(), "{key}");
        }
        // An address of the user's: pictures at the same base when an
        // image model is given, and no key needed.
        app.settings.ai.endpoint = "http://my-box:9000/v1/chat/completions".into();
        app.settings.ai.image_model.clear();
        assert!(app.settings.ai.can_make_pictures());
        assert!(
            app.ai_image_prefs().is_none(),
            "no image model: no pictures"
        );
        assert_eq!(app.ai_image_tip(), Some(tr("aiui.set_up_tip")));
        app.settings.ai.image_model = "sd-xl".into();
        app.ai.key.clear();
        let p = app.ai_image_prefs().expect("custom");
        assert_eq!(p.endpoint(), "http://my-box:9000/v1");
        assert!(app.ai_image_ready());
        // AI turned off greys everything.
        app.settings.ai_enabled = false;
        assert_eq!(app.ai_image_tip(), Some(tr("aiimg.off")));
    }

    #[test]
    fn requests_are_built_for_both_kinds_of_service() {
        let mut p = ImagePrefs::default();
        let r = build_generate(&p, "key1", "a cat", Aspect::Wide169, 2);
        assert_eq!(
            r.url,
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-image:generateContent"
        );
        assert!(r
            .headers
            .contains(&("x-goog-api-key".into(), "key1".into())));
        let v: Value = serde_json::from_str(&r.body).expect("json");
        assert_eq!(
            v.pointer("/contents/0/parts/0/text")
                .and_then(|t| t.as_str()),
            Some("a cat")
        );
        assert_eq!(
            v.pointer("/generationConfig/imageConfig/aspectRatio")
                .and_then(|t| t.as_str()),
            Some("16:9")
        );
        let e = build_edit(&p, "key1", "make it blue", b"PNG", false);
        let v: Value = serde_json::from_str(&e.body).expect("json");
        assert_eq!(
            v.pointer("/contents/0/parts/0/inline_data/data")
                .and_then(|t| t.as_str()),
            Some(b64(b"PNG").as_str())
        );
        p.provider = ImageProvider::OpenAi;
        let r = build_generate(&p, "key2", "a dog", Aspect::Portrait23, 9);
        assert_eq!(r.url, "https://api.openai.com/v1/images/generations");
        assert!(r
            .headers
            .contains(&("authorization".into(), "Bearer key2".into())));
        let v: Value = serde_json::from_str(&r.body).expect("json");
        assert_eq!(v["n"], 4, "at most four at a time");
        assert_eq!(v["size"], "1024x1536");
        assert!(v.get("response_format").is_none());
        p.model = "dall-e-3".into();
        let v: Value = serde_json::from_str(&build_generate(&p, "", "x", Aspect::Wide169, 1).body)
            .expect("json");
        assert_eq!(
            (v["size"].as_str(), v["response_format"].as_str()),
            (Some("1792x1024"), Some("b64_json"))
        );
        p.model.clear();
        let e = build_edit(&p, "key2", "cut", b"\x89PNGDATA", true);
        assert_eq!(e.url, "https://api.openai.com/v1/images/edits");
        assert!(e.body.is_empty() && !e.bytes.is_empty());
        let text = String::from_utf8_lossy(&e.bytes);
        assert!(text.contains("name=\"background\"\r\n\r\ntransparent"));
        assert!(text.contains("filename=\"image.png\"") && text.contains("PNGDATA"));
        assert!(e
            .headers
            .iter()
            .any(|(k, v)| k == "content-type" && v.starts_with("multipart/form-data; boundary=")));
        // A service at the user's own address may take no key.
        p.endpoint = "http://localhost:8080/v1/".into();
        assert!(!p.needs_key());
        let r = build_generate(&p, "", "x", Aspect::Square, 1);
        assert_eq!(r.url, "http://localhost:8080/v1/images/generations");
        assert!(!r.headers.iter().any(|(k, _)| k == "authorization"));
    }

    #[test]
    fn prompts_carry_style_palette_and_vector_words() {
        let p = compose_prompt("  a fox ", Some(2), Some(1), false);
        assert!(p.starts_with("a fox") && p.contains("Watercolour") && p.contains("grey"));
        let v = compose_prompt("a fox", Some(0), None, true);
        assert!(v.starts_with("Flat vector") && !v.contains("Photorealistic"));
        assert_eq!(
            compose_prompt(&"x".repeat(9000), None, None, false)
                .chars()
                .count(),
            MAX_PROMPT
        );
    }

    #[test]
    fn replies_and_errors_are_read() {
        let one = png(2, 2, |_, _| [1, 2, 3, 255]);
        let r = gemini_reply(&one).expect("reply");
        assert_eq!(
            parse_images(ImageProvider::Gemini, &r).expect("images"),
            vec![one.clone()]
        );
        let r = HttpResponse {
            status: 200,
            body: json!({"data": [{"b64_json": b64(&one)}, {"b64_json": b64(&one)}]}).to_string(),
        };
        assert_eq!(
            parse_images(ImageProvider::OpenAi, &r)
                .expect("images")
                .len(),
            2
        );
        let bad = HttpResponse {
            status: 401,
            body: json!({"error": {"message": "Incorrect API key"}}).to_string(),
        };
        assert!(parse_images(ImageProvider::OpenAi, &bad)
            .unwrap_err()
            .contains("Incorrect API key"));
        let blocked = HttpResponse {
            status: 200,
            body: json!({"promptFeedback": {"blockReason": "SAFETY"}}).to_string(),
        };
        assert!(parse_images(ImageProvider::Gemini, &blocked)
            .unwrap_err()
            .contains("SAFETY"));
        let words = HttpResponse {
            status: 200,
            body: json!({"candidates": [{"content": {"parts": [{"text": "I cannot draw that"}]}}]})
                .to_string(),
        };
        assert!(parse_images(ImageProvider::Gemini, &words)
            .unwrap_err()
            .contains("I cannot draw that"));
        let url = HttpResponse {
            status: 200,
            body: json!({"data": [{"url": "https://x"}]}).to_string(),
        };
        assert!(parse_images(ImageProvider::OpenAi, &url).is_err());
        let junk = HttpResponse {
            status: 200,
            body: "not json".into(),
        };
        assert!(parse_images(ImageProvider::Gemini, &junk).is_err());
        assert!(to_png(b"not a picture").is_none());
        let models = HttpResponse {
            status: 200,
            body: json!({"models": [{"name": "models/gemini-2.5-flash"}, {"name": "models/gemini-2.5-flash-image"}]}).to_string(),
        };
        assert_eq!(
            parse_models(ImageProvider::Gemini, &models).expect("models"),
            vec!["gemini-2.5-flash-image"]
        );
    }

    #[test]
    fn green_is_keyed_out_and_edges_keep_part_of_their_opacity() {
        let mut img = RgbaImage::from_fn(4, 1, |x, _| {
            Rgba(match x {
                0 => [0, 255, 0, 255],
                1 => [200, 30, 40, 255],
                2 => [90, 170, 90, 255],
                _ => [250, 250, 250, 255],
            })
        });
        chroma_key(&mut img);
        assert_eq!(img.get_pixel(0, 0).0[3], 0);
        assert_eq!(img.get_pixel(1, 0).0, [200, 30, 40, 255]);
        let edge = img.get_pixel(2, 0).0;
        assert!(edge[3] > 0 && edge[3] < 255, "{edge:?}");
        assert!(edge[1] <= 90, "green spill taken out: {edge:?}");
        assert_eq!(img.get_pixel(3, 0).0[3], 255);
    }

    #[test]
    fn nothing_is_sent_while_ai_is_off_or_not_set_up() {
        let mut app = App::headless();
        app.ai_image.fake = Some(|_| panic!("nothing may be sent"));
        app.ai_image.prompt = "a cat".into();
        // The usual service makes drawings only.
        app.ai.key = "k".into();
        app.ai_create(false);
        assert_eq!(
            app.ai_image.status,
            ImageStatus::Failed(tr("aiui.drawings_only_tip"))
        );
        let mut app = ready_app(ImageProvider::Gemini);
        app.ai_image.fake = Some(|_| panic!("nothing may be sent"));
        app.ai_image.prompt = "a cat".into();
        app.settings.ai_enabled = false;
        app.ai_create(false);
        assert_eq!(app.ai_image.status, ImageStatus::Failed(tr("aiimg.off")));
        app.settings.ai_enabled = true;
        app.ai.key.clear();
        assert!(!app.ai_image_ready());
        app.ai_create(false);
        assert_eq!(
            app.ai_image.status,
            ImageStatus::Failed(tr("aiimg.not_set_up"))
        );
        app.ai.key = "k".into();
        app.ai_image.prompt.clear();
        app.ai_create(false);
        assert_eq!(
            app.ai_image.status,
            ImageStatus::Failed(tr("aiimg.need_prompt"))
        );
        app.ai_remove_background();
        assert!(app.ai_image.sent.is_empty());
    }

    fn count_bitmaps(app: &App) -> usize {
        app.doc()
            .all_layers()
            .flat_map(|l| l.shapes.iter())
            .filter(|s| matches!(s.kind, ShapeKind::Bitmap { .. }))
            .count()
    }

    #[test]
    fn created_pictures_go_on_the_page_one_undo_step_each() {
        let mut app = ready_app(ImageProvider::Gemini);
        app.ai_image.fake = Some(|_| gemini_reply(&png(40, 20, |_, _| [200, 10, 10, 255])));
        app.ai_image.prompt = "a red banner".into();
        app.ai_image.count = 2;
        app.ai_create(false);
        assert_eq!(app.ai_image.sent.len(), 2, "Gemini: one request a picture");
        assert_eq!(app.ai_image.status, ImageStatus::Done(2));
        assert_eq!(count_bitmaps(&app), 2);
        assert_eq!(app.ai_image.results.len(), 2);
        // Centred on the page within four fifths of its width.
        let s = app.selected_shapes().pop().expect("selected");
        let ShapeKind::Bitmap { rect, .. } = s.kind else {
            panic!()
        };
        let page = app.page_size();
        assert!((rect.width() - page.width * 0.8).abs() < 1e-6);
        assert!(((rect.x0 + rect.x1) / 2.0 - page.width / 2.0).abs() < 1e-6);
        app.undo();
        assert_eq!(count_bitmaps(&app), 1);
    }

    #[test]
    fn vectors_are_traced_from_the_picture() {
        let mut app = ready_app(ImageProvider::Gemini);
        app.ai_image.fake = Some(|_| {
            gemini_reply(&png(64, 64, |x, y| {
                if (x as i32 - 32).pow(2) + (y as i32 - 32).pow(2) < 400 {
                    [20, 60, 200, 255]
                } else {
                    [255, 255, 255, 255]
                }
            }))
        });
        app.ai_image.mode = Mode::Vector;
        app.ai_image.prompt = "a blue dot".into();
        app.ai_generate();
        assert_eq!(app.ai_image.status, ImageStatus::Done(1));
        assert_eq!(
            count_bitmaps(&app),
            0,
            "the picture is replaced by its trace"
        );
        let groups = app
            .doc()
            .all_layers()
            .flat_map(|l| l.shapes.iter())
            .filter(|s| matches!(s.kind, ShapeKind::Group { .. }))
            .count();
        assert_eq!(groups, 1);
    }

    #[test]
    fn remix_and_remove_background_work_on_the_selected_bitmap() {
        let mut app = ready_app(ImageProvider::Gemini);
        let src = png(8, 8, |_, _| [255, 255, 255, 255]);
        let id = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(10.0, 10.0, 50.0, 50.0),
                width_px: 8,
                height_px: 8,
                png: src.into(),
                fx: None,
            })
            .expect("bitmap");
        app.select(vec![id]);
        // Remove Background through a service that paints the back green.
        app.ai_image.fake = Some(|_| {
            gemini_reply(&png(8, 8, |x, _| {
                if x < 4 {
                    [0, 255, 0, 255]
                } else {
                    [180, 40, 40, 255]
                }
            }))
        });
        app.ai_remove_background();
        assert_eq!(app.ai_image.status, ImageStatus::Done(1));
        let body: Value = serde_json::from_str(&app.ai_image.sent[0].body).expect("json");
        assert!(body.to_string().contains("#00FF00"));
        let s = app.doc().find_shape(id).cloned().expect("shape");
        let ShapeKind::Bitmap { png: out, rect, .. } = &s.kind else {
            panic!()
        };
        assert_eq!(*rect, Rect::new(10.0, 10.0, 50.0, 50.0), "stays in place");
        let img = crate::bitmap_fx::decode(out).expect("png");
        assert_eq!(img.get_pixel(0, 0).0[3], 0);
        assert_eq!(img.get_pixel(7, 0).0[3], 255);
        app.undo();
        let s = app.doc().find_shape(id).cloned().expect("shape");
        let ShapeKind::Bitmap { png: back, .. } = &s.kind else {
            panic!()
        };
        assert_eq!(
            crate::bitmap_fx::decode(back)
                .expect("png")
                .get_pixel(0, 0)
                .0,
            [255, 255, 255, 255]
        );
        // Remix adds the changed picture over the source.
        app.select(vec![id]);
        app.ai_image.mode = Mode::Remix;
        app.ai_image.remix_prompt = "make it red".into();
        app.ai_image.fake = Some(|_| gemini_reply(&png(16, 8, |_, _| [255, 0, 0, 255])));
        app.ai_generate();
        assert_eq!(count_bitmaps(&app), 2);
        let s = app.selected_shapes().pop().expect("remix");
        let ShapeKind::Bitmap { rect, .. } = s.kind else {
            panic!()
        };
        assert!((rect.height() - 40.0).abs() < 1e-9 && (rect.width() - 80.0).abs() < 1e-9);
    }

    #[test]
    fn art_styles_remix_the_selection_with_their_request() {
        let mut app = ready_app(ImageProvider::Gemini);
        let pic = png(8, 8, |_, _| [255, 255, 255, 255]);
        let layer = app.active_layer().expect("a layer");
        let id = app.engine.new_shape_id();
        let mut s = Shape::new(
            id,
            ShapeKind::Bitmap {
                rect: Rect::new(10.0, 10.0, 50.0, 50.0),
                width_px: 8,
                height_px: 8,
                png: pic.into(),
                fx: Default::default(),
            },
        );
        s.fill = tracedraw_core::Fill::None;
        app.run(tracedraw_core::Command::AddShape { layer, shape: s });
        app.select(vec![id]);
        app.ai_image.style = Some(0);
        app.ai_image.fake = Some(|_| gemini_reply(&png(8, 8, |_, _| [0, 0, 255, 255])));
        app.ai_art_style(2);
        assert_eq!(app.ai_image.mode, Mode::Remix);
        assert_eq!(app.ai_image.style, None);
        assert_eq!(app.ai_image.remix_prompt, ART_STYLES[2].1);
        assert!(app.ai_image.sent[0].body.contains("pencil sketch"));
        assert_eq!(app.docker_tab, crate::app::DockerTab::AiGenerate);
        assert_eq!(count_bitmaps(&app), 2);
        // Every style has its menu name.
        for (key, words) in ART_STYLES {
            assert_ne!(crate::i18n::tr(key), key);
            assert!(words.contains("Keep its subject"));
        }
    }

    #[test]
    fn errors_of_the_service_are_shown_and_nothing_is_placed() {
        let mut app = ready_app(ImageProvider::OpenAi);
        app.ai_image.fake = Some(|_| {
            Ok(HttpResponse {
                status: 429,
                body: json!({"error": {"message": "Rate limit reached"}}).to_string(),
            })
        });
        app.ai_image.prompt = "x".into();
        app.ai_create(false);
        match &app.ai_image.status {
            ImageStatus::Failed(e) => assert!(e.contains("429") && e.contains("Rate limit")),
            s => panic!("{s:?}"),
        }
        assert_eq!(count_bitmaps(&app), 0);
        app.ai_image.fake = Some(|_| Err("network down".into()));
        app.ai_create(false);
        assert_eq!(
            app.ai_image.status,
            ImageStatus::Failed("network down".into())
        );
    }

    #[test]
    fn aspect_ratios_match_the_reference_picture() {
        assert_eq!(Aspect::nearest(1600, 900), Aspect::Wide169);
        assert_eq!(Aspect::nearest(1000, 1000), Aspect::Square);
        assert_eq!(Aspect::nearest(800, 1000), Aspect::Portrait45);
        assert_eq!(Aspect::nearest(2100, 900), Aspect::Ultrawide219);
        let mut app = ready_app(ImageProvider::Gemini);
        app.ai_image.aspect = Aspect::MatchSource;
        assert_eq!(app.ai_aspect(), Aspect::Square, "no reference: square");
        app.ai_source_from_bytes(&png(30, 20, |_, _| [1, 2, 3, 255]));
        assert_eq!(app.ai_aspect(), Aspect::Landscape32);
        app.ai_source_from_bytes(b"junk");
        assert!(matches!(app.ai_image.status, ImageStatus::Failed(_)));
    }

    #[test]
    fn an_example_image_goes_with_the_prompt() {
        let mut app = ready_app(ImageProvider::Gemini);
        app.ai_image.fake = Some(|_| gemini_reply(&png(8, 8, |_, _| [9, 9, 9, 255])));
        app.ai_source_from_bytes(&png(4, 4, |_, _| [200, 0, 0, 255]));
        app.ai_image.prompt = "the same in blue".into();
        app.ai_create(false);
        let body: Value = serde_json::from_str(&app.ai_image.sent[0].body).expect("json");
        assert!(body
            .pointer("/contents/0/parts/0/inline_data/data")
            .is_some());
        assert!(body.to_string().contains("reference"));
        // With the OpenAI API the example is edited.
        let mut app = ready_app(ImageProvider::OpenAi);
        app.ai_image.fake = Some(|_| {
            Ok(HttpResponse {
                status: 200,
                body: json!({"data": [{"b64_json": b64(&png(4, 4, |_, _| [1, 1, 1, 255]))}]})
                    .to_string(),
            })
        });
        app.ai_source_from_bytes(&png(4, 4, |_, _| [200, 0, 0, 255]));
        app.ai_image.prompt = "x".into();
        app.ai_image.count = 2;
        app.ai_create(false);
        assert_eq!(app.ai_image.sent.len(), 2);
        assert!(app
            .ai_image
            .sent
            .iter()
            .all(|r| r.url.ends_with("/images/edits")));
    }

    #[test]
    fn results_replace_the_last_ones_and_can_stay_out_of_the_drawing() {
        let mut app = ready_app(ImageProvider::Gemini);
        app.ai_image.fake = Some(|_| gemini_reply(&png(8, 8, |_, _| [9, 9, 9, 255])));
        app.ai_image.prompt = "x".into();
        app.ai_image.count = 3;
        app.ai_create(false);
        assert_eq!(app.ai_image.results.len(), 3);
        app.ai_image.auto_place = false;
        app.ai_image.count = 1;
        app.ai_create(false);
        assert_eq!(
            app.ai_image.results.len(),
            1,
            "a new Generate replaces them"
        );
        assert_eq!(count_bitmaps(&app), 3, "not inserted");
        assert_eq!(app.ai_image.status, ImageStatus::Done(1));
    }

    #[test]
    fn svg_vectors_come_from_the_drawing_service_as_shapes() {
        assert_eq!(
            extract_svg(
                "Here:\n```svg\n<svg viewBox='0 0 10 10'><rect width='5' height='5'/></svg>\n```"
            ),
            Some("<svg viewBox='0 0 10 10'><rect width='5' height='5'/></svg>".to_string())
        );
        assert_eq!(extract_svg("no drawing"), None);
        // The usual service (Messages API), without a key: nothing is sent.
        let mut app = App::headless();
        app.settings.ai_enabled = true;
        app.ai_image.mode = Mode::Vector;
        app.ai_image.vector_engine = VectorEngine::Svg;
        app.ai_image.prompt = "a red square and a blue circle".into();
        app.ai_generate();
        assert_eq!(
            app.ai_image.status,
            ImageStatus::Failed(tr("aiimg.svg_needs_drawing"))
        );
        app.ai.key = "k".into();
        app.ai_image.fake = Some(|_| {
            Ok(HttpResponse {
                status: 200,
                body: json!({
                    "content": [{"type": "text", "text": "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 50'><rect x='0' y='0' width='40' height='40' fill='#ff0000'/><circle cx='75' cy='25' r='20' fill='#0000ff'/></svg>"}],
                    "stop_reason": "end_turn"
                })
                .to_string(),
            })
        });
        app.ai_generate();
        assert_eq!(app.ai_image.status, ImageStatus::Done(1));
        let body: Value = serde_json::from_str(&app.ai_image.sent[0].body).expect("json");
        assert!(body["system"].as_str().is_some_and(|t| t.contains("SVG")));
        let g = app.selected_shapes().pop().expect("group");
        let ShapeKind::Group { children } = &g.kind else {
            panic!("a group");
        };
        assert_eq!(children.len(), 2);
        assert_eq!(g.name.as_deref(), Some(tr("aiimg.vector_name").as_str()));
        // Fitted to four fifths of the page width.
        use tracedraw_core::geometry::Shape as _;
        let b = g.page_path().bounding_box();
        assert!(
            (b.width() - app.page_size().width * 0.8).abs() < 0.5,
            "{}",
            b.width()
        );
        app.undo();
        assert!(app.doc().all_layers().all(|l| l.shapes.is_empty()));
    }
}
