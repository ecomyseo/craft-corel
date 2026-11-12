# Craft Plumilla

Editor vectorial independiente en Rust, en desarrollo, derivado de
[TraceDraw](https://github.com/tracedraw/tracedraw) bajo licencia
**MIT o Apache-2.0**. Se conservan sus autores, avisos y atribuciones.

La meta es ofrecer herramientas, opciones e intercambio de archivos equivalentes a
los de CorelDRAW 2026 (versión 27.2.0.135) y abrir y guardar sus archivos CDR.
**La equivalencia total y la compatibilidad del 100 % todavía no están conseguidas
ni verificadas.**

> **Aviso de marcas.** Craft Plumilla es un proyecto independiente: no tiene
> relación con Corel Corporation ni con Alludo, ni está patrocinado ni respaldado
> por ellas. CorelDRAW y Corel son marcas registradas de Corel Corporation; se
> nombran solo para indicar con qué archivos y programa es compatible.

## Descarga

En [Releases](https://github.com/ecomyseo/craft-plumilla/releases) están el
instalador de Windows de 64 bits y la versión portable en ZIP. El instalador se
instala para el usuario actual sin pedir permisos de administrador, no se apropia
de los archivos `.cdr` y solo ofrece abrir los `.tdraw` si se marca esa opción.
Las preferencias de una copia anterior con el nombre antiguo se trasladan solas.

## Código y compilación

El proyecto Rust completo está en [`craft-plumilla`](craft-plumilla).
Requiere Rust 1.97.0; en Windows, el compilador y enlazador de C++ para MSVC.

```sh
rustup toolchain install 1.97.0 --profile minimal
cargo +1.97.0 build --locked --release --manifest-path craft-plumilla/Cargo.toml -p craft-plumilla
```

El ejecutable resultante es `craft-plumilla`. El modo `craft-plumilla --mcp`
funciona sin ventanas y ofrece edición, apertura, guardado, exportación y scripts.
La interfaz usa egui/eframe; el modelo trabaja en milímetros.

Los scripts propios para compilar y empaquetar la distribución con EXE y DLL
están en [`utiles`](utiles). Ese flujo usa una instalación local de Rust 1.97.0
en `.tools`, cachés de dependencias y registros de construcción; no se incluyen
esas cachés ni binarios en el repositorio. El paquete modular exige conservar
las DLL junto al EXE y usar los módulos de la misma construcción. El instalador se
genera con Inno Setup a partir de
[`packaging/windows/craft-plumilla.iss`](craft-plumilla/packaging/windows/craft-plumilla.iss).

## Alcance actual

- Edición vectorial, páginas, capas, curvas, texto y efectos procedentes de la
  base abierta. Su inventario está en
  [`docs/features.md`](craft-plumilla/docs/features.md); las declaraciones
  heredadas están separadas de las comprobaciones de esta variante.
- Formato editable propio `.tdraw` y lectura/escritura CDR con límites. La
  conservación entre nuestros programas no demuestra interoperabilidad completa
  con otro editor.
- Dos modelos de scripts diferenciados: API nativa en milímetros y modelo
  parcial de macros mediante `object_model: "vgcore"`. No es un servidor COM.
- Menús del botón derecho de página, escritorio, objeto, estilos, ortografía,
  mezcla y malla (con tiradores y tipos de nodo), rollovers con tres estados y
  adquisición de imágenes por WIA.

No se distribuyen ejecutables, DLL, recursos ni código propietario de CorelDRAW.
Un ejecutable sin firma puede recibir avisos de Windows; las pruebas de código no
garantizan reputación de SmartScreen.

## Documentación y comprobaciones

Los manuales de usuario (con capturas) y técnico en HTML y PDF están en
[`docs/manual-craft-plumilla`](craft-plumilla/docs/manual-craft-plumilla).
Las pruebas y la auditoría distinguen ejecución local, datos propios,
documentación de referencia y aspectos pendientes de verificar.

```sh
cargo +1.97.0 test --locked --manifest-path craft-plumilla/Cargo.toml -p tracedraw-core --lib
cargo +1.97.0 test --locked --manifest-path craft-plumilla/Cargo.toml -p tracedraw-cdr --test document_units
cargo +1.97.0 test --locked --manifest-path craft-plumilla/Cargo.toml -p craft-plumilla --lib document_unit_tests -- --test-threads=1
```

Se usa [Craft Anything](https://github.com/storytold/craft-anything) como
herramienta auxiliar de investigación y comprobación en el entorno de desarrollo.
Craft Plumilla no es una publicación oficial de sus desarrolladores. El código del
editor procede de la base abierta indicada y de las modificaciones de esta variante.

## Licencias

[MIT](LICENSE-MIT) o [Apache-2.0](LICENSE-APACHE), con los avisos de terceros en
[`ATTRIBUTION.md`](craft-plumilla/ATTRIBUTION.md) y los textos de licencia
que acompañan a las dependencias incluidas en `vendor`.
