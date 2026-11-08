# AI Drawing

The AI Generate docker's Draw with words mode (its fourth mode button; also Object > AI Generate > Draw with Words and the arrow of the toolbar's AI Generate button) shows a panel
that turns a request in words ("a red bicycle with blue wheels", "make
the circle bigger and put the title under it") into a drawing. An AI
service of the user's choice, with the user's own key, writes a program
for the scripting object model (the one of the Scripts panel), and
TraceDraw runs it as one undo step. TraceDraw has no AI of its own and no
account; nothing is sent until the user presses Draw.

## Prompt bar

While a service is set up (a model, and a key where the service needs
one; a chat completions service at an address of the user's needs no
key), a bar shows above the colour palette, as wide as the drawing
window: "AI", the model list, the request (Enter draws, like Draw), Draw
(a spinner and Cancel while waiting), what the last request did, `...`
(opens AI Generate in Draw with words) and `x` (hides the bar). Window > Toolbars >
AI Prompt Bar and "Show the prompt bar above the palette" in the panel
turn it on and off (on by default); it does not show on the Welcome
Screen. The request is shared with the panel's, and a request that drew
clears it.

The model list offers the service's own models once fetched ("Fetch the
service's models": a GET of `models` next to the address, `/v1/models`
for the Messages API, `data[].id` of the reply), before that the usual
ones (`claude-sonnet-5-5`, `claude-opus-5-5`, `claude-haiku-5-5`,
`claude-fable-5-1` for the Messages API, none for chat completions); the
model in use is always listed.

## Connection

The connection is the one AI service, also set in Tools > Options > AI >
Service (see `ai-options.md`), whose key also serves pictures when the
service can make them; with AI features turned off there, Draw and the
prompt bar are off and the panel shows a banner. For a service that can
make pictures (OpenAI, Google Gemini, a custom OpenAI-compatible
address) the connection also has an Image model row; for one that makes
drawings only, a line says so.

The Connection section is open until a key is entered for a service that
needs one.

| Field | Default | Notes |
|---|---|---|
| Service | Anthropic (Messages API) | or OpenAI-compatible (chat completions), the API many services and local model servers offer |
| Address | `https://api.anthropic.com/v1/messages`, or `https://api.openai.com/v1/chat/completions` | empty uses the default shown in grey; local servers: their own address, such as `http://localhost:11434/v1/chat/completions` |
| Model | `claude-sonnet-5-5` for the Messages API; none for chat completions | the model name as the service lists it |
| API key | empty | a password field; needed by the Messages API, optional for chat completions (sent as `Authorization: Bearer`) |
| Remember the key on this computer | off | on: the key is kept in the preferences file (in the browser: the page's local storage), in plain text; off: it lives only until TraceDraw closes |

Choosing another service clears the address and model (their defaults
apply).

## Drawing

- Write the request and press Draw (or Ctrl+Enter in the request).
  While waiting the panel shows a spinner and Cancel (the reply is then
  dropped when it comes).
- What is sent: the instructions and a reference of the object model
  (`ai::SYSTEM_PROMPT`: units in millimetres, origin at the page's
  bottom-left corner with y up, angles counter-clockwise in degrees, text
  sizes in points, colours, every method the program may use); up to
  the 3 earlier requests of the conversation with their replies; the
  request with a JSON description of the active page: its size, its
  layers (the active one marked) and up to 300 objects (id, type, name,
  bounds in mm, fill, outline, opacity, text and font, group children;
  the number of objects left out), and the selection. File names,
  pictures and the other pages are not sent.
- Messages API requests carry `x-api-key`, `anthropic-version:
  2023-06-01`, `max_tokens` 8000 and the instructions as `system`; chat
  completions requests put the instructions first as a `system` message.
- The reply's program is its JavaScript code block (else its first code
  block, else the reply itself when it reads as code). It runs as an
  untrusted script: Export and ImportBitmap are refused (no file access),
  each function call may run at most 1,000,000 loop iterations, calls
  nest at most 400 deep and the program may make at most 20,000 changes.
  It is all or nothing: a program that fails leaves the drawing as it
  was. Its changes are one undo step, "AI Drawing".
- A program that fails is sent back once, with its error, asking for the
  whole corrected program ("The script failed; asking for a corrected
  one..."). If that one fails too, nothing is drawn and the error shows.
- After a drawing, the panel says how many changes were made, shows what
  the program printed, and keeps the program under Script, where Open in
  Scripts copies it to the Scripts panel for editing and running again.
- New conversation forgets the earlier requests.
- Errors shown: the service's own message with the HTTP status (a wrong
  key, no credit, too many requests), a network error, a reply cut short
  (ask for less at a time), a reply with no program, a program over
  256 KB.
- The program can also use `ActiveLayer.CreatePath(d)`: SVG path data
  (`M L H V C S Q T A Z`) in page coordinates, which the Scripts panel
  has too.

## Desktop and browser

The desktop program sends the request over HTTPS, checking certificates
against the system's trusted ones and using the proxy of the usual
environment variables (`HTTPS_PROXY`, `NO_PROXY`), and waits up to five
minutes. In the browser the page sends the request itself, so the
service must accept requests from web pages: the Messages API does when
told the key is the user's own (the request carries
`anthropic-dangerous-direct-browser-access: true`); a local server must
allow the page's origin (for example `OLLAMA_ORIGINS`).

## Checks

- A request to a test transport draws a red circle as one undo step,
  the page goes along, and the next request carries the earlier one
  without its page (`ai::tests::a_request_draws_through_the_service_as_one_undo_step`).
- A program that draws then fails is sent back with its error, and only
  the corrected drawing stays; a program that fails twice draws nothing
  (`ai::tests::a_failing_script_is_sent_back_once_and_nothing_half_drawn_stays`).
- Export and ImportBitmap are refused and an endless loop stops; the
  user's own scripts still export
  (`ai::tests::scripts_from_the_service_cannot_reach_files_or_run_forever`).
- Requests for both kinds of service carry their headers, model and
  messages (`ai::tests::requests_are_built_for_both_kinds_of_service`);
  replies, error replies, cut and empty replies are read
  (`ai::tests::replies_and_errors_are_read`); code blocks are found
  (`ai::tests::the_program_is_taken_from_its_code_block`).
- No request leaves without words, a key (where needed) and a model
  (`ai::tests::requests_need_words_a_key_and_a_model`); network and
  service errors are shown (`ai::tests::errors_of_the_service_and_the_network_are_shown`).
- The desktop transport posts to a server on the same computer and reads
  its error replies (`ai::tests::the_desktop_transport_posts_and_reads_error_replies`);
  the real Messages API refuses a made-up key with 401
  (`ai::tests::the_real_service_refuses_a_made_up_key`, run by hand with
  `--ignored`).
- Every method the reference names exists in the object model
  (`ai::tests::the_reference_names_only_methods_the_object_model_has`).
- The key is written to the preferences only when it is to be
  remembered (`ui::ai_docker::tests::the_key_is_kept_only_when_asked`);
  the panel draws in every state (`ui::ai_docker::tests::the_docker_draws_in_every_state`).
- The active page's description lists up to the limit and counts the
  rest (`describe::tests::the_active_page_is_described_up_to_a_number_of_objects`).
- The prompt bar shows only with a service set up and the bar on, and
  the window draws with it in every state
  (`ui::ai_bar::tests::the_bar_shows_while_a_service_is_set_up`); the
  model list takes the service's models and keeps the one in use; the
  models address of both kinds of service
  (`ui::ai_bar::tests::the_model_list_offers_the_services_models`).
- Checked on screen: a request typed in the bar and Enter drew a sun, a
  hill with a gradient and a title from a test service on the same
  computer, and cleared the field.
