# Manuales de Craft Corel

Documentación en castellano de la aplicación independiente basada en TraceDraw. Cada manual tiene una fuente Markdown única; el generador produce HTML autónomo y PDF con el mismo contenido.

Repositorio del proyecto: [ecomyseo/craft-corel](https://github.com/ecomyseo/craft-corel).

| Documento | Lectura en navegador | PDF | Fuente |
|---|---|---|---|
| Manual de usuario | [HTML con búsqueda](manual-usuario.html) | [PDF](manual-usuario.pdf) | [Markdown](manual-usuario.md) |
| Guía técnica | [HTML con búsqueda](guia-tecnica.html) | [PDF](guia-tecnica.pdf) | [Markdown](guia-tecnica.md) |

El HTML funciona sin servicios externos: ofrece índice, búsqueda de apartados y estilo de impresión. Pulse `/` para buscar y `Esc` para limpiar. El PDF incluye índice enlazado y marcadores.

La edición identifica las funciones con evidencia de producto, las capacidades heredadas y los límites pendientes. No certifica compatibilidad completa con el programa de referencia ni una instalación original. Consulte la edición y el alcance descritos al comienzo de cada manual.

Para regenerar, instale en su entorno de documentación Python con `reportlab`, `markdown`, `beautifulsoup4`, `pymupdf` y `Pillow`, y ejecute `generar.py --evidence` con la zona de evidencia configurada en el generador del espacio de trabajo. La construcción no necesita abrir la aplicación ni compilar Rust. La tipografía PDF es Bitstream Vera; su licencia se conserva en `LICENCIA-TIPOGRAFIA.txt`.

Los informes de comprobación y renders pertenecen a la evidencia de publicación, fuera de esta carpeta de documentos. No se incorporan capturas del programa original ni recursos propietarios de ejercicios o tutoriales.
