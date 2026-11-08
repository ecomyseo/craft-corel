# Craft Corel

Editor vectorial independiente en Rust, en desarrollo, derivado de
[TraceDraw](https://github.com/tracedraw/tracedraw) bajo licencia
**MIT o Apache-2.0**. Se conservan sus autores, avisos y atribuciones.

El objetivo es ampliar sus herramientas e intercambio de archivos tomando como
referencia CorelDRAW 2026, versión 27.2.0.135. **La equivalencia total y la
compatibilidad del 100 % todavía no están conseguidas ni verificadas.**
El proyecto es independiente y no está afiliado ni respaldado por Corel.

## Código y compilación

El proyecto Rust completo está en [`craft-corel-nativo`](craft-corel-nativo).
Requiere Rust 1.97.0; en Windows, el compilador y enlazador de C++ para MSVC.

```sh
rustup toolchain install 1.97.0 --profile minimal
cargo +1.97.0 build --locked --release --manifest-path craft-corel-nativo/Cargo.toml -p craft-corel-nativo
```

El ejecutable resultante es `craft-corel`. El modo `craft-corel --mcp` funciona
sin ventanas y ofrece edición, apertura, guardado, exportación y scripts.
La interfaz usa egui/eframe; el modelo trabaja en milímetros.

Los scripts propios para compilar y empaquetar la distribución con EXE y DLL
están en [`utiles`](utiles). Ese flujo usa una instalación local de Rust 1.97.0
en `.tools`, cachés de dependencias y registros de construcción; no se incluyen
esas cachés ni binarios en el repositorio. El paquete modular exige conservar
las DLL junto al EXE y usar los módulos de la misma construcción.

## Alcance actual

- Edición vectorial, páginas, capas, curvas, texto y efectos procedentes de la
  base abierta. Su inventario está en
  [`docs/features.md`](craft-corel-nativo/docs/features.md); las declaraciones
  heredadas están separadas de las comprobaciones de esta variante.
- Formato editable propio `.tdraw` y lectura/escritura CDR con límites. La
  conservación entre nuestros programas no demuestra interoperabilidad completa
  con otro editor.
- Dos modelos de scripts diferenciados: API nativa en milímetros y modelo
  parcial de macros mediante `object_model: "vgcore"`. No es un servidor COM.
- Preferencias independientes para la unidad de macros y las dos reglas.
  La conservación de esos ajustes en CDR utiliza un suplemento privado;
  no se presenta como una implementación de las unidades estándar del formato.

No se distribuyen ejecutables, DLL, recursos ni código propietario de CorelDRAW.
Este repositorio contiene fuentes; la distribución final y su instalador siguen
pendientes del objetivo completo. Un ejecutable sin firma puede recibir avisos
de Windows; las pruebas de código no garantizan reputación de SmartScreen.

## Documentación y comprobaciones

Los manuales de usuario y técnico en HTML/PDF se incorporan en
[`docs/manual-craft-corel`](craft-corel-nativo/docs/manual-craft-corel).
Las pruebas y la auditoría distinguen ejecución local, datos propios,
documentación de referencia y aspectos pendientes de verificar.

```sh
cargo +1.97.0 test --locked --manifest-path craft-corel-nativo/Cargo.toml -p tracedraw-core --lib
cargo +1.97.0 test --locked --manifest-path craft-corel-nativo/Cargo.toml -p tracedraw-cdr --test document_units
cargo +1.97.0 test --locked --manifest-path craft-corel-nativo/Cargo.toml -p craft-corel-nativo --lib document_unit_tests -- --test-threads=1
```

Se usa [Craft Anything](https://github.com/storytold/craft-anything) como
herramienta auxiliar de investigación y comprobación en el entorno de desarrollo.
Craft Corel no es una publicación oficial de sus desarrolladores. El código del
editor procede de la base abierta indicada y de las modificaciones de esta variante.

## Licencias

[MIT](LICENSE-MIT) o [Apache-2.0](LICENSE-APACHE), con los avisos de terceros en
[`ATTRIBUTION.md`](craft-corel-nativo/ATTRIBUTION.md) y los textos de licencia
que acompañan a las dependencias incluidas en `vendor`.
