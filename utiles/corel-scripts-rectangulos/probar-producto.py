"""Comprueba modelos de scripts, unidades y cuatro esquinas sin abrir ventanas."""
from pathlib import Path
import argparse
import ctypes
import hashlib
import importlib.util
import json
import math
import shutil
import subprocess
import time
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]


def load_rpc():
    path = ROOT / "utiles/corel-portapapeles/comprobar-procesos.py"
    spec = importlib.util.spec_from_file_location("owned_rectangle_rpc", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module, path


def first_shape(document):
    objects = [s for p in document["pages"] for layer in p["layers"] for s in layer["shapes"]]
    assert len(objects) == 1, "Debe conservarse exactamente una figura."
    return objects[0]


def assert_bounds(shape, wanted):
    assert shape["kind"]["type"] == "rect"
    rect = shape["kind"]["rect"]
    assert all(abs(rect[key] - number) < 1e-9 for key, number in zip(["x0", "y0", "x1", "y1"], wanted))
    assert shape["transform"] == [1, 0, 0, 1, 0, 0]


def verify_render(actual_path, expected_path):
    # Public positive case: 40 by 20 mm, corner percentages 20/40/60/80,
    # hence physical radii UL/UR/LR/LL = 2/4/6/8 mm. No transformed corners.
    with Image.open(actual_path) as original:
        image = Image.alpha_composite(Image.new("RGBA", original.size, "white"), original.convert("RGBA")).convert("RGB")
    image.save(actual_path.with_name(actual_path.stem + "-on-white.png"))
    assert image.size == (1000, 700), "La página de 100 por 70 mm a 254 dpi debe medir 1000 por 700 píxeles."
    expected = Image.new("RGB", image.size, "white")
    observed, wanted = image.load(), expected.load()
    fill, tolerance, checked, ignored = (20, 150, 80), 0.15, 0, 0
    for py in range(image.height):
        y = 70 - (py + 0.5) / 10
        for px in range(image.width):
            x = (px + 0.5) / 10
            if not (10 <= x <= 50 and 10 <= y <= 30):
                distance = math.hypot(max(10 - x, 0, x - 50), max(10 - y, 0, y - 30))
                inside = False
            else:
                distance = min(x - 10, 50 - x, y - 10, 30 - y)
                inside = True
                for cx, cy, radius, in_corner in [
                    (12, 28, 2, x < 12 and y > 28),
                    (46, 26, 4, x > 46 and y > 26),
                    (44, 16, 6, x > 44 and y < 16),
                    (18, 18, 8, x < 18 and y < 18),
                ]:
                    if in_corner:
                        signed = radius - math.hypot(x - cx, y - cy)
                        inside, distance = signed > 0, abs(signed)
                        break
            colour = fill if inside else (255, 255, 255)
            wanted[px, py] = colour
            if distance <= tolerance:
                ignored += 1
                continue
            assert max(abs(a - b) for a, b in zip(observed[px, py], colour)) <= 1, (px, py, observed[px, py], colour)
            checked += 1
    assert checked > 690000 and ignored < 10000
    expected.save(expected_path)
    return {"passed": True, "pixels_checked": checked, "boundary_pixels_excluded": ignored,
        "boundary_margin_pixels": 1.5, "channel_tolerance": 1,
        "alpha_preservation_verified": False,
        "scope": "Independent circular-corner mask against the actual CPU PNG export; no GUI/GPU/original-editor comparison."}


def wrong_render_controls(actual_path):
    with Image.open(actual_path) as original:
        base = Image.alpha_composite(Image.new("RGBA", original.size, "white"), original.convert("RGBA")).convert("RGB")
    swapped = base.copy()
    swapped.paste(base.crop((100, 400, 500, 600)).transpose(Image.Transpose.FLIP_LEFT_RIGHT), (100, 400))
    square = Image.new("RGB", base.size, "white")
    ImageDraw.Draw(square).rectangle((100, 400, 499, 599), fill=(20, 150, 80))
    result = []
    for name, image in [("swapped-corners", swapped), ("square", square)]:
        path = actual_path.with_name(name + ".png")
        image.save(path)
        try:
            verify_render(path, path.with_name(name + "-expected.png"))
        except AssertionError as error:
            assert error.args and isinstance(error.args[0], tuple) and len(error.args[0]) == 4
            result.append({"name": name, "rejected": True, "first_pixel_mismatch": error.args[0]})
        else:
            raise AssertionError("Una imagen con esquinas incorrectas pasó la comparación.")
    return result


def main():
    if not __debug__:
        raise RuntimeError("Las comprobaciones requieren Python sin optimización de aserciones.")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=Path)
    args = parser.parse_args()
    helper, helper_path = load_rpc()
    manifest_path = helper.workspace(args.manifest)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    runtime = helper.workspace(Path(manifest["directory"]))
    executable = runtime / "craft-corel.exe"
    assert helper.sha(executable) == helper.KNOWN_LAUNCHER == manifest["launcher_sha256"]
    assert executable.stat().st_size < 2_000_000
    for item in manifest["files"]:
        if Path(item["path"]).suffix.lower() in {".exe", ".dll"}:
            assert helper.sha(runtime / item["path"]) == item["sha256"]
    ctypes.WinDLL("kernel32").SetErrorMode(0x0001 | 0x0002 | 0x8000)
    out = ROOT / "evidencias/craft-corel-nativo/scripts-rectangulos/producto" / str(time.time_ns())
    out.mkdir(parents=True)
    report = {"passed": False, "manifest": str(manifest_path), "manifest_sha256": helper.sha(manifest_path),
        "script_sha256": helper.sha(Path(__file__)), "helper_sha256": helper.sha(helper_path)}
    source = reader = None
    try:
        positive = subprocess.run([str(executable), "--verify-runtime"], cwd=runtime,
            env=helper.profile_env(out / "positive-config"), capture_output=True, text=True,
            creationflags=helper.FLAGS, timeout=20)
        assert positive.returncode == 0 and "ABI 1" in positive.stdout
        negative = out / "without-engine"
        negative.mkdir()
        shutil.copy2(executable, negative / executable.name)
        rejected = subprocess.run([str(negative / executable.name), "--verify-runtime"], cwd=negative,
            env=helper.profile_env(out / "negative-config"), capture_output=True, text=True,
            creationflags=helper.FLAGS, timeout=20)
        assert rejected.returncode == 1 and "ABI 1" not in rejected.stdout
        source = helper.Rpc(executable, out / "source", 0)
        source.initialize()
        tools = source.request("tools/list", {})["tools"]
        tool = next(tool for tool in tools if tool["name"] == "run_script")
        assert tool["inputSchema"]["properties"]["object_model"]["enum"] == ["native", "vgcore"]

        def script(text, model="vgcore", error=False):
            arguments = {"source": text}
            if model is not None:
                arguments["object_model"] = model
            return source.call("run_script", arguments, error=error)

        source.call("new_document", {"width_mm": 100, "height_mm": 70})
        assert script("const r=ActiveLayer.CreateRectangle(10,30,50,10); print(r.SizeWidth,r.SizeHeight);", None) == "50 10"
        native = source.saved("native.tdraw")
        assert native["metadata"].get("script_unit") is None
        assert_bounds(first_shape(native), [10, 30, 60, 40])
        source.call("new_document", {"width_mm": 100, "height_mm": 70})
        script("ActiveDocument.Unit=cdrMillimeter;")
        assert script("const r=ActiveLayer.CreateRectangle(10,30,50,10,20,40,60,80); r.Fill.UniformColor.RGBAssign(20,150,80); r.Outline.SetNoOutline(); print(r.SizeWidth,r.SizeHeight,r.Rectangle.MaxRadius);") == "40 20 10"
        rounded = source.saved("rounded.tdraw")
        assert rounded["metadata"]["script_unit"] == 3
        shape = first_shape(rounded)
        assert_bounds(shape, [10, 10, 50, 30])
        assert shape["kind"]["corners"]["radii"] == [2, 4, 8, 6]
        assert shape["visible"] is True and shape["opacity"] == 1
        assert shape.get("stroke") is None and shape.get("shadow") is None and not shape["effects"]
        assert shape["fill"]["fill"] == "solid" and shape["fill"]["model"] == "rgb"
        assert all(abs(shape["fill"][key] - value / 255) < 1e-7
            for key, value in zip(["r", "g", "b"], [20, 150, 80]))
        rendered = source.folder / "rounded.png"
        source.call("export", {"path": str(rendered), "dpi": 254})
        report["render"] = verify_render(rendered, source.folder / "expected-rounded.png")
        report["wrong_render_controls"] = wrong_render_controls(rendered)

        before_selection = json.loads(source.call("document_info"))["selection"]
        source.call("undo")
        before = source.saved("before-errors.tdraw")
        assert not helper.roots(before) and before["metadata"]["script_unit"] == 3
        empty_selection = json.loads(source.call("document_info"))["selection"]
        assert empty_selection == []
        assert before["ids"]["next"] >= rounded["ids"]["next"]
        for index, model in enumerate([None, True, 42, {}, [], "", "other"]):
            source.call("run_script", {"object_model": model, "source": "ActiveLayer.CreateRectangle(1,2,3,4);"}, error=True)
            assert source.saved(f"after-model-error-{index}.tdraw") == before
            assert json.loads(source.call("document_info"))["selection"] == empty_selection
        bad_sources = ["ActiveDocument.Unit=17;", "ActiveLayer.CreateRectangle(1,2,3,4,Infinity);",
            "ActiveLayer.CreateRectangle2(1,2,3,4,Infinity);",
            "ActiveLayer.CreateRectangleRect({x:1,y:2,Width:3,Height:4},Infinity);"]
        for index, text in enumerate(bad_sources):
            script(text, error=True)
            assert source.saved(f"after-error-{index}.tdraw") == before
            assert json.loads(source.call("document_info"))["selection"] == empty_selection
        script("ActiveDocument.Unit=cdrMillimeter;")
        assert source.saved("same-unit.tdraw") == before
        source.call("redo")
        redone = source.saved("redone.tdraw")
        assert helper.without_allocator(redone) == helper.without_allocator(rounded)
        assert redone["ids"]["next"] >= before["ids"]["next"]
        assert json.loads(source.call("document_info"))["selection"] == before_selection

        reader = helper.Rpc(executable, out / "reader", 0)
        reader.initialize()
        assert reader.process.pid != source.process.pid
        reader.call("open", {"path": str(source.folder / "rounded.tdraw")})
        assert helper.without_allocator(reader.saved("reopened.tdraw")) == helper.without_allocator(rounded)
        assert reader.call("run_script", {"object_model": "vgcore", "source": "print(ActiveDocument.Unit,ActivePage.Shapes.Item(1).SizeWidth);"}) == "3 40"
        cdr = source.folder / "rounded-v27.cdr"
        source.call("save", {"path": str(cdr), "version": 27})
        reader.call("open", {"path": str(cdr)})
        cdr_document = reader.saved("cdr-reopened.tdraw")
        report["cdr_macro_unit_observed"] = cdr_document["metadata"].get("script_unit")
        report["cdr_external_macro_unit_compatibility_verified"] = False
        cdr_png = reader.folder / "cdr-rounded.png"
        reader.call("export", {"path": str(cdr_png), "dpi": 254})
        report["cdr27_render_roundtrip"] = verify_render(cdr_png, reader.folder / "expected-cdr-rounded.png")
        # Return to the saved native drawing before checking its persisted unit.
        reader.call("open", {"path": str(source.folder / "rounded.tdraw")})
        script("ActiveDocument.Unit=cdrTenthMicron;")
        assert script("print(ActiveDocument.Unit,ActivePage.Shapes.Item(1).SizeWidth);") == "0 400000"
        assert script("print(ActivePage.Shapes.All.Item(1).SizeWidth);", "native") == "40"
        assert reader.call("run_script", {"object_model": "vgcore", "source": "print(ActiveDocument.Unit);"}) == "3"
        assert source.saved("unit-zero.tdraw")["metadata"]["script_unit"] == 0

        positive_constructors = []
        for name, expression in [
            ("sides", "ActiveLayer.CreateRectangle(10,30,50,10,20,40,60,80)"),
            ("dimensions", "ActiveLayer.CreateRectangle2(10,10,40,20,2,4,6,8)"),
            ("rectangle-object", "ActiveLayer.CreateRectangleRect({x:10,y:10,Width:40,Height:20},2,4,6,8)"),
        ]:
            source.call("new_document", {"width_mm": 100, "height_mm": 70})
            initial = source.saved(name + "-empty.tdraw")
            assert initial["metadata"].get("script_unit") is None
            assert script(f"ActiveDocument.Unit=cdrMillimeter; const r={expression}; print(r.SizeWidth,r.SizeHeight,r.Rectangle.MaxRadius);") == "40 20 10"
            created = source.saved(name + "-positive.tdraw")
            shape = first_shape(created)
            assert_bounds(shape, [10, 10, 50, 30])
            assert shape["kind"]["corners"]["radii"] == [2, 4, 8, 6]
            source.call("undo")
            undone = source.saved(name + "-undone.tdraw")
            assert helper.without_allocator(undone) == helper.without_allocator(initial)
            assert undone["ids"]["next"] >= created["ids"]["next"]
            source.call("redo")
            positive_redone = source.saved(name + "-redone.tdraw")
            assert helper.without_allocator(positive_redone) == helper.without_allocator(created)
            assert positive_redone["ids"]["next"] >= undone["ids"]["next"]
            positive_constructors.append({"name": name, "passed": True, "all_four_radii": True, "one_undo_redo": True})
        source.call("new_document", {"width_mm": 100, "height_mm": 70})
        empty = source.saved("before-settings.tdraw")
        assert empty["metadata"].get("script_unit") is None
        script("ActiveDocument.SaveSettings(); ActiveDocument.Unit=cdrMillimeter; ActiveDocument.RestoreSettings();")
        assert source.saved("after-settings.tdraw") == empty
        report["positive_constructors"] = positive_constructors
        report["dimensional_anchor_scope"] = "Existing owned lower-left convention; official text remains contradictory."
        report["selection_scope"] = "Actual product cases start with empty selection; nonempty selection is covered separately by application tests."
        report.update(passed=True, positive_and_negative_abi=True, distinct_model_geometry=True,
            macro_unit_persists_between_calls_and_processes=True, unit_zero_and_native_mm_preserved=True,
            invalid_constructors_and_unit_preserve_redo=True, same_unit_noop=True,
            omitted_selector_keeps_native_and_invalid_selectors_preserve_redo=True,
            save_restore_preserves_absent_metadata=True, system_clipboard_used=False,
            visible_gui_tested=False, original_coreldraw_executed=False, full_equivalence_verified=False)
    except BaseException as error:
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        cleanup_errors = []
        for rpc in [source, reader]:
            if rpc is not None:
                try:
                    rpc.close()
                except Exception as error:
                    cleanup_errors.append(str(error))
        if cleanup_errors:
            report.update(passed=False, cleanup_errors=cleanup_errors)
        (out / "result.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(json.dumps({"passed": report["passed"], "evidence": str(out)}))
        if cleanup_errors and "error" not in report:
            raise RuntimeError("El proceso de prueba no terminó correctamente.")


if __name__ == "__main__":
    main()
