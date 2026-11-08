# Craft Corel · Manual de usuario

Edición del 8 de noviembre de 2026. Aplicación vectorial de desarrollo basada en TraceDraw. Manual en castellano para la distribución portable de Windows de 64 bits.

Este manual explica cómo trabajar con documentos, organizar el dibujo y preparar archivos de intercambio. La referencia de producto comprobada al redactarlo es la portable modular 214714. Los cambios posteriores de unidades y reglas se identifican expresamente cuando aún corresponden a fuente en validación. Craft Corel es una aplicación independiente: no requiere instalar CorelDRAW y no acredita compatibilidad completa con sus documentos, herramientas o macros.

[TOC]

## 1. Instalar y conservar la portable

1. Extraiga el ZIP completo en una carpeta donde pueda escribir, por ejemplo `C:\CraftCorel\`.
2. Mantenga `craft-corel.exe`, `craft-corel-engine.dll` y las demás DLL en el mismo conjunto de carpetas del paquete. Copiar únicamente el ejecutable no instala la aplicación.
3. Abra `craft-corel.exe`. La carpeta `craft-corel-settings` conserva las preferencias de esa portable.
4. Cree un dibujo de prueba y guárdelo con nombre nuevo en formato `.tdraw` antes de abrir un trabajo importante.

No necesita Python, Rust ni el programa de referencia para usar esta portable. Las DLL internas no son complementos intercambiables entre versiones: al actualizar, extraiga el paquete nuevo completo en otra carpeta y conserve el anterior para poder volver atrás. La actualización automática del proyecto de origen está desactivada en esta distribución.

La edición de desarrollo documentada no dispone de firma de editor. Un hash permite comprobar que el ZIP coincide con el entregado, pero no sustituye la firma ni una revisión antivirus. Si el sistema bloquea el archivo o falta una DLL, conserve el mensaje exacto y compruebe primero que la extracción esté completa; no mezcle bibliotecas descargadas de sitios distintos.

## 2. Primer dibujo, paso a paso

1. Use Archivo > Nuevo o `Ctrl+N`. Elija un nombre y el tamaño físico de página.
2. Seleccione Rectángulo y dibuje una figura. Con la herramienta de selección, cambie su posición y tamaño desde la barra de propiedades.
3. Aplique un relleno de la paleta y ajuste el contorno. Compruebe qué objeto está seleccionado antes de modificar propiedades.
4. Añada texto y otro objeto. Agrúpelos con `Ctrl+G` si deben moverse juntos.
5. Guarde con `Ctrl+S` como `primer-dibujo.tdraw`.
6. Exporte una copia con `Ctrl+E` a PNG o PDF. Abra el resultado en un visor para revisar página, tamaño y contenido.
7. Pruebe `Ctrl+Z` y `Ctrl+Shift+Z` para entender qué acción compone cada paso del historial.

El formato nativo es la copia de trabajo más completa. Una exportación sirve para una finalidad concreta y puede perder propiedades editables aunque la imagen se vea igual. Conserve el documento nativo y el archivo recibido inicialmente.

## 3. Orientarse en el espacio de trabajo

La caja de herramientas reúne selección, formas, curvas, texto y edición de nodos. La barra de propiedades cambia según la herramienta o selección. Los paneles de objetos y propiedades permiten revisar estructura, relleno, contorno y otros ajustes. La paleta aplica colores y la barra de estado ayuda a reconocer la selección y los mensajes de una operación.

Las pestañas superiores corresponden a documentos. Las pestañas de página cambian la página del dibujo activo. Cambiar de documento no equivale a cambiar de página. Cada documento mantiene su selección e historial propios; un asterisco junto al nombre indica cambios sin guardar.

Para desplazarse, use la rueda y la herramienta de mano. `F4` ajusta el dibujo a la vista, `Shift+F4` encuadra la página y `Shift+F2` la selección. El zoom cambia la vista, no las dimensiones físicas del contenido.

Los nombres de menú pueden aparecer en castellano o inglés según el idioma elegido. Si una herramienta no aparece, consulte el espacio de trabajo y los paneles visibles antes de asumir que se ha perdido el documento.

## 4. Páginas, capas y orden de objetos

Organice un trabajo con varias caras o tamaños en páginas. Revise el tamaño de la página activa antes de exportar: algunos formatos generan una sola página y otros admiten el documento completo.

Las capas agrupan objetos y pueden estar ocultas o bloqueadas. Un objeto bloqueado no se edita hasta desbloquearlo; uno oculto puede seguir existiendo aunque no aparezca en pantalla. Para localizar contenido aparentemente perdido, revise primero visibilidad, bloqueo, página y orden de apilado.

En el panel Objetos puede seleccionar la capa de destino. Los objetos nuevos se dirigen a una capa editable según las reglas de la aplicación. Antes de importar un logotipo o una imagen, active la página y capa deseadas. Separe, por ejemplo, fondo, ilustración y texto para facilitar revisiones.

Use Agrupar para mover elementos como una unidad. Desagrupar recupera los objetos individuales. Combinar curvas y agrupar no son la misma operación: combinar cambia la estructura geométrica de los trazados. Antes de convertir texto o contornos a curvas, conserve una copia editable.

## 5. Dibujar formas y editar curvas

Las formas básicas incluyen rectángulos, elipses, polígonos, estrellas y curvas. Las herramientas heredadas de tres puntos, espirales y formas comunes amplían la construcción; su cobertura en esta edición se distingue en el apéndice.

En un rectángulo puede ajustar cada esquina. Compruebe el orden visual de las cuatro esquinas cuando use valores distintos. Un redondeo porcentual de macros se calcula sobre la mitad del lado menor; un radio físico es una longitud. En un rectángulo de 40 × 20 mm, un 50 % corresponde a 5 mm, no a 10 mm.

La herramienta Forma permite seleccionar nodos y manejadores de una curva. Un nodo de esquina y uno suave producen uniones diferentes. Para preparar un corte o un contorno cerrado, revise que el trazado esté cerrado y que no existan segmentos duplicados. El aspecto de pantalla no demuestra por sí solo que el trazado sea adecuado para una máquina.

Las operaciones de recorte, soldadura, intersección, combinación y separación están presentes en la base del editor. Aplíquelas sobre una copia y revise los nodos resultantes si el destino exige geometría exacta. No se ha verificado aquí cada variante de estas herramientas mediante interacción visible.

## 6. Seleccionar, transformar y alinear

Haga clic para seleccionar; añada objetos con Mayús. Un marco selecciona varios elementos. Los modificadores de arrastre dependen de la herramienta, por lo que conviene revisar la ayuda contextual antes de una transformación importante.

Los tiradores de selección permiten cambiar tamaño. Una segunda selección puede mostrar giro e inclinación. Para un resultado numérico, utilice los campos de posición, anchura, altura y ángulo. Revise el punto de referencia antes de escalar o colocar con coordenadas.

Para distribuir una serie de etiquetas, dibuje una, duplíquela, seleccione todas y use el panel de alineación y distribución. Compruebe el criterio elegido: página, selección u objeto de referencia. Una distribución uniforme de centros no equivale a una separación uniforme de los bordes.

`Ctrl+D` duplica; `Ctrl+R` repite la última acción repetible. El historial permite deshacer el resultado, pero un archivo exportado no conserva ese historial de edición.

## 7. Texto, tablas y fuentes

El texto artístico resulta práctico para títulos y rótulos. El texto de párrafo se organiza dentro de un marco. Mantenga fuentes disponibles y revise el resultado al abrir un archivo recibido: una sustitución de fuente puede cambiar saltos de línea, ancho y altura.

Para aplicar un estilo a una parte de un texto, seleccione los caracteres y después cambie la propiedad. Una selección de objeto y una selección de caracteres no tienen el mismo alcance. Las tablas permiten organizar contenido por celdas; revise dimensiones y saltos al exportar.

El editor dispone de importación de TXT, RTF y DOCX y de funciones heredadas de texto en trayecto, ajuste y escritura. No se garantiza que un documento de oficina complejo conserve todos sus estilos o que cada combinación tipográfica coincida con otro programa.

Antes de enviar un PDF, verifique si las fuentes se incluyen o si el texto se convierte en curvas según las opciones de exportación. Convertir a curvas evita depender de la fuente para esa copia, pero elimina la edición textual ordinaria. Conserve siempre el original editable.

## 8. Relleno, contorno y efectos

Un relleno pinta el interior; el contorno dibuja su borde. Puede haber rellenos uniformes, degradados, patrones y transparencias. Cuando un objeto parezca vacío, compruebe tanto relleno como contorno y opacidad.

Los efectos de la base incluyen mezclas, contornos, sombras, envolventes, distorsiones y extrusión. Algunas operaciones conservan parámetros editables y otras aplican un resultado. Esa diferencia importa al corregir después el trabajo o exportar a otro formato.

Para un intercambio delicado, exporte una muestra pequeña con el efecto y compárela visualmente. Si el destinatario solo necesita apariencia, una copia rasterizada puede ser útil; si necesita edición vectorial, pruebe la lectura del resultado y revise qué objetos siguen siendo editables.

No se equipara la presencia de un menú con la validación de todas sus opciones. El apéndice identifica familias heredadas que requieren pruebas adicionales en el flujo concreto del usuario.

## 9. Imágenes, color e ICC

Abrir una imagen crea un documento; importar la coloca en el dibujo activo. Revise resolución, tamaño físico y transparencia después de colocarla. Una imagen pequeña ampliada no gana detalle por aumentar sus DPI nominales.

En gestión de color, Asignar y Convertir cumplen funciones diferentes. Asignar conserva los valores de color y cambia su interpretación mediante el perfil; Convertir modifica valores para buscar una apariencia equivalente dentro de las posibilidades del perfil de destino. En datos CMYK retenidos se comprueban canales, alfa y perfiles en recorridos propios concretos.

Si hay un conflicto al importar o pegar contenido CMYK, la aplicación puede mantener la calibración del destino, asignarla al contenido o convertir el destino al perfil entrante según la política configurada. Revise el aviso y el resultado. Un error de perfil no equivale a una conversión correcta; conserve el archivo y los datos antes de insistir.

La validación de color utiliza perfiles y documentos de prueba propios. No constituye una prueba de impresión ni una comparación completa con el motor de color de otro editor. RGB, escala de grises, tintas especiales y combinaciones de estilos no tienen una garantía universal de conversión.

## 10. Unidades, reglas y macros

El dibujo interno utiliza milímetros. Las reglas y los campos pueden expresar las distancias en otras unidades. En el modelo de macros, `Document.Unit` controla las longitudes que interpreta el script. Cambiar la unidad de visualización no debe escalar físicamente los objetos.

La edición 214714 conserva la unidad de macros en TDRAW, pero pierde ese ajuste al reabrir CDR. La fuente alpha posterior incorpora ejes independientes y conservación privada CDR, todavía pendiente del paquete nuevo verificado. Otros programas pueden ignorar esa extensión; conserve TDRAW. No se atribuye esta mejora a 214714.

Las reglas H/V y la unidad de automatización son conceptos distintos. Un dibujo puede usar milímetros en macros, pulgadas en la regla horizontal y centímetros en la vertical. Una copia de documento nativa y el producto comprobado de la fase determinan qué preferencias sobreviven a cada recorrido.

No se han resuelto mediante evidencia primaria todas las equivalencias de píxel y cícero. Las convenciones actuales no acreditan las 17 escalas del programa de referencia. Si necesita una medida crítica, trabaje en milímetros o pulgadas y verifique la dimensión física de salida.

## 11. Abrir, importar, guardar y exportar

| Acción | Para qué sirve | Revisión recomendada |
|---|---|---|
| Abrir | Cargar un archivo como documento | Páginas, avisos, fuentes, perfiles y tamaño |
| Importar | Añadir contenido al documento activo | Capa, posición, calibración y recursos |
| Guardar TDRAW | Mantener la copia editable propia | Nombre, ubicación y cierre/reapertura |
| Guardar CDR | Intercambio y recorrido propio CDR | Versión, contenido, ajustes y lectura del destinatario |
| Exportar | Crear una copia para un destino concreto | Página, resolución, formato real y contenido |

La lectura incluye TDRAW, CDR, SVG/SVGZ, PDF, AI, EPS/PS, DXF, PSD, EMF/WMF, PLT, CMX, documentos de texto e imágenes compatibles. La cobertura de objetos varía por formato. CDR y CMX siguen siendo parciales; abrir un archivo no garantiza que todos sus recursos sean editables o idénticos al original.

La salida incluye SVG/SVGZ, PDF, AI compatible con PDF o AI 8, EPS, PS/PRN, DXF, EMF/WMF, PLT, PSD, HTML y los formatos de imagen PNG, JPEG, BMP, TIFF, WebP y GIF. CDR y TDRAW se guardan como documentos. Cada formato conserva capacidades distintas.

Para una imagen web, elija PNG si necesita transparencia o WebP si el destinatario lo admite. El modo WebP sin pérdida conserva los colores y alfa de la salida rasterizada; calidad baja no debe cuantizar sus colores en ese modo. JPEG usa compresión con pérdida y no conserva transparencia. GIF tiene una paleta limitada y no equivale a RGBA completo.

Para impresión, prepare PDF y compruebe tamaño de página, fuentes, transparencia y color en un visor. La existencia de exportación PDF no garantiza el resultado de todas las imprentas. Revise las especificaciones del destinatario.

En exportación rasterizada, los DPI y el tamaño físico determinan las dimensiones en píxeles. Los recorridos propios corrigen errores mínimos de redondeo cerca de un entero, sin reducir fracciones reales. La fidelidad visual y la conservación de dimensiones se miden por separado: un archivo válido puede presentar pérdidas.

## 12. Copiar entre instancias

La selección propia puede transportarse como SVG con un bloque nativo versionado que conserva recursos y datos del dibujo. Las pruebas de producto cubren dos procesos independientes, tres políticas CMYK, selección, deshacer/rehacer y datos completos de imágenes y perfiles.

El límite del modelo es de 128 grupos anidados. La recopia real a esa profundidad está comprobada en 210340 y posteriores ensayados; 129 se rechaza sin destruir el estado anterior. Los bloques propios dañados o de versión desconocida no se aceptan como una copia simplificada silenciosa.

Esas pruebas usaron bytes capturados y proveedores de prueba. No validan por sí solas el portapapeles de Windows, la interfaz visible ni el portapapeles del navegador. Si el intercambio del sistema falla, use un archivo TDRAW, SVG o una exportación adecuada como ruta alternativa y conserve los originales.

## 13. Automatización y asistentes

El servidor MCP permite abrir, guardar, importar, exportar y ejecutar scripts sin una ventana. Su uso es opcional para el usuario que dibuja. La guía técnica incluye configuración, herramientas y ejemplos.

Existen dos modelos de JavaScript. El modelo `native`, predeterminado, conserva la API propia en milímetros. `vgcore` utiliza nombres y convenciones del modelo de macros compatible de manera parcial. No son intercambiables por compartir un nombre de método.

Por ejemplo, `CreateRectangle(10,30,50,10)` crea una figura de 50 × 10 en el modelo propio. Con `vgcore` y unidad de milímetros, esos valores representan lados y producen 40 × 20. Elija el modelo explícitamente para evitar dibujos con dimensiones equivocadas.

Las macros VBA se compilan al motor de scripts propio; no ejecutan el programa original ni ofrecen una instalación COM equivalente. Pruebe las macros recibidas en copias de sus documentos. Las operaciones válidas completadas antes de un error de script pueden mantenerse en un paso de historial; un error no implica que nada haya cambiado.

## 14. Resolver problemas y recuperar trabajo

| Síntoma | Qué comprobar primero |
|---|---|
| El EXE no inicia o falta una DLL | Extracción completa y bibliotecas de la misma versión |
| El archivo abre con diferencias | Avisos, versión, fuentes, perfiles, efectos y cobertura parcial del formato |
| No aparece un objeto | Página activa, visibilidad, bloqueo, orden y zoom |
| Un tamaño de script es inesperado | Modelo `native`/`vgcore`, unidad y significado de los argumentos |
| Cambia la apariencia CMYK | Asignación frente a conversión, perfil disponible y política de entrada |
| La exportación no contiene lo esperado | Página seleccionada, extensión, resolución y formato elegido |
| Una operación falla después de editar | Revisar el historial y el documento antes de guardar sobre otra copia |

La base incluye copias automáticas y recuperación de sesiones, configurables en Opciones > Guardar. El comportamiento heredado usa archivos nativos de respaldo y puede ofrecer recuperarlos tras una salida anómala. Compruebe su carpeta e intervalo en la instalación que utiliza; no se debe confiar en una copia que nunca se ha generado y reabierto.

Ante una incidencia, anote versión del paquete, pasos exactos, mensaje de error y un archivo de prueba que pueda compartir. Evite enviar documentos con datos privados. Adjunte una comparación de la página esperada y la obtenida al mismo tamaño si el problema es visual.

## 15. Atajos de consulta rápida

Estos atajos proceden de las funciones documentadas de la base. No se han vuelto a recorrer todos mediante el teclado del escritorio del usuario.

| Atajo | Acción |
|---|---|
| Ctrl+N / Ctrl+O | Nuevo / Abrir |
| Ctrl+S / Ctrl+Mayús+S | Guardar / Guardar como |
| Ctrl+I / Ctrl+E | Importar / Exportar |
| Ctrl+Z / Ctrl+Mayús+Z | Deshacer / Rehacer |
| Ctrl+D / Ctrl+R | Duplicar / Repetir |
| Ctrl+G / Ctrl+U | Agrupar / Desagrupar |
| Ctrl+Q | Convertir a curvas |
| F4 / Mayús+F4 | Ajustar dibujo / Ajustar página |
| Mayús+F2 | Encuadrar selección |
| Alt+Enter | Propiedades |
| Esc | Cancelar la interacción actual cuando la herramienta lo permite |

## 16. Apéndice: cobertura y límites de esta edición

Validada significa comprobada en el recorrido indicado, no en todos sus usos. Heredada significa presente y documentada en la base, sin repetir aquí toda su interacción. Pendiente significa diferencia conocida, cobertura parcial o falta de prueba suficiente.

| Familia | Estado de evidencia de esta edición |
|---|---|
| Portable y DLL | Validada: módulos reales, integridad de paquete y ejecución sin ventana. La interfaz visible del escritorio no está ensayada. |
| Rectángulos y radios de macros | Validada: tres constructores, esquinas asimétricas, unidades y recorrido propio PNG/CDR con máscara geométrica. |
| TDRAW, historial y modelos MCP | Validada en escenarios propios; no equivalencia de toda la API. |
| Color CMYK, ICC e importación | Validada en muestras y políticas propias concretas; cobertura global parcial. |
| Copia nativa entre procesos | Validada con proveedores de prueba y límites de profundidad. Portapapeles real del sistema y navegador pendientes. |
| Selección, zoom, alineación y transformaciones | Heredada; consultar las notas de comportamiento para variantes y modificadores. |
| Curvas, nodos, recorte y operaciones de formas | Heredada; no todas las combinaciones se han validado en esta edición. |
| Texto, tablas, escritura y fuentes | Heredada con comprobaciones locales de recursos; no fidelidad tipográfica universal. |
| Degradados, patrones, efectos y transparencias | Heredada; cada intercambio debe comprobar su resultado. |
| Importación/exportación general | Códecs reales y recorridos propios comprobados; fidelidad y conservación varían por contenido. CDR/CMX parciales. |
| Impresión, preimpresión y mezcla de datos | Heredada o parcial; sin prueba de impresora ni flujo de imprenta completo. |
| Lápiz, tableta, escáner y complementos | Sin validación de hardware aquí; no hay un host general de filtros externos ni una pila de escáner propia. |
| Navegador y Android | Existen en la base; compilar WASM no certifica ejecución en navegador ni funcionamiento móvil. |
| Unidades CDR estándar, píxel y cícero | Pendientes de interoperabilidad y factores completos. La conservación por extensión propia se identifica por separado. |

La guía técnica describe las pruebas, arquitectura y contratos de automatización. Este manual no sustituye la comprobación de un documento real en el programa receptor ni certifica una réplica completa del programa de referencia.
