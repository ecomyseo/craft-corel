# Auditoría de la versión en desarrollo

La revisión de publicación está en curso. Este documento se actualizará con los
resultados finales; no constituye un certificado de seguridad ni acredita
compatibilidad completa con el programa de referencia.

La auditoría del ejecutable detectó agotamiento de pila al abrir un TDRAW propio
de 40 grupos anidados. Ese candidato se rechazó. El código incorpora una lectura
nativa en un trabajador con pila acotada, comprobando primero los límites de
entrada y profundidad; la pila principal del ejecutable sigue siendo de 1 MiB.
La prueba desde un llamador de 1 MiB pasa. Está pendiente repetir la reproducción
completa en el nuevo paquete, incluida la profundidad máxima admitida.

El primer código publicado incorpora las preferencias independientes de macros
y reglas y el suplemento privado de unidades CDR. La prueba de conservación
entre nuestros programas se distingue de la interoperabilidad con otro editor.

La consulta primaria de OSV sobre las 555 dependencias de registro del archivo
de versiones identificó un aviso de seguridad de memoria en Tokio 1.53.2 para
Unix; la versión fijada se ha actualizado a 1.53.3. Se están repitiendo las
comprobaciones. Los avisos de falta de mantenimiento de `paste`, `rustybuzz` y
`ttf-parser` siguen abiertos y no se presentan como resueltos.

La revisión estática señaló el actualizador heredado, las cabeceras HTTP locales
y presupuestos insuficientes para archivos y datos comprimidos. Se han añadido
guardas de actualización, cotas HTTP y límites de recursos. Falta completar la
validación de estos cambios y cerrar la lectura inicial de la lista de flujos
CDR y las rutas de lectura de las bibliotecas que usa la CLI.

No se han ejecutado el programa de referencia ni acciones sobre el escritorio
o el portapapeles del sistema. Las capturas nuevas de reglas proceden de la
pintura real de egui, rasterizada sin interfaz visible; no son capturas del
programa original.

Las licencias y atribuciones de la base y del código incluido en `vendor` se
conservan. El análisis inicial de patrones de credenciales no encontró claves
reales. La comprobación de UTF-8 documenta siete líneas intencionales: ejemplos
de reparación de texto y mayúsculas portuguesas válidas; las excepciones se
verifican por archivo, línea y huella exacta.

Quedan pendientes los resultados finales de pruebas, el análisis antivirus de
la entrega y la revisión del estado servido en GitHub. La aplicación y sus
ejecutables siguen siendo una versión en desarrollo.
