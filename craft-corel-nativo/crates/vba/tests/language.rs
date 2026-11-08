//! VBA statements and expressions, run on boa.

mod common;
use common::{run, run_err, run_modules};
use tracedraw_vba::{compile, list_macros, Module, ModuleKind};

#[test]
fn hello_and_arithmetic() {
    let out = run(r##"
Sub Main()
    Debug.Print "Hello"
    Debug.Print 1 + 2 * 3, 7 \ 2; 7 Mod 3
    Debug.Print 2 ^ 3 ^ 2, -2 ^ 2
    Debug.Print 10 / 4
    Debug.Print "a" & 1 & True
End Sub
"##);
    assert_eq!(out[0], "Hello");
    assert_eq!(out[1], "7             31");
    assert_eq!(out[2], "64            -4");
    assert_eq!(out[3], "2.5");
    assert_eq!(out[4], "a1True");
}

#[test]
fn identifiers_ignore_case() {
    let out = run(r##"
Option Explicit
Sub Main()
    Dim Counter As Integer
    counter = 5
    COUNTER = COUNTER + 1
    debug.print CoUnTeR
    Debug.Print ucase("abc"), LEN("abcd")
End Sub
"##);
    assert_eq!(out[0], "6");
    assert_eq!(out[1], "ABC           4");
}

#[test]
fn integer_variables_round_like_banks() {
    let out = run(r##"
Sub Main()
    Dim i As Integer, l As Long, s As String, b As Boolean
    i = 2.5: Debug.Print i
    i = 3.5: Debug.Print i
    l = -2.5: Debug.Print l
    s = 12.5: Debug.Print s
    b = 5: Debug.Print b
    Debug.Print CInt(True), CLng("42"), Round(2.675, 2), Round(0.5), Round(1.5)
    Debug.Print Int(-2.5), Fix(-2.5)
End Sub
"##);
    assert_eq!(out[0], "2");
    assert_eq!(out[1], "4");
    assert_eq!(out[2], "-2");
    assert_eq!(out[3], "12.5");
    assert_eq!(out[4], "True");
    assert_eq!(
        out[5],
        "-1            42            2.68          0             2"
    );
    assert_eq!(out[6], "-3            -2");
}

#[test]
fn if_select_and_loops() {
    let out = run(r##"
Sub Main()
    Dim i, total, s
    For i = 1 To 10 Step 3
        total = total + i
    Next i
    Debug.Print total
    For i = 5 To 1 Step -2: s = s & i: Next
    Debug.Print s
    i = 0
    Do While i < 3
        i = i + 1
    Loop
    Debug.Print i
    Do
        i = i - 1
    Loop Until i <= 0
    Debug.Print i
    While i < 2
        i = i + 1
    Wend
    Debug.Print i
    For i = 1 To 100
        If i = 4 Then Exit For
    Next
    Debug.Print i
    If i > 3 Then Debug.Print "big" Else Debug.Print "small"
    If i = 1 Then
        Debug.Print "one"
    ElseIf i = 4 Then
        Debug.Print "four"
    Else
        Debug.Print "other"
    End If
    Select Case i
        Case 1, 2: Debug.Print "low"
        Case 3 To 5: Debug.Print "mid"
        Case Is > 5: Debug.Print "high"
        Case Else: Debug.Print "none"
    End Select
    Select Case "b"
        Case "a": Debug.Print "A"
        Case Else: Debug.Print "else"
    End Select
End Sub
"##);
    assert_eq!(
        out,
        vec!["22", "531", "3", "0", "2", "4", "big", "four", "mid", "else"]
    );
}

#[test]
fn nested_loops_closed_by_one_next() {
    let out = run(r##"
Sub Main()
    Dim i, j, n
    For i = 1 To 3
    For j = 1 To 2
        n = n + 1
    Next j, i
    Debug.Print n
End Sub
"##);
    assert_eq!(out, vec!["6"]);
}

#[test]
fn do_loops_exit_and_until() {
    let out = run(r##"
Sub Main()
    Dim n
    Do Until n = 5
        n = n + 1
        If n = 3 Then Exit Do
    Loop
    Debug.Print n
    Do
        n = n + 10
    Loop While n < 30
    Debug.Print n
End Sub
"##);
    assert_eq!(out, vec!["3", "33"]);
}

#[test]
fn functions_byref_byval_optional_and_named_arguments() {
    let out = run(r##"
Function Twice(ByVal x As Long) As Long
    Twice = x * 2
End Function

Sub Bump(n)
    n = n + 1
End Sub

Sub NoBump(ByVal n)
    n = n + 1
End Sub

Function Greet(Optional name As String = "World", Optional punct) As String
    If IsMissing(punct) Then punct = "!"
    Greet = "Hello, " & name & punct
End Function

Function Fact(n As Long) As Long
    If n <= 1 Then
        Fact = 1
    Else
        Fact = n * Fact(n - 1)
    End If
End Function

Function Total(ParamArray items()) As Double
    Dim v
    For Each v In items
        Total = Total + v
    Next
End Function

Sub Main()
    Dim a As Long, arr(3) As Long
    a = 5
    Bump a
    Debug.Print a
    NoBump a
    Debug.Print a
    Call Bump(a)
    Debug.Print a
    Bump (a)
    Debug.Print a
    arr(2) = 10
    Bump arr(2)
    Debug.Print arr(2)
    Debug.Print Twice(21), Fact(5)
    Debug.Print Greet(), Greet("VBA"), Greet(punct:="?", name:="you")
    Debug.Print Total(1, 2, 3.5)
End Sub
"##);
    assert_eq!(out[0], "6");
    assert_eq!(out[1], "6");
    assert_eq!(out[2], "7");
    assert_eq!(out[3], "7");
    assert_eq!(out[4], "11");
    assert_eq!(out[5], "42            120");
    assert_eq!(out[6], "Hello, World! Hello, VBA!   Hello, you?");
    assert_eq!(out[7], "6.5");
}

#[test]
fn arrays_bounds_redim_and_erase() {
    let out = run(r##"
Option Base 1
Sub Main()
    Dim a(3) As Integer, b(0 To 2, 1 To 2), c() As String, v
    Debug.Print LBound(a), UBound(a)
    a(1) = 1.6: a(3) = 3
    Debug.Print a(1) + a(3)
    b(2, 2) = "x"
    Debug.Print b(2, 2), UBound(b, 1), LBound(b, 2)
    ReDim c(1 To 2)
    c(1) = "p": c(2) = "q"
    ReDim Preserve c(1 To 3)
    c(3) = "r"
    Debug.Print Join(c, "-")
    v = Array(10, 20, 30)
    Debug.Print LBound(v), v(2)
    Dim d
    d = Split("a,b,c", ",")
    Debug.Print LBound(d), UBound(d), d(1)
    Erase a
    Debug.Print a(3)
    Dim copy1, copy2
    copy1 = Array(1, 2)
    copy2 = copy1
    copy2(1) = 99
    Debug.Print copy1(1), copy2(1)
End Sub
"##);
    assert_eq!(out[0], "1             3");
    assert_eq!(out[1], "5");
    assert_eq!(out[2], "x             2             1");
    assert_eq!(out[3], "p-q-r");
    assert_eq!(out[4], "1             20");
    assert_eq!(out[5], "0             2             b");
    assert_eq!(out[6], "0");
    assert_eq!(out[7], "1             99");
}

#[test]
fn subscript_out_of_range_names_the_line() {
    let e = run_err("Sub Main()\n    Dim a(2)\n    a(5) = 1\nEnd Sub\n");
    assert!(
        e.contains("Run-time error '9': Subscript out of range"),
        "{e}"
    );
    assert!(e.contains("Module1.Main, line 3"), "{e}");
}

#[test]
fn on_error_resume_next_and_err() {
    let out = run(r##"
Sub Main()
    Dim x
    On Error Resume Next
    x = 1 / 0
    Debug.Print Err.Number, Err.Description
    Err.Clear
    Debug.Print Err.Number
    Err.Raise 1000, "Me", "custom"
    Debug.Print Err.Number; " "; Err.Description
    On Error GoTo 0
    Debug.Print "after"
End Sub
"##);
    assert_eq!(out[0], "11            Division by zero");
    assert_eq!(out[1], "0");
    assert_eq!(out[2], "1000 custom");
    assert_eq!(out[3], "after");
}

#[test]
fn on_error_goto_handler_and_resume_next() {
    let out = run(r##"
Sub Main()
    On Error GoTo Handler
    Debug.Print "start"
    Err.Raise 5
    Debug.Print "resumed"
    Exit Sub
Handler:
    Debug.Print "caught " & Err.Number
    Resume Next
End Sub
"##);
    assert_eq!(out, vec!["start", "caught 5", "resumed"]);
}

#[test]
fn errors_propagate_to_the_caller_handler() {
    let out = run(r##"
Sub Inner()
    Dim c As Collection
    c.Add 1
End Sub

Sub Main()
    On Error GoTo Fail
    Inner
    Debug.Print "not here"
    Exit Sub
Fail:
    Debug.Print Err.Number & ": " & Err.Description
End Sub
"##);
    assert_eq!(
        out,
        vec!["91: Object variable or With block variable not set"]
    );
}

#[test]
fn goto_jumps_between_top_level_labels() {
    let out = run(r##"
Sub Main()
    Dim n
    n = 0
Again:
    n = n + 1
    If n < 3 Then GoTo Again
    Debug.Print n
End Sub
"##);
    assert_eq!(out, vec!["3"]);
}

#[test]
fn end_stops_quietly() {
    let out = run("Sub Main()\n Debug.Print 1\n End\n Debug.Print 2\nEnd Sub");
    assert_eq!(out, vec!["1"]);
}

#[test]
fn with_blocks_nest_and_reach_members() {
    let out = run(r##"
Type Point
    X As Double
    Y As Double
End Type

Sub Main()
    Dim p As Point, q As Point
    With p
        .X = 3
        .Y = 4
        Debug.Print Sqr(.X ^ 2 + .Y ^ 2)
    End With
    q = p
    q.X = 10
    Debug.Print p.X, q.X
End Sub
"##);
    assert_eq!(out, vec!["5", "3             10"]);
}

#[test]
fn collections_and_dictionaries() {
    let out = run(r##"
Sub Main()
    Dim c As New Collection, d, k, item
    c.Add "one"
    c.Add "two", "second"
    c.Add "zero", Before:=1
    Debug.Print c.Count, c(1), c("SECOND"), c.Item(3)
    For Each item In c
        Debug.Print item;
    Next
    Debug.Print
    c.Remove 1
    Debug.Print c.Count
    Set d = CreateObject("Scripting.Dictionary")
    d.Add "a", 1
    d("b") = 2
    d("a") = d("a") + 10
    Debug.Print d.Count, d("a"), d.Exists("b"), d.Exists("z")
    For Each k In d.Keys
        Debug.Print k & "=" & d(k)
    Next
End Sub
"##);
    assert_eq!(out[0], "3             zero          two           two");
    assert_eq!(out[1], "zeroonetwo");
    assert_eq!(out[2], "2");
    assert_eq!(out[3], "2             11            True          False");
    assert_eq!(out[4], "a=11");
    assert_eq!(out[5], "b=2");
}

#[test]
fn strings_and_comparisons() {
    let out = run(r##"
Sub Main()
    Dim s As String
    s = "Hello World"
    Debug.Print Left(s, 5), Right(s, 5), Mid(s, 7), Mid(s, 7, 3)
    Debug.Print InStr(s, "o"), InStr(6, s, "o"), InStr(1, s, "WORLD", vbTextCompare), InStrRev(s, "o")
    Debug.Print Replace(s, "o", "0"), Trim("  x  ") & "|", StrReverse("abc")
    Debug.Print "a" < "b", "B" < "a", 10 < 9, "10" = 10, "abc" Like "a*", "a1" Like "a#"
    Debug.Print StrComp("a", "A"), StrComp("a", "A", vbTextCompare), Space(2) & "x", String(3, "z")
    Mid(s, 1, 5) = "HOWDY"
    Debug.Print s
    Debug.Print Chr(65) & Asc("a"), LCase("ABC"), Len(12345)
    Debug.Print Val("  12abc"), Val("&HFF"), Str(5), CStr(0.1 + 0.2)
End Sub
"##);
    assert_eq!(out[0], "Hello         World         World         Wor");
    assert_eq!(out[1], "5             8             7             8");
    assert_eq!(out[2], "Hell0 W0rld   x|            cba");
    assert_eq!(
        out[3],
        "True          True          False         True          True          True"
    );
    assert_eq!(out[4], "1             0               x           zzz");
    assert_eq!(out[5], "HOWDY World");
    assert_eq!(out[6], "A97           abc           5");
    assert_eq!(out[7], "12            255            5            0.3");
}

#[test]
fn option_compare_text_makes_comparisons_ignore_case() {
    let out = run(r##"
Option Compare Text
Sub Main()
    Debug.Print "abc" = "ABC", InStr("Hello", "L"), "A" Like "a"
End Sub
"##);
    assert_eq!(out, vec!["True          3             True"]);
}

#[test]
fn formats_numbers_and_dates() {
    let out = run(r##"
Sub Main()
    Dim d As Date
    Debug.Print Format(1234.5, "#,##0.00"), Format(0.256, "0.0%"), Format(5, "000"), Format(3.14159, "0.00")
    Debug.Print Format(-1.5, "0.0;(0.0)"), Format(0, "0;-0;zero"), Format(1234567, "#,##0"), Format(12345, "0.00E+00")
    d = #3/7/2024 14:05:09#
    Debug.Print Format(d, "yyyy-mm-dd"), Format(d, "dd/mm/yy hh:nn:ss"), Format(d, "mmmm d, yyyy"), Format(d, "h:mm AM/PM")
    Debug.Print Year(d), Month(d), Day(d), Hour(d), Minute(d), Second(d), Weekday(d)
    Debug.Print DateSerial(2024, 2, 30), DateAdd("m", 1, #1/31/2024#), DateDiff("d", #1/1/2024#, #3/1/2024#)
    Debug.Print CStr(#1/2/2003#), Format("text", ">"), Format(True, "Yes/No")
End Sub
"##);
    assert_eq!(out[0], "1,234.50      25.6%         005           3.14");
    assert_eq!(out[1], "(1.5)         zero          1,234,567     1.23E+04");
    assert_eq!(
        out[2],
        "2024-03-07    07/03/24 14:05:09           March 7, 2024 2:05 PM"
    );
    assert_eq!(
        out[3],
        "2024          3             7             14            5             9             5"
    );
    assert_eq!(out[4], "3/1/2024      2/29/2024     60");
    assert_eq!(out[5], "1/2/2003      TEXT          Yes");
}

#[test]
fn type_checks_and_conversions() {
    let out = run(r##"
Sub Main()
    Dim v, o As Object, c As New Collection
    Debug.Print TypeName(v), TypeName(1), TypeName(100000), TypeName(1.5), TypeName("s")
    Debug.Print IsEmpty(v), IsNull(Null), IsNumeric("12"), IsNumeric("x"), IsArray(Array())
    Debug.Print o Is Nothing, TypeName(c), TypeOf c Is Collection, VarType("x")
    Debug.Print IIf(1 > 2, "a", "b"), Choose(2, "x", "y", "z"), Switch(False, 1, True, 2)
    Debug.Print Hex(255), Oct(8), Hex(-1), CBool("true"), Not True, Not 0, 5 And 3, True Or False
End Sub
"##);
    assert_eq!(
        out[0],
        "Empty         Integer       Long          Double        String"
    );
    assert_eq!(
        out[1],
        "True          True          True          False         True"
    );
    assert_eq!(out[2], "True          Collection    True          8");
    assert_eq!(out[3], "b             y             2");
    assert_eq!(out[4], "FF            10            FFFFFFFF      True          False         -1            1             True");
}

#[test]
fn constants_enums_and_module_variables() {
    let out = run(r##"
Private Const Greeting As String = "hi"
Public Enum Sizes
    Small = 1
    Medium
    Large = 10
End Enum
Dim counter As Long

Sub Count()
    counter = counter + 1
End Sub

Sub Main()
    Const Local1 = 3
    Count
    Count
    Debug.Print Greeting, Medium, Sizes.Large, counter, Local1
End Sub
"##);
    assert_eq!(
        out,
        vec!["hi            2             10            2             3"]
    );
}

#[test]
fn static_variables_keep_their_value() {
    let out = run(r##"
Function NextId() As Long
    Static n As Long
    n = n + 1
    NextId = n
End Function
Sub Main()
    NextId
    NextId
    Debug.Print NextId()
End Sub
"##);
    assert_eq!(out, vec!["3"]);
}

#[test]
fn conditional_compilation_picks_the_64_bit_declares() {
    let out = run(r##"
#If VBA7 Then
    Private Declare PtrSafe Sub Sleep Lib "kernel32" (ByVal ms As LongPtr)
#Else
    Private Declare Sub Sleep Lib "kernel32" (ByVal ms As Long)
#End If
Sub Main()
    On Error Resume Next
    Sleep 10
    Debug.Print Err.Number, Err.Description
End Sub
"##);
    assert_eq!(out[0].split_whitespace().next(), Some("445"));
    assert!(out[0].contains("Windows API function Sleep"), "{}", out[0]);
}

#[test]
fn msgbox_and_inputbox_use_the_host_hooks() {
    let src = r##"
Sub Main()
    Dim r
    r = MsgBox("Continue?", vbYesNo + vbQuestion, "Ask")
    Debug.Print r
    Debug.Print InputBox("Name?", "Ask", "Bob")
    MsgBox Prompt:="done"
End Sub
"##;
    let hooks = r##"
function __host_msgbox(t, b, title) { __lines.push("msg:" + t + "|" + b + "|" + title); return 6; }
function __host_inputbox(p, t, d) { return d + "!"; }
"##;
    let r = run_modules(
        &[Module::new("Module1", ModuleKind::Standard, src)],
        "Module1",
        "Main",
        hooks,
    );
    r.result.unwrap();
    assert_eq!(
        r.lines,
        vec!["msg:Continue?|36|Ask", "6", "Bob!", "msg:done|0|"]
    );
}

#[test]
fn modules_call_each_other_and_classes_work() {
    let m1 = r##"
Public Total As Long
Public Function AddUp(a, b)
    AddUp = a + b
End Function
"##;
    let cls = r##"
VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "Counter"
Private mCount As Long
Public Name As String

Private Sub Class_Initialize()
    mCount = 100
End Sub

Public Sub Increment(Optional by As Long = 1)
    mCount = mCount + by
End Sub

Public Property Get Value() As Long
    Value = mCount
End Property

Public Property Let Value(v As Long)
    mCount = v
End Property
"##;
    let main = r##"
Sub Main()
    Dim c As Counter
    Set c = New Counter
    c.Increment
    c.Increment by:=5
    Debug.Print c.Value
    c.Value = 7
    c.name = "first"
    Debug.Print c.Value, c.Name, TypeName(c)
    Total = Helpers.AddUp(2, 3)
    Debug.Print Helpers.Total, AddUp("a", "b")
End Sub
"##;
    let r = run_modules(
        &[
            Module::new("Helpers", ModuleKind::Standard, m1),
            Module::new("Counter", ModuleKind::Class, cls),
            Module::new("Main", ModuleKind::Standard, main),
        ],
        "Main",
        "Main",
        "",
    );
    r.result.unwrap();
    assert_eq!(
        r.lines,
        vec![
            "106",
            "7             first         Counter",
            "5             ab"
        ]
    );
}

#[test]
fn compile_errors_name_the_module_and_line() {
    let e = compile(&[Module::new(
        "M",
        ModuleKind::Standard,
        "Sub A()\n  If x Then\n  x = 1\nEnd Sub\n",
    )])
    .unwrap_err();
    assert_eq!(e.module, "M");
    assert!(e.message.contains("End If"), "{e}");
    let e = compile(&[Module::new(
        "M",
        ModuleKind::Standard,
        "Sub A()\n  Dim i\n  Next i\nEnd Sub\n",
    )])
    .unwrap_err();
    assert_eq!(e.line, 3, "{e}");
}

#[test]
fn the_macro_list_has_public_parameterless_subs() {
    let mods = [
        Module::new(
            "Tools",
            ModuleKind::Standard,
            "Sub Run1()\nEnd Sub\nPrivate Sub Hidden()\nEnd Sub\nSub WithArg(x)\nEnd Sub\nFunction F()\nEnd Function\n",
        ),
        Module::new("Priv", ModuleKind::Standard, "Option Private Module\nSub A()\nEnd Sub\n"),
        Module::new("C", ModuleKind::Class, "Sub M()\nEnd Sub\n"),
    ];
    assert_eq!(
        list_macros(&mods),
        vec![("Tools".to_string(), "Run1".to_string())]
    );
}

#[test]
fn a_name_and_a_colon_starting_a_line_is_a_label() {
    // As in VBA: "Count: Count" is the label Count, then one call.
    let out = run("Dim n\nSub Count()\n n = n + 1\nEnd Sub\nSub Main()\n Count: Count\n Debug.Print n\nEnd Sub\n");
    assert_eq!(out, vec!["1"]);
}

#[test]
fn line_continuations_and_colons() {
    let out = run("Sub Main()\n    Dim s: s = \"a\" & _\n        \"b\" & _\n        \"c\": Debug.Print s\nEnd Sub\n");
    assert_eq!(out, vec!["abc"]);
}

#[test]
fn line_map_points_back_to_vba_lines() {
    let p = compile(&[Module::new(
        "M",
        ModuleKind::Standard,
        "Sub A()\n\n  Debug.Print 1\nEnd Sub\n",
    )])
    .unwrap();
    let js_line = p.js.lines().position(|l| l.contains("__f.l=3;")).unwrap() as u32 + 1;
    assert_eq!(p.vba_line(js_line), Some(("M", 3)));
}

#[test]
fn rnd_repeats_with_a_negative_seed() {
    let out = run(r##"
Sub Main()
    Dim a, b
    Rnd -1: Randomize 5: a = Rnd
    Rnd -1: Randomize 5: b = Rnd
    Debug.Print a = b, a >= 0 And a < 1
End Sub
"##);
    assert_eq!(out, vec!["True          True"]);
}

#[test]
fn more_operators_and_patterns() {
    let out = run(r##"
Sub Main()
    Debug.Print True Xor False, True Eqv False, False Imp True, 6 Xor 3, &H10 + &O10
    Debug.Print "Cat" Like "[A-C]at", "Bat" Like "[!B]at", "a*b" Like "a[*]b", "x9" Like "?#"
    Dim o As Object
    Debug.Print o Is Nothing, Not o Is Nothing
    Debug.Print #1/31/2024# + 1, #1/31/2024# - #1/1/2024#
End Sub
"##);
    assert_eq!(
        out[0],
        "True          False         True          5             24"
    );
    assert_eq!(out[1], "True          False         True          True");
    assert_eq!(out[2], "True          False");
    assert_eq!(out[3], "2/1/2024      30");
}

#[test]
fn nested_with_and_exit_function() {
    let out = run(r##"
Type Inner
    V As Long
End Type
Type Outer
    I As Inner
    Name As String
End Type

Function FirstOver(limit As Long) As Long
    Dim i As Long
    For i = 1 To 100
        If i * i > limit Then
            FirstOver = i
            Exit Function
        End If
    Next
    FirstOver = -1
End Function

Sub Main()
    Dim o As Outer
    With o
        .Name = "x"
        With .I
            .V = 7
        End With
        Debug.Print .Name, .I.V
    End With
    Debug.Print FirstOver(50)
End Sub
"##);
    assert_eq!(out, vec!["x             7", "8"]);
}

#[test]
fn resume_retries_and_line_number_labels() {
    let out = run(r##"
Sub Main()
    Dim tries As Long
    On Error GoTo Retry
10  tries = tries + 1
20  If tries < 3 Then Err.Raise 5
30  Debug.Print "done after " & tries
    Exit Sub
Retry:
    If tries < 5 Then Resume 10
End Sub
"##);
    assert_eq!(out, vec!["done after 3"]);
    let out = run(r##"
Sub Main()
    Dim n, d
    On Error GoTo H
    n = 10 / d
    Debug.Print "back " & n
    Exit Sub
H:
    d = 4
    Resume
End Sub
"##);
    // Resume runs the failed statement again, now with a valid divisor.
    assert_eq!(out, vec!["back 2.5"]);
}

#[test]
fn property_set_static_sub_and_redim_preserve() {
    let cls = r##"
Private mTarget As Object
Public Property Set Target(o As Object)
    Set mTarget = o
End Property
Public Property Get Target() As Object
    Set Target = mTarget
End Property
"##;
    let main = r##"
Static Sub Tick()
    Dim n As Long
    n = n + 1
    Debug.Print "tick " & n
End Sub

Sub Main()
    Dim h As New Holder, c As New Collection, a()
    Set h.Target = c
    Debug.Print h.Target Is c
    Tick
    Tick
    ReDim a(1 To 2, 1 To 2)
    a(1, 2) = "keep"
    ReDim Preserve a(1 To 2, 1 To 4)
    Debug.Print a(1, 2), UBound(a, 2)
    Erase a
    On Error Resume Next
    Debug.Print UBound(a)
    Debug.Print Err.Number
End Sub
"##;
    let r = run_modules(
        &[
            Module::new("Holder", ModuleKind::Class, cls),
            Module::new("M", ModuleKind::Standard, main),
        ],
        "M",
        "Main",
        "",
    );
    r.result.unwrap();
    assert_eq!(
        r.lines,
        vec!["True", "tick 1", "tick 2", "keep          4", "9"]
    );
}

#[test]
fn library_odds_and_ends() {
    let out = run(r##"
Sub Main()
    Dim t
    t = Timer
    DoEvents
    Debug.Print Environ("PATH") = "", t >= 0, IsDate("2024-01-02"), Weekday(#1/7/2024#), MonthName(2, True)
    Debug.Print FormatNumber(1234.567, 1), FormatPercent(0.25, 0), StrConv("hello world", vbProperCase)
    Debug.Print Join(Filter(Array("apple", "berry", "grape"), "ap"), ","), Replace("aXbXc", "x", "-", , , vbTextCompare)
    Debug.Print DateDiff("m", #1/15/2024#, #4/1/2024#), DatePart("q", #8/1/2024#), Format(DateAdd("d", -1, #3/1/2024#), "yyyy-mm-dd")
End Sub
"##);
    assert_eq!(
        out[0],
        "True          True          True          1             Feb"
    );
    assert_eq!(out[1], "1,234.6       25%           Hello World");
    assert_eq!(out[2], "apple,grape   a-b-c");
    assert_eq!(out[3], "3             3             2024-02-29");
}

#[test]
fn goto_reaches_a_label_inside_a_loop_and_ifs_around_it() {
    // The common "skip to Next" idiom: a label just before Next.
    let out = run(r##"
Sub Main()
    Dim i As Integer, c As Integer
    For i = 1 To 6
        If i Mod 2 = 0 Then c = c + 10: GoTo myNext
        Select Case i
            Case 3
                GoTo myNext
        End Select
        c = c + 1
myNext:
    Next i
    Debug.Print c
    Dim n As Integer
    Do While n < 3
        n = n + 1
        If n = 2 Then GoTo Again
        Debug.Print "n"; n
Again:
    Loop
    GoTo Done
    Debug.Print "skipped"
Done:
    Debug.Print "done"
End Sub
"##);
    assert_eq!(out, vec!["32", "n1", "n3", "done"]);
}

#[test]
fn a_do_loop_on_one_line_with_colons() {
    let out = run(r##"
Sub Main()
    Dim n As Integer
    Do: n = n + 1: DoEvents: Loop While n < 5
    Debug.Print n
    Do: n = n - 1: Loop Until n = 2
    Debug.Print n
End Sub
"##);
    assert_eq!(out, vec!["5", "2"]);
}

#[test]
fn file_numbers_in_arguments_parse_and_fail_at_run_time() {
    let e = run_err(
        r##"
Sub Main()
    Dim s As String, hF As Integer
    hF = 1
    s = Input(LOF(hF), #hF)
End Sub
"##,
    );
    assert!(e.contains("445") || e.contains("Not supported"), "{e}");
}

#[test]
fn undeclared_names_under_option_explicit_are_left_to_the_host() {
    // Names the host or a form provides are not known when compiling.
    let m = Module::new(
        "m",
        ModuleKind::Standard,
        "Option Explicit\r\nSub Main()\r\nOptimization = True\r\nEnd Sub\r\n",
    );
    assert!(compile(&[m]).is_ok());
}

#[test]
fn goto_from_an_if_branch_into_its_else_branch() {
    let out = run(r##"
Sub Main()
    Dim i As Integer
    For i = 1 To 3
        If i = 2 Then
            Debug.Print "two"
            GoTo AfterIt
        Else
            Debug.Print "other"
AfterIt:
            Debug.Print "after"; i
        End If
    Next
End Sub
"##);
    assert_eq!(
        out,
        vec!["other", "after1", "two", "after2", "other", "after3"]
    );
}

#[test]
fn a_module_variable_as_new_class_of_a_later_module() {
    let mods = [
        Module::new(
            "Main",
            ModuleKind::Standard,
            "Public cfg As New Config\r\nSub Go()\r\n  cfg.Size = 4\r\n  Debug.Print cfg.Size * 2\r\nEnd Sub\r\n",
        ),
        Module::new("Config", ModuleKind::Class, "Public Size As Integer\r\n"),
    ];
    let r = run_modules(&mods, "Main", "Go", "");
    assert_eq!(r.lines, vec!["8"], "{:?}", r.result);
}

#[test]
fn settings_are_kept_for_the_run_without_a_host() {
    let out = run(r##"
Sub Main()
    Debug.Print "[" & GetSetting("App", "Opt", "Size", "10") & "]"
    SaveSetting "App", "Opt", "Size", 12
    SaveSetting "App", "Opt", "Name", "x"
    Debug.Print GetSetting("app", "opt", "SIZE")
    Dim a
    a = GetAllSettings("App", "Opt")
    Debug.Print UBound(a, 1), a(1, 0), a(1, 1)
    DeleteSetting "App", "Opt", "Size"
    Debug.Print "[" & GetSetting("App", "Opt", "Size") & "]"
End Sub
"##);
    assert_eq!(
        out,
        vec!["[10]", "12", "1             Name          x", "[]"]
    );
}
