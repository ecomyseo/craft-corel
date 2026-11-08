# Craft Corel · Guía técnica

Edición del 8 de noviembre de 2026. Arquitectura, automatización y criterios de comprobación de la aplicación independiente basada en TraceDraw.

La portable 214714 constituye la base de producto comprobada de esta edición. Los resultados se identifican por recorrido y artefacto. El desarrollo posterior de unidades CDR y reglas debe vincularse a su paquete verificado antes de atribuirle esas capacidades. La versión objetivo de una API no identifica el producto original ni demuestra equivalencia completa.

[TOC]

## 1. Arquitectura y responsabilidades

El sistema está escrito en Rust. El modelo y las órdenes pertenecen al núcleo; la interfaz nativa utiliza egui/eframe y existe un destino WebAssembly. La aplicación de escritorio no es un contenedor Electron ni una automatización de una instalación externa.

| Componente | Responsabilidad |
|---|---|
| core | Documento, geometría, color, recursos, órdenes e historial |
| cdr-format | Contenedores, RIFF, ZIP y registros de formato |
| cdr | Lectura y escritura de contenido CDR y CMX |
| io | TDRAW, importadores y exportadores de intercambio |
| render | Representación CPU y cachés de imagen |
| text | Fuentes, composición y contornos de texto |
| tracecut | Geometría auxiliar de corte y medición |
| vba-project / vba | Proyectos de macros y compilación al motor propio |
| aplicación | Ventana, herramientas, paneles, scripts y servidor MCP |
| CLI | Inspección, conversión y diagnósticos sin interfaz |

La geometría interna se almacena en milímetros. Los tamaños tipográficos pueden expresarse en puntos. Un cambio de unidad de presentación no debe multiplicar ni dividir las coordenadas del modelo.

La regla de edición es construir una orden y ejecutarla mediante el motor. Un importador prepara recursos, identificadores y políticas antes de confirmar. Las pruebas de transacción distinguen una preparación fallida de una operación parcial que haya creado objetos o consumido identificadores.

## 2. Distribución modular y despliegue

La distribución portable contiene un lanzador `craft-corel.exe`, la aplicación en `craft-corel-engine.dll` y módulos de Rust compilados como DLL reales. La cadena incluye bibliotecas de codificación, núcleo, formato CDR, representación, texto, geometría auxiliar, CDR, IO y macros; además conserva runtimes y avisos de licencia.

El conjunto documentado se compila con Rust 1.97.0 y una interfaz privada compatible dentro de esa compilación. No intercambie DLL entre paquetes: una extensión `.dll` o un enlace correcto por sí solos no demuestran que pueda ejecutarse otra combinación.

El paquete se verifica mediante hashes de archivos, importaciones PE, ejecución real sin ventana y pruebas funcionales sobre el conjunto distribuido. La ausencia de una DLL obligatoria debe producir un error identificable; no se sustituye el motor por una biblioteca vacía.

Las preferencias portables se guardan junto a la aplicación. Para una prueba aislada utilice una carpeta de paquete nueva y un perfil separado según las herramientas del proyecto. No copie automáticamente configuraciones privadas a una publicación.

La edición de desarrollo no tiene firma Authenticode propia. Integridad de ZIP, firma, reputación de descarga y detección antivirus son comprobaciones distintas. La publicación del código fuente no equivale a distribuir un instalador firmado.

Las actualizaciones del proyecto de origen están desactivadas. La fuente alpha actual mantiene el bloqueo en comprobación, descarga, instalación y reinicio; sus controles de interfaz y regresiones se comprueban por separado del mero valor de una constante. No se debe reactivar una actualización de otra distribución para completar DLL faltantes.

## 3. Formatos y rutas de intercambio

`save` conserva documentos: TDRAW o CDR. `export` crea representaciones o archivos de intercambio. La extensión se comprueba y un formato desconocido debe fallar sin guardar JSON nativo bajo otro nombre.

| Ruta | Salidas o entradas relevantes | Límite práctico |
|---|---|---|
| Documento propio | TDRAW | Conserva el modelo propio; comprobar versión y límites de lectura |
| CDR | Lectura/escritura de versiones implementadas, hasta 27 en el escritor propio | Cobertura parcial; una versión emitida no certifica compatibilidad externa total |
| Vector y página | SVG/SVGZ, PDF, AI, EPS/PS/PRN, DXF, EMF/WMF, PLT | Texto, efectos, unidades y páginas varían según formato |
| Imágenes | PNG, JPEG, BMP, TIFF, WebP, GIF; PSD por ruta específica | Compresión, paleta, alfa, perfiles y capas tienen capacidades diferentes |
| Texto | TXT, RTF, DOCX | Importación de contenido y estilos implementados, no motor de oficina completo |
| CMX | Importación parcial | No todos los tipos de relleno, lentes u objetos están cubiertos |

Los códecs rasterizados compartidos aceptan opciones de DPI, calidad y WebP sin pérdida. Deben escribir el formato solicitado y rechazar extensiones desconocidas. Las pruebas decodifican realmente los archivos de salida. El WebP sin pérdida conserva RGBA sin cuantización previa; los casos con calidad 90 y 40 comprueban también colores impares y alfa.

La conservación de DPI y las dimensiones se comprueban por separado de la validez del códec. El tamaño rasterizado utiliza techo para fracciones físicas reales, con tolerancia numérica estrecha alrededor de enteros para evitar añadir un píxel por error de coma flotante. No se acepta una comparación de apariencia como sustituto de las dimensiones.

La nueva fuente alpha añade límites de entrada y expansión, con repetición de pruebas todavía en curso al fijar esta edición: lectura general de hasta 512 MiB, texto/JSON nativo y SVGZ expandido hasta 256 MiB; lista de datos CDR hasta 1 MiB y 4.096 nombres; agregado de flujos externos de hasta 3 GiB y presupuestos propios de bloques comprimidos. Son límites de implementación por capa, no una promesa de que cualquier documento inferior a ellos abra completo o resulte seguro. El paquete final y su informe deben confirmar los recorridos ejercitados.

AI dispone de una modalidad compatible con PDF y otra AI 8 basada en EPS. SVGZ es SVG comprimido. La reimportación de un formato no demuestra que todos los objetos conserven estructura, color o edición; las métricas deben separar archivo válido, tamaño, apariencia y datos.

## 4. Servidor MCP sin ventana

El ejecutable admite `--mcp`: JSON-RPC 2.0 por entrada y salida estándar, un mensaje por línea, con negociación MCP. No abra una ventana para ese modo. Configure un cliente con la ruta real de su ejecutable:

```json
{
  "mcpServers": {
    "craft-corel": {
      "command": "C:/CraftCorel/craft-corel.exe",
      "args": ["--mcp"]
    }
  }
}
```

Inicie con `initialize` y consulte `tools/list` para usar el esquema de ese binario. Los registros deben ir al canal previsto y no contaminar las líneas JSON del protocolo.

| Herramienta | Función |
|---|---|
| open | Abre una ruta y devuelve información del documento o error real |
| new_document | Crea un documento con tamaño y título opcionales |
| save | Guarda TDRAW/CDR; admite versión CDR cuando procede |
| export | Exporta según extensión y opciones verificadas |
| run_script | Ejecuta JavaScript con modelo explícito o predeterminado |
| import | Incorpora un archivo y devuelve identificadores y aviso |
| copy_selection | Devuelve SVG con selección nativa propia; no usa el portapapeles del SO |
| paste_selection | Recibe SVG ordinario o bloque nativo propio y devuelve selección |
| document_info | Describe páginas, capas, objetos, selección y propiedades |
| undo / redo | Recorre un paso de historial |

Son 11 herramientas contando `undo` y `redo` por separado. El modo de puente hacia una ventana abierta existe en la base, pero sus controles visibles y transporte de escritorio no se dan por probados mediante el servidor sin ventana.

`export` recibe `path`, `page_index`, `dpi`, `quality`, `webp_lossless` y, para AI, `ai_version`. Los valores predeterminados documentados son página 0, 96 DPI, calidad 90 y compresión WebP con pérdida. DPI se limita a 1..2400 y calidad a enteros 1..100. Consulte el esquema real antes de depender de una opción.

## 5. Contratos de JavaScript

`run_script.object_model` admite `native`, predeterminado, y `vgcore`. Un nombre o tipo desconocido se rechaza antes de evaluar el script, conservando documento, selección, asignador e historial pendiente.

En `native`, las longitudes son milímetros y el contrato histórico del rectángulo es posición y dimensiones. En `vgcore`, las longitudes usan `ActiveDocument.Unit` y las colecciones siguen convenciones de macros de índice 1.

Elija `object_model: vgcore` y envíe este JavaScript en `source`:

```javascript
ActiveDocument.Unit = cdrMillimeter;
const s = ActiveLayer.CreateRectangle(10,30,50,10);
print(s.SizeWidth,s.SizeHeight);
```

Ese ejemplo produce 40 × 20 en el modelo VG. La misma llamada de cuatro números en `native` produce 50 × 10. No autodetecte el modelo por el nombre del método ni cambie scripts existentes para resolver una diferencia que procede de mezclar contratos.

Una ejecución agrupa sus cambios en un paso. Si el programa falla después de órdenes válidas, esas órdenes pueden mantenerse: es el comportamiento propio existente de scripts de usuario. Esto se distingue del rechazo previo de un argumento inválido de la herramienta MCP.

## 6. Rectángulos y macros VBA

Los contratos públicos 27.2 recuperados distinguen tres constructores. `CreateRectangle` recibe lados `Left, Top, Right, Bottom` y cuatro porcentajes opcionales. `CreateRectangle2` recibe posición, anchura, altura y cuatro radios físicos opcionales. `CreateRectangleRect` recibe un rectángulo y los cuatro radios físicos. Cada opcional parte de 0.

El orden público es superior izquierda, superior derecha, inferior derecha e inferior izquierda. El núcleo usa otro orden interno y el adaptador remapea las inferiores. `Rectangle.MaxRadius` es la mitad del lado menor, no el mayor radio previamente asignado.

La documentación contiene una contradicción editorial sobre el anclaje de `CreateRectangle2`. El modelo propio conserva su convención inferior izquierda y la etiqueta como tal. La reacción exacta del programa original ante radios negativos, dimensiones invertidas, no finitos o coerciones fraccionarias no se ha ejecutado. Las guardas propias no se presentan como una reproducción de sus errores.

VBA se analiza y compila al motor JavaScript propio. No se usa una instalación COM del editor de referencia. La compatibilidad de nombres no acredita todos los métodos, parámetros ni comportamientos de un SDK completo.

Las propiedades `Application.Version`, `VersionMajor`, `VersionMinor` y `VersionBuild` de la documentación original describen su aplicación; no constituyen una orden para fingir su identidad. La versión propia, el objetivo de API y la versión de formato de archivo deben identificarse por separado.

## 7. Unidades y reglas independientes

La enumeración de automatización tiene identificadores de 0 a 16; 0 es válido y no debe confundirse con ausencia. En el modelo propio se conserva la unidad opcional en `Metadata.script_unit`, sin alterar geometría interna. TDRAW mantiene esa preferencia entre peticiones y procesos en las pruebas de 214714.

Las páginas oficiales 27.2 de `Rulers.HUnits` y `Rulers.VUnits` dicen que afectan solo a interfaz y no a automatización. La nueva fuente mantiene preferencias opcionales independientes para cada eje. El host comprueba identificadores antes de ordenar; un valor desconocido almacenado se conserva, aunque su lectura de macros deba dar un error explícito.

La portable 214714 pierde `script_unit=3` al reabrir un CDR propio, aun conservando la escena. La fuente alpha posterior incorpora una extensión privada versionada para conservar unidad de macros y ejes, todavía pendiente de producto final. RIFF utiliza el bloque propio `ccun`; ZIP utiliza `craft-corel/document-units.json`. El vínculo con la configuración mediante FNV detecta obsolescencia accidental: no es una firma criptográfica, una defensa contra manipulación maliciosa ni una prueba de que otro editor lo interprete. No añade una tabla de unidades estándar del formato original 27 y se mantiene diferenciada de la base 214714.

La investigación MCFG encontró una muestra oficial de formato 21 con configuración de 7.944 bytes y una muestra propia 27 que reutiliza esa plantilla. Una especificación declarativa pública deja comentados sus campos de unidades y advierte de posiciones incorrectas en versiones nuevas. No se interpreta ese dato como confirmación independiente de la estructura de 27.

Los factores de píxel y cícero difieren entre algunas rutas heredadas de interfaz y macros. La documentación recuperada no decide todos sus factores. No se certifican las 17 conversiones por el mero hecho de conservar sus identificadores.

## 8. Color, perfiles y transacciones de importación

Asignación conserva muestras y cambia calibración. Conversión transforma muestras mediante los perfiles disponibles. Los datos CMYK retenidos, alfa y perfil deben actualizarse o conservarse de manera coherente; los errores no deben aceptar silenciosamente una apariencia incorrecta.

La importación prepara contenido y recursos antes de una sola transacción. Los identificadores extranjeros de símbolos, estilos, tintas, fuentes y otros recursos se remapean; una referencia del archivo entrante no puede apuntar por accidente a un símbolo ya existente del destino.

Las pruebas propias cubren tres políticas CMYK, perfiles sintéticos diferenciados, destinos válidos e inválidos y restauración por deshacer/rehacer. Un perfil sintético sirve para distinguir rutas de cálculo, pero no para certificar precisión de impresión. RGB, gris, tintas especiales y todo estilo con color requieren sus propios oráculos.

La asignación y conversión no deben confundirse con el refresco de la vista previa. Una vista previa PNG puede regenerarse sin cambiar las tintas originales. Una prueba solo de PNG no sustituye el cotejo de todos los bytes de canales y perfiles.

## 9. Transporte nativo de selección

El formato propio utiliza un bloque versionado dentro del SVG. La decodificación distingue SVG ajeno sin bloque, bloque propio válido y bloque propio dañado. Un bloque propio inválido debe dar error, sin una importación SVG simplificada que pierda datos silenciosamente.

La validación limita bytes, profundidad, recursos, imágenes y expansión de símbolos antes de deserializar o clonar estructuras grandes. Los límites del modelo y del XML son diferentes: 128 grupos del modelo pueden producir más de 128 antecesores XML en una vista previa real.

Las pruebas llegaron a 128 grupos con recopia nativa y rechazaron 129 conservando el rehacer anterior. El candidato 205021 falló esa recopia y permanece rechazado; no se le transfiere la aprobación del sucesor. El soporte de pila de trabajadores nativos no modifica la pila principal del ejecutable ni demuestra comportamiento en navegador.

Las comprobaciones entre procesos usan el mismo formato de bytes del producto, pero no invocan el portapapeles del sistema. La publicación real de Windows y las promesas asíncronas del navegador son verificaciones separadas.

## 10. CLI y diagnósticos

Cuando se dispone de su binario, `craft-corel-cli` permite inspeccionar, informar, comprobar y convertir sin ventana. La portable modular de escritorio no debe darse por acompañada del CLI si este no figura en su manifiesto.

```text
craft-corel-cli info dibujo.cdr
craft-corel-cli inspect dibujo.cdr
craft-corel-cli check dibujo.cdr
craft-corel-cli convert dibujo.tdraw salida.svg
```

La inspección de contenedor y el comprobador de un archivo ayudan a detectar fallos estructurales, pero no sustituyen el render ni la lectura del destinatario. Las pruebas de mutación o estrés deben ejecutarse sobre copias de trabajo y mantener evidencia de entradas y errores.

Para una incidencia exporte una muestra mínima, anote el modelo de script, las unidades, la versión del paquete y el resultado esperado. Compare imágenes al mismo tamaño y conserve perfiles, fuentes y datos fuente cuando sean necesarios y se puedan compartir legalmente.

## 11. Construcción y mantenimiento

La herramienta documentada del trabajo local es Rust 1.97.0. Use un checkout y un directorio de destino separados para experimentos de módulos; no modifique un producto entregado para resolver un enlace. Los scripts de empaquetado y comprobación del espacio de trabajo fijan fuentes, dependencias y artefactos.

Una verificación razonable de cambios de fuente incluye pruebas focalizadas, las regresiones afectadas y compilación WASM cuando la ruta se comparte. Clippy y compilación son controles útiles, pero no demuestran que la portable distribuida ejecute un flujo de importación o conserve una imagen.

Cada paquete debe mantener sus avisos de licencia, hashes y procedencia. Las dependencias se cotejan con el lock y los archivos fijados; los suplementos de licencia deben corresponder al grafo concreto. No se transfiere el sello de un paquete anterior por tener el mismo número de bibliotecas.

La evidencia actual distingue 393 paquetes de grafo de Windows, 379 procedentes del registro y otros locales o vendorizados. Es un resultado del conjunto auditado, no un número universal para todos los destinos o futuras compilaciones.

## 12. Matriz de cobertura y criterio de cierre

| Área | Evidencia comprobada | Lo que no demuestra |
|---|---|---|
| Paquete modular | Hashes, ZIP, importaciones, carga y pruebas reales sin ventana | Firma, reputación, GPU o todas las versiones de Windows |
| Rectángulos | Constructores, cuatro radios, control negativo y máscara PNG/CDR propia | Anclaje original sin contradicción o todos los objetos CDR |
| Modelos MCP | Selector, argumentos inválidos, historial y recorridos propios | Toda la API VG ni ejecución COM original |
| Color CMYK | Canales/perfiles/alfa en escenarios propios | Impresión certificada o todas las políticas de color |
| Copia nativa | Bytes entre procesos, recursos y límites | Portapapeles real del SO o navegador |
| Raster y vectores | Formato real, lecturas y métricas específicas | Fidelidad universal de tamaño, color, texto y edición |
| Fuente WASM | Comprobación de compilación | Arranque, fuentes, rendimiento y portapapeles web reales |
| Interfaz y hardware | Capacidades heredadas y pruebas sintéticas o sin ventana | Recorrido completo de escritorio, lápiz o impresora |
| Unidades CDR | Problema propio observado; conservación privada en validación | Semántica estándar completa del original 27 |

Una fase se cierra únicamente para su alcance demostrado. El objetivo de equivalencia completa sigue abierto. Una prueba de lectura, un código de salida 0, un menú visible o una mejora de velocidad no sustituyen la reproducción completa y su oráculo independiente.

## 13. Referencias y documentación complementaria

El repositorio del proyecto es [ecomyseo/craft-corel](https://github.com/ecomyseo/craft-corel). Consulte su revisión y el manifiesto del paquete que utilice; la existencia del repositorio no certifica la descarga de un instalador firmado.

El repositorio contiene `docs/features.md`, notas por función en `docs/behavior/`, las notas de formato `docs/cdr-format.md` y el registro `CLEANROOM.md`. Los estados heredados de la base deben leerse junto con las observaciones y límites de esta edición.

Las fuentes públicas de API consultadas incluyen [CreateRectangle 27.2](https://community.coreldraw.com/sdk/api/draw/27.2/m/layer.createrectangle?lang=vba), [CreateRectangle2 27.2](https://community.coreldraw.com/sdk/api/draw/27.2/m/layer.createrectangle2?lang=vba), [Document.Unit](https://community.coreldraw.com/sdk/api/draw/27.2/p/document.unit?lang=vba), [Rulers.HUnits](https://community.coreldraw.com/sdk/api/draw/27.2/p/rulers.hunits?lang=vba) y [Rulers.VUnits](https://community.coreldraw.com/sdk/api/draw/27.2/p/rulers.vunits?lang=vba). Describen contratos públicos; no son pruebas de ejecución local.

Este documento y el manual de usuario se mantienen cada uno en una fuente Markdown. El generador produce HTML autónomo con índice y búsqueda y PDF con índice enlazado. Los informes de validación registran hashes, extracción textual, dimensiones de página y renders revisados; el HTML no necesita servicios externos ni una conexión para leerse.
