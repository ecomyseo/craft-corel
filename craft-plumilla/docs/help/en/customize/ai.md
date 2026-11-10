# AI drawing and generating

TraceDraw can draw from a description in words and make pictures with an AI
service you choose and set up with your own key. TraceDraw has no AI of its
own and no account: nothing is sent until you run an AI command, and then
only what that command needs goes to the service you set up. Each request is
billed by your service; local model servers on your computer cost nothing.

## Setting up the AI service {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] has two pages.

![The Service page of the AI options](../../img/customize-ai-service.png)

On the **Service** page, set up one AI service for every AI feature:

1. Choose a **Preset**: Anthropic, OpenAI, Google Gemini, OpenCode Zen,
   OpenCode Go, or a model server on your computer (Ollama, LM Studio,
   llama.cpp server, vLLM). A preset fills in
   the kind of service and the address; **Custom** takes any address that
   speaks the same language as OpenAI's chat completions.
2. Type the **API key** of your account with that service (local servers need
   none). Tick **Remember the key on this computer** to keep it; it is then
   stored in plain text in your preferences.
3. Choose the **Model**, and the **Image model** for services that can make
   pictures (OpenAI, Google Gemini, or a custom address with an image model).
4. Click [[aiui.test]]. It asks the service for its models and says how many it
   has, or shows its error.

**Show the prompt bar** puts the AI prompt bar above the color palette.

## General {#general}

![The General page of the AI options](../../img/customize-ai-general.png)

- [[aiui.enable]] (on): turned off, every AI command is grayed.
- [[aiui.services]]: whether [[aiui.drawings]] and [[aiui.pictures]] are
  [[aiui.ready]] or [[aiui.not_ready]].
- [[aiui.features]]: what each AI command does.
- [[aiui.privacy_title]]: only what a command needs, to the service you set
  up, when you run it.

## Drawing with words {#draw-with-words}

Type a request such as "a red bicycle with blue wheels" or "make the circle
bigger and put the title under it", in the prompt bar or in the **Draw with
words** mode of the [[docker.ai_generate]] panel
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]]), and
press **Draw** or <kbd>Enter</kbd>. The service writes a script for the
drawing, which TraceDraw runs as one undo step. The conversation is kept, so
you can ask for changes; **New conversation** starts again. **Open in
Scripts** shows the script.

## The AI Generate panel {#generate}

The [[docker.ai_generate]] panel ([[menu.tools > menu.tools.ai_generate]], or
the AI Generate button on the standard toolbar) makes pictures with the image
model of your service.

![The AI Generate panel](../../img/customize-ai-generate.png)

Choose a mode with the buttons at the top:

- **Create image**: a picture from a description. **Inspire me** fills in an
  idea. Choose the aspect ratio.
- **Remix image**: changes a picture as you describe (the selected bitmap at
  first). Ready-made requests include mockup, restore photo, colorize sketch,
  replace background, change season and recolor.
- **Create vectors**: an illustration turned into editable shapes, either a
  picture traced into curves or an SVG drawing written by the text model.
- **Draw with words** (see above).

Every mode has a **Model**, an optional reference picture (**Use selection**
or **Import file**), the **Prompt**, and **Style** and **Color palette** tiles.
At the bottom, set the number of **Images** (1 to 4) and **Insert into
drawing**, then click **Generate**. Results stay in the panel; click one to
insert it.

## Other AI commands {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] keeps the subject of a
  picture and makes the rest transparent.
- [[menu.bitmaps.ai_art_styles]] repaints the selection in one of eight styles.

While AI is off or not set up, the commands are grayed, and their tip says
why; the [[aiui.set_up]] button opens the right options page.
