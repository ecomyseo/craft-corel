//! VBA against a host object model (the fake one in `common`): member
//! access, default members, collections, forms.

mod common;
use common::{run, run_err, run_modules};
use tracedraw_vba::{Module, ModuleKind};

#[test]
fn walks_the_selection_and_changes_shapes() {
    let out = run(r##"
Sub Main()
    Dim s As Shape, i As Long, sr
    Set sr = ActiveSelection
    Debug.Print sr.Count, sr.Shapes.Count
    For Each s In ActiveSelection.Shapes
        s.Move 1, 2
        Debug.Print s.Name & " at " & s.PositionX & "," & s.PositionY
    Next s
    For i = 1 To ActiveDocument.SelectionRange.Shapes.Count
        With ActiveDocument.SelectionRange.Shapes(i)
            .SetSize .SizeWidth * 2, .SizeHeight
            Debug.Print .Name, .SizeWidth
        End With
    Next
End Sub
"##);
    assert_eq!(
        out,
        vec![
            "2             2",
            "A at 1,2",
            "B at 21,2",
            "A             20",
            "B             10",
        ]
    );
}

#[test]
fn host_members_ignore_case_and_methods_need_no_parentheses() {
    let out = run(r##"
Sub Main()
    Dim c, s
    Set c = activedocument.createcolor
    c.rgbassign 255, 128, 0
    Debug.Print c.HEXVALUE
    Set s = ActiveLayer.Shapes.Item("C")
    Debug.Print s.name, ActiveLayer.Shapes("B").sizewidth
    Set s = ActiveLayer.CreateRectangle(0, 0, 4, 3)
    Debug.Print s.SizeWidth * s.SizeHeight, ActiveLayer.Shapes.Count
    s.Fill.ApplyUniformFill CreateRGBColor(1, 2, 3)
    Debug.Print s.Fill.Type, s.Fill.UniformColor.HexValue
    ActiveDocument.Unit = cdrMillimeter
    Debug.Print ActiveDocument.Unit
    s.Delete
    Debug.Print s.deleted
End Sub
"##);
    assert_eq!(
        out,
        vec![
            "#ff8000",
            "C             5",
            "12            4",
            "1             #010203",
            "3",
            "True",
        ]
    );
}

#[test]
fn host_methods_take_named_arguments() {
    let js = r##"
var Tool = { Place(x, y, label) { return label + "@" + x + "," + y; }, Bare(a) { return a; } };
Tool.Place.__params = ["x", "y", "label"];
"##;
    let src = r##"
Sub Main()
    Debug.Print Tool.Place(label:="L", y:=2, x:=1)
    Debug.Print Tool.Place(5, , "M")
    On Error Resume Next
    Debug.Print Tool.Bare(a:=1)
    Debug.Print Err.Number
End Sub
"##;
    let r = run_modules(
        &[Module::new("M", ModuleKind::Standard, src)],
        "M",
        "Main",
        js,
    );
    r.result.unwrap();
    assert_eq!(r.lines, vec!["L@1,2", "M@5,undefined", "448"]);
}

#[test]
fn a_missing_member_is_error_438_and_nothing_is_91() {
    let e = run_err("Sub Main()\n ActiveDocument.NoSuchThing\nEnd Sub");
    assert!(e.contains("'438'"), "{e}");
    assert!(e.contains("NoSuchThing"), "{e}");
    let e = run_err("Sub Main()\n Dim s As Shape\n Debug.Print s.Name\nEnd Sub");
    assert!(e.contains("'91'"), "{e}");
    let e = run_err("Sub Main()\n UndefinedProc 1\nEnd Sub");
    assert!(e.contains("'35'"), "{e}");
}

#[test]
fn typeof_and_bang_access() {
    let out = run(r##"
Sub Main()
    Dim d
    Set d = CreateObject("Scripting.Dictionary")
    d("k") = 5
    Debug.Print d!k, TypeOf ActiveLayer Is Layer, TypeOf ActiveLayer Is Shape, TypeName(ActiveLayer)
End Sub
"##);
    assert_eq!(out, vec!["5             True          False         Layer"]);
}

#[test]
fn forms_get_controls_from_the_host_and_run_event_handlers() {
    let form = r##"
VERSION 5.00
Begin {C62A69F0-16DC-11CE-9E98-00AA00574A4F} UserForm1
   Caption         =   "Resize"
   ClientHeight    =   3000
   Begin {978C9E23-D4B0-11CE-BF2D-00AA003F40D0} Label1
   End
End
Attribute VB_Name = "UserForm1"
Private Sub UserForm_Initialize()
    TextBox1.Text = "10"
    Me.Caption = "Resize shapes"
End Sub

Private Sub CommandButton1_Click()
    Dim f As Double
    f = CDbl(TextBox1.Text)
    Debug.Print "factor " & f * 2 & " in " & Me.Caption
    Me.Hide
End Sub
"##;
    let main = r##"
Sub Main()
    UserForm1.Show
    Debug.Print "shown"
    Unload UserForm1
End Sub
"##;
    let js = r##"
function __host_form_create(form) {
  form.__controls["TextBox1"] = { Text: "" };
}
function __host_form_show(form, modal) {
  __lines.push("show " + modal + " " + form.__props.caption);
  form.__event("CommandButton1", "Click", []);
}
function __host_form_hide(form) { __lines.push("hide"); }
function __host_form_unload(form) { __lines.push("unload"); }
"##;
    let r = run_modules(
        &[
            Module::new("UserForm1", ModuleKind::Form, form),
            Module::new("Main", ModuleKind::Standard, main),
        ],
        "Main",
        "Main",
        js,
    );
    r.result.unwrap();
    assert_eq!(
        r.lines,
        vec![
            "show true Resize shapes",
            "factor 20 in Resize shapes",
            "hide",
            "shown",
            "unload",
        ]
    );
}

#[test]
fn a_typical_community_macro_runs() {
    // Renames, filters and counts selected shapes, with error handling
    // and a helper function.
    let out = run(r##"
Option Explicit

Private Function Pad(ByVal n As Long, ByVal width As Integer) As String
    Pad = Right$(String$(width, "0") & CStr(n), width)
End Function

Public Sub Main()
    Dim sr As ShapeRange, s As Shape, n As Long, wide As New Collection
    On Error GoTo ErrHandler
    Set sr = ActiveDocument.SelectionRange
    If sr.Count = 0 Then
        MsgBox "Select something first", vbExclamation
        Exit Sub
    End If
    For Each s In sr.Shapes
        n = n + 1
        s.Name = "Item_" & Pad(n, 3)
        If s.SizeWidth > 6# Then wide.Add s, s.Name
    Next s
    Debug.Print n & " renamed, " & wide.Count & " wide: " & wide(1).Name
    Exit Sub
ErrHandler:
    MsgBox "Error " & Err.Number & ": " & Err.Description, vbCritical
End Sub
"##);
    assert_eq!(out, vec!["2 renamed, 1 wide: Item_001"]);
}

#[test]
fn unsupported_statements_fail_at_run_time_with_a_clear_message() {
    let e = run_err("Sub Main()\n Open \"c:\\x.txt\" For Output As #1\nEnd Sub");
    assert!(
        e.contains("Not supported in TraceDraw: Open statement"),
        "{e}"
    );
    let e = run_err(
        "Sub Main()\n Dim fso\n Set fso = CreateObject(\"Scripting.FileSystemObject\")\nEnd Sub",
    );
    assert!(e.contains("'429'"), "{e}");
}

#[test]
fn class_default_member_and_property_with_arguments() {
    let cls = r##"
Private items(1 To 3) As String
Public Property Get Item(ByVal i As Long) As String
Attribute Item.VB_UserMemId = 0
    Item = items(i)
End Property
Public Property Let Item(ByVal i As Long, ByVal v As String)
    items(i) = v
End Property
Public Function Self() As Bag
    Set Self = Me
End Function
"##;
    let main = r##"
Sub Main()
    Dim b As New Bag
    b.Item(2) = "two"
    Debug.Print b.Item(2), b(2), b.Self.Item(2)
End Sub
"##;
    let r = run_modules(
        &[
            Module::new("Bag", ModuleKind::Class, cls),
            Module::new("M", ModuleKind::Standard, main),
        ],
        "M",
        "Main",
        "",
    );
    r.result.unwrap();
    assert_eq!(r.lines, vec!["two           two           two"]);
}
