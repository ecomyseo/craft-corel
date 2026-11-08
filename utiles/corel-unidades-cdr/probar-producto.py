"""Prueba unidades independientes y CDR propio en procesos sin ventanas."""
from pathlib import Path
import argparse
import copy
import ctypes
import importlib.util
import json
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def settings(doc):
    meta = doc["metadata"]
    rulers = meta["rulers"]
    return [meta.get("script_unit"), rulers.get("horizontal_unit"), rulers.get("vertical_unit")]


def main():
    if not __debug__:
        raise RuntimeError("Las pruebas requieren aserciones activas.")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=Path)
    args = parser.parse_args()
    helper_path = ROOT / "utiles/corel-portapapeles/comprobar-procesos.py"
    render_path = ROOT / "utiles/corel-scripts-rectangulos/probar-producto.py"
    helper = load(helper_path, "owned_units_rpc")
    render = load(render_path, "owned_units_render")
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
    out = ROOT / "evidencias/craft-corel-nativo/unidades-cdr/producto" / str(time.time_ns())
    out.mkdir(parents=True)
    report = {"passed": False, "manifest": str(manifest_path), "manifest_sha256": helper.sha(manifest_path),
              "script_sha256": helper.sha(Path(__file__)), "helper_sha256": helper.sha(helper_path),
              "render_oracle_sha256": helper.sha(render_path), "roundtrips": [],
              "system_clipboard_used": False, "visible_gui_tested": False,
              "original_coreldraw_executed": False, "external_cdr_unit_interoperability_verified": False,
              "full_equivalence_verified": False}
    clients = []
    try:
        positive = subprocess.run([str(executable), "--verify-runtime"], cwd=runtime,
            env=helper.profile_env(out / "abi-config"), capture_output=True, text=True,
            creationflags=helper.FLAGS, timeout=20)
        assert positive.returncode == 0 and "ABI 1" in positive.stdout
        missing = out / "without-engine"
        missing.mkdir()
        shutil.copy2(executable, missing / executable.name)
        negative = subprocess.run([str(missing / executable.name), "--verify-runtime"], cwd=missing,
            env=helper.profile_env(out / "negative-config"), capture_output=True, text=True,
            creationflags=helper.FLAGS, timeout=20)
        assert negative.returncode == 1 and "ABI 1" not in negative.stdout
        source = helper.Rpc(executable, out / "source", 0)
        clients.append(source)
        source.initialize()

        def script(rpc, text, error=False, model="vgcore"):
            return rpc.call("run_script", {"object_model": model, "source": text}, error=error)

        source.call("new_document", {"width_mm": 100, "height_mm": 70})
        script(source, "ActiveDocument.Unit=cdrMillimeter; const r=ActiveLayer.CreateRectangle(10,30,50,10,20,40,60,80); r.Fill.UniformColor.RGBAssign(20,150,80); r.Outline.SetNoOutline();")
        assert script(source, "ActiveDocument.Rulers.HUnits=cdrInch; ActiveDocument.Rulers.VUnits=cdrCentimeter; print(ActiveDocument.Unit,ActiveDocument.Rulers.HUnits,ActiveDocument.Rulers.VUnits);") == "3 1 4"
        configured = source.saved("configured.tdraw")
        assert settings(configured) == [3, 1, 4]
        render.assert_bounds(render.first_shape(configured), [10, 10, 50, 30])
        assert render.first_shape(configured)["kind"]["corners"]["radii"] == [2, 4, 8, 6]
        source.call("undo")
        before = source.saved("before-ruler-errors.tdraw")
        assert settings(before) == [3, None, None]
        for axis in ["HUnits", "VUnits"]:
            for invalid in ["null", "true", "'3'", "-1", "17", "0.5", "NaN", "Infinity"]:
                script(source, f"ActiveDocument.Rulers.{axis}={invalid};", error=True)
                assert source.saved(f"rejected-{axis}-{len(source.calls)}.tdraw") == before
        source.call("redo")
        assert source.saved("after-ruler-redo.tdraw") == configured
        report["invalid_ruler_setters_preserve_redo"] = True
        report["three_independent_units"] = True
        image = source.folder / "physical.png"
        source.call("export", {"path": str(image), "dpi": 254})
        report["render"] = render.verify_render(image, source.folder / "expected-physical.png")
        report["wrong_render_controls"] = render.wrong_render_controls(image)

        cases = [(None, None, None), (0, 16, 0), (1, 0, 16), (3, 1, 4),
                 (16, 4, 1), (17, 17, 4294967295), (4294967295, 4294967295, 17)]
        for index, wanted in enumerate(cases):
            document = copy.deepcopy(configured)
            document["metadata"]["script_unit"] = wanted[0]
            document["metadata"]["rulers"]["horizontal_unit"] = wanted[1]
            document["metadata"]["rulers"]["vertical_unit"] = wanted[2]
            fixture = source.folder / f"owned-case-{index}.tdraw"
            fixture.write_text(json.dumps(document), encoding="utf-8")
            source.call("open", {"path": str(fixture)})
            for version in [13, 27]:
                cdr = source.folder / f"case-{index}-v{version}.cdr"
                source.call("save", {"path": str(cdr), "version": version})
                reader = helper.Rpc(executable, out / f"reader-{index}-{version}", 0)
                clients.append(reader)
                reader.initialize()
                reader.call("open", {"path": str(cdr)})
                observed = reader.saved("reopened.tdraw")
                assert settings(observed) == list(wanted), (version, wanted, settings(observed))
                dimensions_text = script(reader, "const r=ActivePage.Shapes.All.Item(1); print(r.SizeWidth,r.SizeHeight);", model="native")
                dimensions = [float(value) for value in dimensions_text.split()]
                report["last_dimensions_mm"] = dimensions
                # CDR coordinates are converted through floating-point affine
                # matrices. Check physical lengths, not JS number formatting.
                assert len(dimensions) == 2 and all(abs(value - expected) < 1e-9
                    for value, expected in zip(dimensions, [40, 20])), dimensions
                if wanted[0] is None or wanted[0] <= 16:
                    assert script(reader, "print(ActiveDocument.Unit);") == str(1 if wanted[0] is None else wanted[0])
                    for axis, value in zip(["HUnits", "VUnits"], wanted[1:]):
                        assert script(reader, f"print(ActiveDocument.Rulers.{axis});") == str(3 if value is None else value)
                else:
                    script(reader, "print(ActiveDocument.Unit);", error=True)
                if index == 3 and version == 27:
                    png = reader.folder / "cdr-physical.png"
                    reader.call("export", {"path": str(png), "dpi": 254})
                    report["cdr_render"] = render.verify_render(png, reader.folder / "expected-cdr.png")
                report["roundtrips"].append({"version": version, "units": list(wanted),
                    "cdr_sha256": helper.sha(cdr), "reader_pid": reader.process.pid,
                    "source_pid": source.process.pid, "geometry_mm": [40, 20], "passed": True})
                assert reader.process.pid != source.process.pid
                reader.close()
                clients.remove(reader)
        report.update(passed=True, positive_and_negative_abi=True,
                      scope="Own supplementary units across native processes, RIFF13 and ZIP27; no claim of external editor compatibility.")
    except BaseException as error:
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        cleanup_errors = []
        for rpc in reversed(clients):
            try:
                rpc.close()
            except Exception as error:
                cleanup_errors.append(str(error))
        if cleanup_errors:
            report.update(passed=False, cleanup_errors=cleanup_errors)
        (out / "result.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(json.dumps({"passed": report["passed"], "evidence": str(out)}))
        if cleanup_errors and "error" not in report:
            raise RuntimeError("La limpieza de los procesos falló.")


if __name__ == "__main__":
    main()
