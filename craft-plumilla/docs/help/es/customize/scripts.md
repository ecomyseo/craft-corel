# Scripts y automatización

Los scripts automatizan el trabajo repetitivo: dibujar formas a partir de
datos, cambiar muchos objetos a la vez o añadir un formato de archivo. Se
escriben en JavaScript en el panel [[docker.scripts]]
([[menu.tools > menu.tools.scripts >
menu.tools.scripts_docker]]).

![El panel Scripts](../../img/customize-scripts.png)

## Escribir y ejecutar scripts {#run}

Un script trabaja con los objetos de TraceDraw: `Application`,
`ActiveDocument`, `ActivePage`, `ActiveLayer` y las formas, rellenos,
contornos, colores, curvas y efectos. Por ejemplo:

```js
for (let i = 0; i < 5; i++) {
  ActiveLayer.CreateRectangle(10 + i * 25, 10, 20, 20);
}
```

- [[docker.run]] ejecuta el script. Sus cambios son un solo paso de deshacer.
- [[docker.record]] graba lo que hace como un script (también
  [[menu.tools > menu.tools.scripts > menu.tools.scripts_record]]).
- [[docker.load]] y [[docker.save]] abren y guardan scripts;
  [[menu.tools.scripts_run]] ejecuta un archivo de script.
- [[docker.clear]] vacía el editor; la [[docker.output]] muestra lo que
  imprime el script y sus errores.

## Depuración {#debug}

En la versión de escritorio, haga clic en el margen del editor para poner un
punto de interrupción y luego haga clic en [[docker.debug]]. El script se
detiene en el punto de interrupción: puede **Continuar**, **Avanzar un paso**
o **Detener**, y escribir una expresión para evaluarla donde está en pausa el
script.

## Filtros de script {#filters}

Un script que define `TraceDrawFilter`, con un nombre, unas extensiones y una
función `Import` o `Export`, añade un formato de archivo a
[[menu.file.import]] y [[menu.file.export]]. Haga clic en
[[docker.install_filter]] para conservarlo; [[docker.script_filters]] enumera
los filtros instalados con [[docker.edit]] y [[docker.remove]].

## Asistentes de IA (MCP) {#mcp}

TraceDraw habla el Model Context Protocol, de modo que los asistentes de IA de
su equipo pueden abrir, editar, inspeccionar, exportar y guardar dibujos:

- `tracedraw --mcp` ejecuta TraceDraw sin ventana para un asistente.
- Para que un asistente edite el dibujo abierto en su ventana, active
  [[options.mcp_window]] en la página [[options.general]] de
  [[menu.tools > menu.tools.options > menu.tools.options_global]], y dé al
  asistente `tracedraw --mcp-window` como comando. Cada cambio es un paso de
  deshacer que puede revertir. Solo pueden conectarse los programas de su
  equipo que tengan el token secreto que escribe TraceDraw.

## La herramienta de línea de comandos {#cli}

Cada descarga incluye también `tracedraw-cli`, que convierte dibujos entre
formatos sin abrir el programa, para trabajos por lotes.

## Macros {#macros}

TraceDraw ejecuta proyectos de macros VBA escritos para programas de dibujo (versión de escritorio). [[menu.tools > menu.tools.macros > menu.tools.macros_load]] añade un proyecto (`.gms`) o archivos de módulo (`.bas`, `.cls`, `.frm`); los proyectos se guardan y se cargan al iniciar. El [[docker.macros]] muestra sus macros: haga doble clic en una o use [[menu.tools > menu.tools.macros > menu.tools.macros_run]] (<kbd>Alt+Shift+F8</kbd>) para ejecutarla. Los formularios y cuadros de mensaje y de entrada de la macro aparecen en pantalla, <kbd>Esc</kbd> o [[menu.tools > menu.tools.macros > menu.tools.macros_stop]] la detiene, y todo lo que cambia una ejecución es un solo paso de deshacer. [[menu.tools > menu.tools.macros > menu.tools.macro_editor]] (<kbd>Alt+F11</kbd>) muestra y edita el código. Las macros que llaman funciones de Windows o al modelo de objetos del propio editor no se ejecutan.
