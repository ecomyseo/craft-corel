# Time Tracker

Records the time spent on a drawing, task by task, and keeps the records
in the drawing (`.tdraw` files; `.cdr` files do not carry them). Code:
`time_tracker.rs` (records, clock, exports) and `ui/time_tracker.rs`
(toolbar and dialog).

## Records

- A drawing has a list of tasks and at most one active task. A task has a
  name, its recorded time (seconds), a start and a finish (moments, shown
  and typed in the local time zone).
- Recording time on the active task adds to its time, sets its start the
  first time (the moment the counted time began) and moves its finish to
  the moment of recording.
- Recording is not an edit of the drawing: it makes no undo step, Undo
  and Redo leave the records alone and it does not mark the drawing as
  changed. The records are written with the drawing's next save. Editing
  the records by hand (adding, renaming or deleting a task, typing a
  time or a date, changing the active task) marks the drawing changed,
  still without an undo step.
- A drawing without tasks saves no records; files from before the Time
  Tracker open with none.

## The clock

- One clock serves every open drawing: while it runs, time goes to the
  active task of the drawing in front. A drawing without an active task
  records nothing (the light shows "No task").
- Time counts only while someone works: up to the idle time (5 minutes by
  default, 1 to 120) after the last input in the window (pointer moves,
  clicks, keys, the wheel, touch). After that the clock waits; the next
  input starts it again without adding the break.
- Counted time is written to the task every 10 seconds, when the clock is
  paused, before the active task changes, and before the drawing is
  saved, closed or left for another tab.
- Starting the clock on a drawing without tasks adds "Task 1"; with tasks
  but none active, the first becomes active.
- While the clock counts, the window repaints once a second so the time
  shown stays current.

## Toolbar

Window > Toolbars > Time Tracker shows a band under the other toolbars
(kept in the preferences, `time_tracker.show_bar`):

1. A stopwatch.
2. The task list (180 px): the drawing's tasks, the active one shown;
   picking one makes it active; New Task adds "Task n" (the first free
   number) and makes it active. "No task" when there is none.
3. The active task's time as h:mm:ss in a monospaced font ("-:--:--"
   without one).
4. The record button: a triangle to start (tooltip Record), two bars to
   pause (Pause).
5. The light: red while counting, amber while idle, grey when paused or
   when the drawing has no active task (the tooltip says which).
6. "Total: h:mm:ss", every task together.
7. Tasks..., which opens the dialog.

Everything but the stopwatch is greyed on the Welcome Screen.

## Dialog

Tools > Time Tracker... or the toolbar's Tasks... button:

- The tasks in a white list under a grey header: a blue dot at the active
  task, then Task, Time (right-aligned), Start and Finish. A click selects
  a row, a double click makes its task active. Eight rows show before the
  list scrolls; an empty list says how to add a task. The total in bold
  sits under the list. The active task's time includes the seconds not yet
  written.
- New Task, Delete, Make Active and Reset Time (time, start and finish
  back to empty) act on the selected task; then Record or Pause and the
  light with its state.
- The selected task's Name, Time, Start and Finish fields. A typed value
  applies when the field is left or Enter is pressed: a time as h:mm:ss,
  h:mm or hours ("1.5" or "1,5"); a date as YYYY-MM-DD HH:MM (seconds
  allowed) or YYYY-MM-DD (midnight); an empty date clears it. An empty name
  or an unreadable value is dropped and the field shows the kept value.
  Selecting another task, Close, Record or Pause and the exports apply
  what was typed first; Esc and the close button drop it.
- "Stop counting after N minutes without input" sets the idle time
  (preferences, `time_tracker.idle_minutes`).
- Export CSV... and Export Text... save the records where the user
  chooses (a download in the browser), named "<drawing> time.csv" or
  ".txt"; the status bar tells where they went.
- Close.

## Exports

- CSV: a header (Task, Time, Hours, Start, Finish in the interface
  language), one line per task and a Total line; the time as h:mm:ss,
  hours with two decimals and a point, dates in local time; fields with a
  comma, a quote or a line break are quoted (quotes doubled); lines end in
  CR LF.
- Text: "Time records: <drawing>", "Written <date>", a blank line, then a
  table with Task, Time, Start and Finish in columns two spaces apart (the
  times right-aligned), a rule of dashes and the total.

## Checks

- `tracedraw_core` `document::tests::time_tracker_round_trips_and_is_left_out_when_empty`
- `tracedraw_core` `engine::tests::time_tracker_stays_out_of_the_history`
- `tracedraw_core` `engine::tests::recording_moves_the_start_and_finish`
- `time_tracker::tests::the_clock_counts_work_and_stops_in_a_break`
- `time_tracker::tests::durations_read_and_write`
- `time_tracker::tests::dates_read_and_write_in_a_time_zone`
- `time_tracker::tests::csv_quotes_names_and_adds_the_unwritten_time`
- `time_tracker::tests::the_text_report_lines_up_its_columns`
- `time_tracker::tests::new_tasks_take_the_first_free_number`
- `time_tracker::tests::recording_goes_to_the_active_task_without_an_undo_step`
- `time_tracker::tests::leaving_a_drawing_writes_its_time_first`
- `ui::time_tracker::tests::typed_fields_apply_to_their_task_and_bad_ones_are_dropped`
- `ui::time_tracker::tests::the_light_tells_what_the_clock_does`
- `ui::time_tracker::tests::the_toolbar_and_dialog_draw`
