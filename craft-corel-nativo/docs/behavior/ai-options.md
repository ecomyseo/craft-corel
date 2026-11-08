# Tools > Options > AI

A dialog of its own (Tools > Options > AI..., also the last row of the
AI Generate button's list) with two pages: General and Service. There is
one AI service for every AI feature, with one key.

## General

- Turn on AI features (on by default). Off greys out every AI command:
  AI Drawing's Draw and prompt bar, AI Generate, Remix, Create vectors
  and Remove Background.
- Services: Drawings and Pictures, each green "Ready" or "Not set up";
  Pictures says "Not offered by this service" when the chosen service
  makes drawings only.
- Features: the AI commands, greyed while the service cannot run them.
- What is sent: only what a command needs, to the service set up, when
  the user runs it.

## Service

The one AI service. Its text model writes drawings (AI Drawing, see
`ai-drawing.md`, and vectors as SVG); when the service can also make
pictures, its image model serves AI Generate, Remix, Create vectors and
Remove Background (see `ai-generate.md`), with the same key.

Fields: Preset, Service, Address, Model (the text model), API key, an
Image model row (only for a service that can make pictures), Show the
prompt bar, Remember the key, and the hint for local model servers.
The Model field can be typed in, and the arrow beside it lists the
models to pick from: the ones the service listed (Test Connection or
Refresh list, the last entry, asks for them), else the usual ones of its
kind (the current Claude models for the Messages API), with the model in
use first when it is in neither. `ai::tests::live_claude_api` (ignored)
runs against the real Messages API with a key in
`TRACEDRAW_ANTHROPIC_KEY`; without one it checks the service's "no key"
answer.
Test Connection asks the service for its models and says how many it
has, or shows its error; for a service that makes pictures it also asks
for the image models, which then fill the Image model list (an error of
that request shows under it). Below, Drawings and Pictures say where
they stand, as on General.

### Presets

The Preset list fills in the kind of service and the address and clears
the model and the image model, so the service's own apply; the address
stays editable, and a typed one shows as "Custom (typed address)".

| Preset | Address | Pictures |
|---|---|---|
| Anthropic (Messages API) | the usual one | no (drawings only) |
| OpenAI | the usual one | OpenAI Images API at the same base and key, image model `gpt-image-1` by default |
| Google Gemini | `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions` | Gemini image API at `https://generativelanguage.googleapis.com/v1beta` with the same key, image model `gemini-2.5-flash-image` by default |
| Ollama (this computer) | `http://localhost:11434/v1/chat/completions` | no (drawings only) |
| LM Studio (this computer) | `http://localhost:1234/v1/chat/completions` | no (drawings only) |
| llama.cpp server (this computer) | `http://localhost:8080/v1/chat/completions` | no (drawings only) |
| vLLM (this computer) | `http://localhost:8000/v1/chat/completions` | no (drawings only) |
| OpenCode | `https://opencode.ai/inference/openai/v1/chat/completions` (the Console Inference API's OpenAI-compatible path; the Zen and Go models are chosen by name) | no (drawings only) |
| Custom (typed address) | the user's, OpenAI-compatible | OpenAI Images API at the same base (the address without `/chat/completions`) when an image model is entered; left empty, no pictures |

Which service can make pictures follows from the address
(`ai::AiPrefs::image_support`): the Messages API never; an
OpenAI-compatible address at `https://api.openai.com/v1` is OpenAI, one
on `https://generativelanguage.googleapis.com/` is Gemini, one of the
four local presets or the OpenCode preset above makes drawings
only, and any other is Custom. Every request to an address on
`https://opencode.ai/` also carries an `x-opencode-session` header: one
id made when the program starts and sent with each request of that run,
which the service requires to count a run's requests together.
For a service that makes drawings only the Image model row is hidden
and one line says so. A service at an address of the user's needs no
key.

## Picture commands while the service cannot make pictures

AI Generate's picture modes, Remix, Create vectors (picture + trace),
Remove Background (Image menu, Object > AI Generate, the property bar,
the AI Generate button's list, a bitmap's context menu) and AI Art
Styles stay greyed while the service makes drawings only. Their hover
tip says the chosen AI service makes drawings only and to choose one
that also makes pictures in Tools > Options > AI > Service; while AI is
off it says so, and while a service that can make pictures is not set up
(no key, or a custom address without an image model) it says what is
needed there. The AI Generate docker shows the same as its banner.

## Older preferences

Versions before this one saved a drawing service and a separate image
service. On start they become the one service (`settings::migrate_ai`,
safe to run again):

- The drawing service stays when it was set up. When it can make
  pictures of the same kind as the old image service (Gemini and
  Gemini; OpenAI and OpenAI at its usual address; a custom address and
  the same base), it takes that image model, and its remembered key
  when it had none.
- When only the image service was set up, it becomes the one service:
  Gemini as the Google Gemini preset with that key; OpenAI at its usual
  address as the OpenAI preset; OpenAI-compatible at another address as
  a custom address (that base with `/chat/completions`) with the image
  model kept. An address that would match a chat-only local preset
  (LocalAI's usual `http://localhost:8080/v1` is llama.cpp's) is written
  with `127.0.0.1` in place of `localhost` (or the other way round), so
  it stays a custom address that makes pictures.
- A remembered key is never lost: when the old image service's key
  could not be taken (another kind of service, or a different key of
  its own), the old image service stays in the preferences file as it
  was.

## Hints

The Hints panel has an "AI features" topic: where drawings and pictures
stand, online services and their keys, AI on this computer (the presets
above), pictures (which services make them, a server of the user's with
both APIs), the browser (a local server must allow the page's origin,
such as `OLLAMA_ORIGINS=*`), what is sent, and Set Up AI.

## Banner

While what a part needs is not ready, the AI Drawing and AI Generate
dockers (and the Service page while AI is off) show a banner: "AI
features", what is missing (AI turned off; a service that makes drawings
only; or the service with the user's own API key and an image model) and
Set Up AI..., which opens General while AI is off and Service otherwise.

## Checks

- The banner's message follows the switch and the service (drawings
  only, pictures not set up), and turning AI off makes both not ready
  (`ui::ai_ui::tests::the_banner_shows_until_the_service_is_set_up`).
- The pages and the banner draw in every state with every preset and a
  custom address
  (`ui::ai_ui::tests::the_pages_and_the_banner_draw_in_every_state`);
  every options page draws (`ui::options::tests::every_page_draws`).
- Test Connection asks for the models and the image models with the one
  key, Gemini's at its own API, and none for a chat-only service
  (`ui::ai_ui::tests::test_connection_asks_for_the_models_and_the_image_models`).
- Which services make pictures, where their image calls go, and the
  tips of the greyed commands
  (`ai_image::tests::pictures_come_from_the_one_service_when_it_can_make_them`).
- Presets fill in the address, a local server needs no key, and a typed
  address is custom; older preference files turn AI features on
  (`ui::ai_ui::tests::presets_fill_the_address_and_typed_ones_stay_custom`).
- Older preferences with a separate image service become the one
  service, and no remembered key is lost
  (`settings::tests::an_older_image_service_alone_becomes_the_one_service`,
  `settings::tests::an_older_drawing_service_stays_and_no_remembered_key_is_lost`).
- The key is written to the preferences only when it is to be
  remembered (`ui::ai_docker::tests::the_key_is_kept_only_when_asked`).
