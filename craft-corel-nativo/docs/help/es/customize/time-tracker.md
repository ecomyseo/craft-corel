# Control de tiempo

El control de tiempo registra el tiempo que dedica a un dibujo, tarea por
tarea, y guarda los registros en el dibujo (en los archivos `.tdraw`). El
tiempo solo cuenta mientras trabaja: tras unos minutos de inactividad, el
reloj espera, y su siguiente acción lo pone en marcha de nuevo.

## La barra de herramientas Control de tiempo {#toolbar}

[[menu.window > menu.window.toolbars > menu.window.toolbar_time_tracker]]
muestra una barra con:

- la lista de tareas, para elegir la tarea activa o añadir una nueva;
- el tiempo de la tarea activa;
- el botón de grabar y pausar;
- una luz: roja mientras cuenta, ámbar en inactividad, gris en pausa o sin
  tarea;
- el total de todas las tareas;
- un botón que abre el cuadro de diálogo.

Poner en marcha el reloj en un dibujo sin tareas añade "Tarea 1".

## El cuadro de diálogo Control de tiempo {#dialog}

[[menu.tools > menu.tools.time_tracker]] enumera las tareas con su tiempo, su
inicio y su fin, y el total.

![El cuadro de diálogo Control de tiempo](../../img/customize-time-tracker.png)

- **Nueva tarea**, **Eliminar**, **Activar** y **Restablecer tiempo** actúan
  sobre la tarea seleccionada. Haga doble clic en una tarea para activarla.
- **Grabar** y **Pausar** ponen en marcha y detienen el reloj.
- Edite el **nombre**, el **tiempo** (como `1:30:00`, `1:30` o `1.5` horas),
  el **inicio** y el **fin** (como `2026-10-19 14:30`) de la tarea
  seleccionada.
- **Dejar de contar tras N minutos sin actividad** fija el tiempo de
  inactividad (5 minutos de forma predeterminada).
- **Exportar CSV** y **Exportar texto** guardan los registros, para una hoja
  de cálculo o un informe.

Registrar tiempo no es una edición del dibujo: no crea ningún paso de
deshacer. Los registros se guardan la próxima vez que se guarde el dibujo.
