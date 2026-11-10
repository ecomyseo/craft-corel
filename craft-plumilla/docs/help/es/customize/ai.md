# Dibujar y generar con IA

TraceDraw puede dibujar a partir de una descripción en palabras y crear
imágenes con un servicio de IA que usted elige y configura con su propia
clave. TraceDraw no tiene IA propia ni cuenta: no se envía nada hasta que
ejecuta un comando de IA, y entonces solo va al servicio que configuró lo que
ese comando necesita. Cada solicitud la factura su servicio; los servidores de
modelos locales en su equipo no cuestan nada.

## Configurar el servicio de IA {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] tiene dos páginas.

![La página Servicio de las opciones de IA](../../img/customize-ai-service.png)

En la página **Servicio**, configure un único servicio de IA para todas las
funciones de IA:

1. Elija un **Ajuste preestablecido**: Anthropic, OpenAI, Google Gemini o un
   servidor de modelos en su equipo (Ollama, LM Studio, llama.cpp server,
   vLLM). Un ajuste preestablecido rellena el tipo de servicio y la dirección;
   **Personalizado** acepta cualquier dirección que hable el mismo lenguaje
   que las chat completions de OpenAI.
2. Escriba la **Clave de API** de su cuenta en ese servicio (los servidores
   locales no la necesitan). Marque **Recordar la clave en este equipo** para
   conservarla; entonces se guarda como texto sin cifrar en sus preferencias.
3. Elija el **Modelo**, y el **Modelo de imagen** para los servicios que
   pueden crear imágenes (OpenAI, Google Gemini o una dirección
   personalizada con un modelo de imagen).
4. Haga clic en [[aiui.test]]. Pide al servicio sus modelos y dice cuántos
   tiene, o muestra su error.

**Mostrar la barra de instrucciones** coloca la barra de instrucciones de IA
encima de la paleta de colores.

## General {#general}

![La página General de las opciones de IA](../../img/customize-ai-general.png)

- [[aiui.enable]] (activado): si se desactiva, todos los comandos de IA
  aparecen atenuados.
- [[aiui.services]]: si [[aiui.drawings]] y [[aiui.pictures]] están
  [[aiui.ready]] o [[aiui.not_ready]].
- [[aiui.features]]: lo que hace cada comando de IA.
- [[aiui.privacy_title]]: solo lo que necesita un comando, al servicio que
  configuró, cuando lo ejecuta.

## Dibujar con palabras {#draw-with-words}

Escriba una petición como "una bicicleta roja con ruedas azules" o "haz el
círculo más grande y pon el título debajo", en la barra de instrucciones o en
el modo **Dibujar con palabras** del panel [[docker.ai_generate]]
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]]), y
pulse **Dibujar** o <kbd>Enter</kbd>. El servicio escribe un script para el
dibujo, que TraceDraw ejecuta como un solo paso de deshacer. La conversación
se conserva, así que puede pedir cambios; **Nueva conversación** empieza de
nuevo. **Abrir en Scripts** muestra el script.

## El panel Generar con IA {#generate}

El panel [[docker.ai_generate]] ([[menu.tools > menu.tools.ai_generate]], o el
botón Generar con IA de la barra de herramientas estándar) crea imágenes con
el modelo de imagen de su servicio.

![El panel Generar con IA](../../img/customize-ai-generate.png)

Elija un modo con los botones de arriba:

- **Crear imagen**: una imagen a partir de una descripción. **Inspírame**
  rellena una idea. Elija la relación de aspecto.
- **Remezclar imagen**: cambia una imagen según lo que describa (el mapa de
  bits seleccionado, de forma predeterminada). Las peticiones ya preparadas
  incluyen maqueta, restaurar foto, colorear boceto, sustituir fondo, cambiar
  estación y cambiar colores.
- **Crear vectores**: una ilustración convertida en formas editables, ya sea
  una imagen calcada en curvas o un dibujo SVG escrito por el modelo de texto.
- **Dibujar con palabras** (vea más arriba).

Todos los modos tienen un **Modelo**, una imagen de referencia opcional
(**Usar selección** o **Importar archivo**), las **Instrucciones** y los
mosaicos **Estilo** y **Paleta de colores**. Abajo, indique el número de
**Imágenes** (de 1 a 4) y **Insertar en el dibujo**, y haga clic en
**Generar**. Los resultados se quedan en el panel; haga clic en uno para
insertarlo.

## Otros comandos de IA {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] conserva el motivo de una
  imagen y hace transparente el resto.
- [[menu.bitmaps.ai_art_styles]] vuelve a pintar la selección en uno de ocho
  estilos.

Mientras la IA está desactivada o sin configurar, los comandos aparecen
atenuados y su sugerencia explica por qué; el botón [[aiui.set_up]] abre la
página de opciones adecuada.
