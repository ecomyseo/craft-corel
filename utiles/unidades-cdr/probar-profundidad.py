"""Sondea grupos TDRAW propios en el EXE real; no altera la pila ni abre ventanas."""
from pathlib import Path
import argparse
import ctypes
import importlib.util
import json
import os
import shutil
import struct
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
KNOWN_LAUNCHER = "86f493bdf977a154a1723c91adaa4dd5583e14be01c36a7cece5e85c118f7a22"
MAX_JSON_DEPTH = 1024
MAX_FIXTURE_BYTES = 8 * 1024 * 1024
MARKER = "__CRAFT_COREL_OWN_NATIVE_GROUP_NODE__"


def require(value, message):
    if not value:
        raise AssertionError(message)


def dump(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n",
                    encoding="utf-8")


def load_helper():
    path = ROOT / "utiles/corel-portapapeles/comprobar-procesos.py"
    spec = importlib.util.spec_from_file_location("owned_native_depth_rpc", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    require(module.ROOT == ROOT and module.KNOWN_LAUNCHER == KNOWN_LAUNCHER,
            "El helper debe pertenecer al mismo espacio de trabajo y lanzador conocido.")
    return module, path


def pe_stack(path):
    data = path.read_bytes()
    require(data[:2] == b"MZ", "Se requiere el lanzador PE conocido.")
    offset = struct.unpack_from("<I", data, 0x3C)[0]
    require(data[offset:offset + 4] == b"PE\0\0", "La firma PE no es válida.")
    optional = offset + 24
    magic = struct.unpack_from("<H", data, optional)[0]
    require(magic in (0x10B, 0x20B), "La cabecera opcional PE no es compatible.")
    return struct.unpack_from("<Q" if magic == 0x20B else "<I", data, optional + 72)[0]


def nesting(text):
    level = maximum = 0
    quoted = escaped = False
    for character in text:
        if quoted:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                quoted = False
        elif character == '"':
            quoted = True
        elif character in "[{":
            level += 1
            maximum = max(maximum, level)
        elif character in "]}":
            level -= 1
            require(level >= 0, "La plantilla JSON tiene cierres sin apertura.")
    require(not quoted and level == 0, "La plantilla JSON no está completa.")
    return maximum


def roots(document):
    return [shape for page in document["pages"] for layer in page["layers"]
            for shape in layer["shapes"]]


def equal(actual, expected):
    """Compare all fields iteratively, without dropping IDs, UIDs or resources."""
    pending = [(actual, expected)]
    while pending:
        left, right = pending.pop()
        if isinstance(right, dict):
            require(isinstance(left, dict) and left.keys() == right.keys(),
                    "Cambió el conjunto completo de campos del documento.")
            pending.extend((left[key], value) for key, value in right.items())
        elif isinstance(right, list):
            require(isinstance(left, list) and len(left) == len(right),
                    "Cambió la longitud de una colección del documento.")
            pending.extend(zip(left, right))
        else:
            require(not isinstance(left, (dict, list)) and left == right
                    and isinstance(left, bool) == isinstance(right, bool),
                    f"Cambió un valor nativo: {left!r} frente a {right!r}.")


def inspect(document, depth, leaf):
    candidates = roots(document)
    require(len(candidates) == 1, "Debe existir una sola raíz nativa.")
    node = candidates[0]
    identifiers, identities = [], []
    groups = 0
    while True:
        identifiers.append(node["id"])
        identities.append(node.get("uid"))
        if node["kind"]["type"] != "group":
            break
        require(node["transform"] == [1, 0, 0, 1, 0, 0], "Un grupo alteró la transformación.")
        require(len(node["kind"]["children"]) == 1, "Cada grupo debe conservar un único hijo.")
        node = node["kind"]["children"][0]
        groups += 1
    require(groups == depth, f"Se esperaban {depth} grupos y se encontraron {groups}.")
    equal(node, leaf)
    require(len(identifiers) == len(set(identifiers)) == depth + 1,
            "Los identificadores de todas las figuras deben ser únicos.")
    require(None not in identities and len(set(identities)) == depth + 1,
            "Las identidades persistentes deben ser válidas y únicas.")
    require(document["ids"]["next"] >= max(identifiers), "El asignador no cubre todas las figuras.")


def make_document(seed, depth):
    # Only the shallow seed is encoded recursively. Group JSON is wrapped as
    # text, so preparation cannot fail first at Python's default depth1000.
    document = json.loads(json.dumps(seed, allow_nan=False))
    leaf = roots(document)[0]
    node = json.dumps(leaf, ensure_ascii=False, separators=(",", ":"), allow_nan=False)
    base = document["ids"]["next"]
    used_uids = {leaf["uid"]}
    for level in range(depth):
        group = dict(leaf)
        identifier = base + level + 1
        identity = f"{0xCAFECC0E000000000000000000000000 + identifier:032x}"
        require(identity not in used_uids, "La identidad propia no puede coincidir con otra.")
        used_uids.add(identity)
        group.update(id=identifier, uid=identity, name=f"Owned native group {level + 1}",
                     transform=[1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
        group.pop("kind")
        prefix = json.dumps(group, ensure_ascii=False, separators=(",", ":"), allow_nan=False)
        node = prefix[:-1] + ',"kind":{"type":"group","children":[' + node + ']}}'
    document["ids"]["next"] = base + depth
    document["pages"][0]["layers"][0]["shapes"] = [MARKER]
    envelope = json.dumps(document, ensure_ascii=False, separators=(",", ":"), allow_nan=False)
    quoted_marker = json.dumps(MARKER)
    require(envelope.count(quoted_marker) == 1, "La marca de la plantilla debe ser única.")
    text = envelope.replace(quoted_marker, node, 1)
    require(len(text.encode("utf-8")) <= MAX_FIXTURE_BYTES, "La plantilla propia excede ocho MiB.")
    return text


def calibrated_cases(seed):
    maximum, outside = 128, MAX_JSON_DEPTH + 1
    require(nesting(make_document(seed, maximum)) <= MAX_JSON_DEPTH,
            "La plantilla de128 grupos debe caber en el contrato JSON.")
    require(nesting(make_document(seed, outside)) > MAX_JSON_DEPTH,
            "Debe existir una cota superior de calibración.")
    while maximum + 1 < outside:
        middle = (maximum + outside) // 2
        if nesting(make_document(seed, middle)) <= MAX_JSON_DEPTH:
            maximum = middle
        else:
            outside = middle
    require(maximum > 128, "La plantilla debe permitir las sondas de40 y128 grupos.")
    cases = []
    for depth, accepted in [(40, True), (128, True), (maximum, True), (maximum + 1, False)]:
        text = make_document(seed, depth)
        measured = nesting(text)
        require((measured <= MAX_JSON_DEPTH) == accepted, "La calibración JSON contradice el oráculo.")
        cases.append({"groups": depth, "json_depth": measured, "accepted": accepted, "text": text})
    return cases


def verify(helper, manifest, executable, runtime, out):
    require(helper.sha(executable) == KNOWN_LAUNCHER == manifest["launcher_sha256"],
            "El lanzador no coincide con la referencia conocida.")
    require(pe_stack(executable) == 1024 * 1024, "La pila principal PE debe reservar un MiB real.")
    hashes = {}
    for entry in manifest["files"]:
        if Path(entry["path"]).suffix.lower() in (".dll", ".exe"):
            binary = (runtime / entry["path"]).resolve(strict=True)
            require(binary.is_relative_to(runtime), "Una entrada del manifiesto escapó del runtime.")
            require(helper.sha(binary) == entry["sha256"], f"Huella incorrecta: {binary.name}.")
            hashes[binary.relative_to(runtime).as_posix()] = entry["sha256"]
    actual = {path.relative_to(runtime).as_posix() for path in runtime.rglob("*")
              if path.is_file() and path.suffix.lower() in (".dll", ".exe")}
    require(actual == set(hashes) and "craft-corel-engine.dll" in hashes,
            "El manifiesto debe cubrir todos los EXE y DLL reales del runtime.")
    positive = subprocess.run([str(executable), "--verify-runtime"], cwd=runtime,
        env=helper.profile_env(out / "positive-config"), capture_output=True, text=True,
        encoding="utf-8", creationflags=helper.FLAGS, timeout=20)
    require(positive.returncode == 0 and "ABI 1" in positive.stdout, "Falló el control positivo de ABI.")
    negative_dir = out / "without-engine"
    negative_dir.mkdir()
    negative_exe = negative_dir / executable.name
    shutil.copy2(executable, negative_exe)
    negative = subprocess.run([str(negative_exe), "--verify-runtime"], cwd=negative_dir,
        env=helper.profile_env(out / "negative-config"), capture_output=True, text=True,
        encoding="utf-8", creationflags=helper.FLAGS, timeout=20)
    require(negative.returncode == 1 and "ABI 1" not in negative.stdout,
            "El control sin motor debe fallar antes de MCP.")
    dump(out / "abi.json", {"positive": {"exit": positive.returncode, "stdout": positive.stdout,
        "stderr": positive.stderr}, "negative": {"exit": negative.returncode,
        "stdout": negative.stdout, "stderr": negative.stderr}, "binary_sha256": hashes,
        "main_pe_stack_reserve": pe_stack(executable)})
    return hashes


def close(rpc, report, failed):
    report["pid"] = rpc.process.pid
    report["exit_code_before_close"] = rpc.process.poll()
    try:
        rpc.close()
    except Exception as error:
        report["cleanup_error"] = f"{type(error).__name__}: {error}"
        if not failed:
            raise
    finally:
        report["exit_code"] = rpc.process.poll()
        if report["exit_code"] is not None:
            report["exit_code_u32"] = report["exit_code"] & 0xFFFFFFFF


def positive_case(helper, executable, folder, path, expected, groups, leaf):
    rpc = helper.Rpc(executable, folder, 0)
    report = {"passed": False, "phase": "initialize"}
    failed = False
    try:
        rpc.initialize()
        report["phase"] = "open native file and describe"
        info = json.loads(rpc.call("open", {"path": str(path)}))
        require(info["selection"] == [], "Abrir un archivo debe comenzar sin selección.")
        summary = info["pages"][0]["layers"][0]["objects"]
        require(len(summary) == 1, "La descripción MCP debe conservar una sola raíz.")
        summary, described = summary[0], 0
        while True:
            require(summary["bounds_mm"] == {"x0": 10, "y0": 10, "x1": 50, "y1": 30},
                    "Los límites físicos calculados por el producto cambiaron.")
            if summary["type"] != "group":
                break
            require(len(summary["children"]) == 1, "La descripción omitió un hijo nativo.")
            summary = summary["children"][0]
            described += 1
        require(described == groups and summary["type"] == "rectangle",
                "La descripción debe atravesar todos los grupos reales.")
        report["phase"] = "save full native document"
        actual = rpc.saved("roundtrip.tdraw")
        equal(actual, expected)
        inspect(actual, groups, leaf)
        report.update(passed=True, phase="complete", objects=groups + 1,
                      full_document_ids_uids_verified=True, operating_system_clipboard_used=False)
        return report, folder / "roundtrip.tdraw"
    except BaseException as error:
        failed = True
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        try:
            close(rpc, report, failed)
        finally:
            dump(folder / "result.json", report)


def negative_case(helper, executable, folder, path):
    rpc = helper.Rpc(executable, folder, 0)
    report = {"passed": False, "phase": "initialize"}
    failed = False
    try:
        rpc.initialize()
        rpc.call("new_document", {"width_mm": 100, "height_mm": 70, "title": "Native refusal target"})
        rpc.call("run_script", {"object_model": "vgcore", "source":
            "ActiveDocument.Unit=cdrMillimeter; const s=ActiveLayer.CreateRectangle2(1,1,3,4);"
            "s.Name='Existing sentinel'; s.CreateSelection();"})
        rpc.call("run_script", {"object_model": "vgcore", "source":
            "ActiveLayer.CreateRectangle2(30,30,5,6).Name='Held redo sentinel';"})
        control = rpc.saved("held-redo.tdraw")
        selection = json.loads(rpc.call("document_info"))["selection"]
        require(len(roots(control)) == 2 and len(selection) == 1, "El control debe tener selección y dos figuras.")
        rpc.call("undo")
        before = rpc.saved("before-refusal.tdraw")
        info = json.loads(rpc.call("document_info"))
        require(len(roots(before)) == 1 and info["selection"] == selection, "Debe conservarse selección activa y redo.")
        report["phase"] = "reject over-depth native open"
        error = rpc.call("open", {"path": str(path)}, error=True)
        require("levels deep" in error and "1024" in error,
                f"Debe rechazar la guarda de profundidad, no otro error: {error}")
        equal(json.loads(rpc.call("document_info")), info)
        equal(rpc.saved("after-refusal.tdraw"), before)
        rpc.call("redo")
        equal(rpc.saved("redone.tdraw"), control)
        require(json.loads(rpc.call("document_info"))["selection"] == selection, "El rechazo alteró el redo o la selección.")
        report.update(passed=True, phase="complete", native_error=error,
                      exact_exposed_document_info_preserved=True, full_native_json_preserved=True,
                      preexisting_redo_preserved=True, operating_system_clipboard_used=False)
        return report
    except BaseException as error:
        failed = True
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        try:
            close(rpc, report, failed)
        finally:
            dump(folder / "result.json", report)


def main():
    require(__debug__, "Las comprobaciones requieren Python sin optimizar aserciones.")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    require("RUST_MIN_STACK" not in os.environ, "No debe heredarse RUST_MIN_STACK en esta sonda.")
    # Python only needs to inspect native JSON whose allowed nesting exceeds its
    # own default1000. This never changes the product environment or PE stack.
    previous_limit = sys.getrecursionlimit()
    sys.setrecursionlimit(max(previous_limit, 4096))
    helper, helper_path = load_helper()
    manifest_path = helper.workspace(args.manifest)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    runtime = helper.workspace(Path(manifest["directory"]))
    executable = runtime / "craft-corel.exe"
    out = ROOT / "evidencias/craft-plumilla/unidades-cdr/pruebas/profundidad" / str(time.time_ns())
    out.mkdir(parents=True)
    state = {"passed": False, "phase": "ABI controls", "manifest": str(manifest_path),
        "manifest_sha256": helper.sha(manifest_path), "script_sha256": helper.sha(Path(__file__)),
        "helper_sha256": helper.sha(helper_path), "results": [], "rust_min_stack_modified": False,
        "python_json_recursion_limit": sys.getrecursionlimit(), "full_equivalence_verified": False,
        "original_coreldraw_executed": False, "system_clipboard_used": False, "visible_gui_tested": False}
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.SetErrorMode.argtypes = [ctypes.c_uint]
    kernel.SetErrorMode.restype = ctypes.c_uint
    kernel.SetErrorMode(0x0001 | 0x0002 | 0x8000)
    source = None
    try:
        state["binary_sha256"] = verify(helper, manifest, executable, runtime, out)
        state["phase"] = "generate shallow owned seed"
        source = helper.Rpc(executable, out / "source", 0)
        source.initialize()
        source.call("new_document", {"width_mm": 100, "height_mm": 70, "title": "Owned native depth seed"})
        source.call("run_script", {"object_model": "vgcore", "source":
            "ActiveDocument.Unit=cdrMillimeter;"
            "ActiveDocument.Rulers.HUnits=cdrInch; ActiveDocument.Rulers.VUnits=cdrCentimeter;"
            "const r=ActiveLayer.CreateRectangle2(10,10,40,20,2,4,6,8);"
            "r.Name='Owned depth leaf'; r.Fill.UniformColor.RGBAssign(20,150,80); r.Outline.SetNoOutline();"})
        seed = source.saved("seed.tdraw")
        source.close()
        source = None
        require(len(seed["pages"]) == len(seed["pages"][0]["layers"]) == len(roots(seed)) == 1,
                "La semilla propia debe tener una página, una capa y una figura.")
        require(not seed.get("fonts") and not seed.get("symbols") and not seed.get("master"),
                "La semilla no debe contener árboles ocultos ni recursos ajenos.")
        leaf = roots(seed)[0]
        require(leaf["kind"]["type"] == "rect" and leaf["kind"]["rect"] ==
                {"x0": 10, "y0": 10, "x1": 50, "y1": 30}, "La geometría física de control no coincide.")
        require(leaf["kind"]["corners"]["radii"] == [2, 4, 8, 6], "Las cuatro esquinas deben ser distintas.")
        require(leaf["fill"]["model"] == "rgb", "La semilla debe conservar RGB propio.")
        require(all(abs(leaf["fill"][key] - value / 255) < 1e-7
                    for key, value in zip(("r", "g", "b"), (20, 150, 80))),
                "El color de control debe ser RGB20/150/80.")
        for case in calibrated_cases(seed):
            depth = case["groups"]
            state["phase"] = f"native depth {depth}"
            folder = out / f"depth-{depth}"
            folder.mkdir()
            path = folder / "input.tdraw"
            path.write_text(case["text"], encoding="utf-8")
            fingerprint = helper.sha(path)
            expected = json.loads(case["text"])
            inspect(expected, depth, leaf)
            entry = {key: case[key] for key in ("groups", "json_depth", "accepted")}
            entry.update(input=str(path), input_sha256=fingerprint, input_bytes=path.stat().st_size,
                         passed=False, evidence=str(folder))
            state["results"].append(entry)
            dump(out / "result.json", state)
            if case["accepted"]:
                result, saved = positive_case(helper, executable, folder / "reader", path, expected, depth, leaf)
                entry["first_process"] = result
                result, _ = positive_case(helper, executable, folder / "reopened", saved, expected, depth, leaf)
                entry["second_process"] = result
            else:
                entry["rejection"] = negative_case(helper, executable, folder / "refusal", path)
            require(helper.sha(path) == fingerprint, "El producto cambió el archivo fuente de la sonda.")
            entry["passed"] = True
            dump(out / "result.json", state)
        state.update(passed=True, phase="complete", main_pe_stack_reserve_bytes=1024 * 1024,
                     max_json_depth_contract=MAX_JSON_DEPTH)
    except BaseException as error:
        state["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        if source is not None:
            try:
                source.close()
            except Exception as error:
                state["source_cleanup_error"] = f"{type(error).__name__}: {error}"
        for case in state["results"]:
            path = Path(case["input"])
            case["input_intact"] = path.is_file() and helper.sha(path) == case["input_sha256"]
            if not case["input_intact"]:
                state["passed"] = False
                state.setdefault("error", "Un archivo fuente propio cambió durante la prueba.")
        dump(out / "result.json", state)
        sys.setrecursionlimit(previous_limit)
        print(json.dumps({"passed": state["passed"], "evidence": str(out), "phase": state["phase"]}))


if __name__ == "__main__":
    main()
