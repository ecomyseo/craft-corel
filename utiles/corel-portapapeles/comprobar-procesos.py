"""Comprueba selecciones nativas entre dos EXE propios, sin portapapeles del SO."""
from pathlib import Path
import argparse
import base64
import copy
import ctypes
import hashlib
import io
import json
import os
import queue
import shutil
import subprocess
import threading
import time
import xml.etree.ElementTree as ET
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
KNOWN_LAUNCHER = "86f493bdf977a154a1723c91adaa4dd5583e14be01c36a7cece5e85c118f7a22"
FLAGS = subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def workspace(path):
    path = path.resolve(strict=True)
    if not path.is_relative_to(ROOT):
        raise ValueError("La entrada debe pertenecer a este espacio de trabajo.")
    return path


def profile_env(path):
    path.mkdir(parents=True, exist_ok=True)
    return dict(os.environ, TRACEDRAW_CONFIG_DIR=str(path), CRAFT_COREL_CONFIG_DIR=str(path))


class Rpc:
    def __init__(self, executable, folder, policy):
        self.folder, self.calls, self.identifier = folder, [], 0
        folder.mkdir(parents=True)
        env = profile_env(folder / "config")
        (folder / "config/settings.json").write_text(json.dumps({"color": {
            "import_cmyk": policy, "intent": "Saturation",
            "black_point_compensation": False, "preserve_pure_black": False,
            "cmyk_profile": "Generic CMYK (open)", "cmyk_profile_path": "",
        }}), encoding="utf-8")
        self.stderr = (folder / "stderr.log").open("wb")
        self.process = subprocess.Popen([str(executable), "--mcp"], cwd=executable.parent,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.stderr,
            text=True, encoding="utf-8", env=env, creationflags=FLAGS)
        self.messages = queue.Queue()

        def receive():
            for line in self.process.stdout:
                try:
                    self.messages.put(json.loads(line))
                except json.JSONDecodeError:
                    self.messages.put({"invalid_reply": True})
            self.messages.put({"closed": True})
        threading.Thread(target=receive, daemon=True).start()

    def request(self, method, params):
        self.identifier += 1
        self.process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": self.identifier,
            "method": method, "params": params}, allow_nan=False) + "\n")
        self.process.stdin.flush()
        reply = self.messages.get(timeout=45)
        assert reply.get("id") == self.identifier and "error" not in reply, reply
        return reply["result"]

    def initialize(self):
        result = self.request("initialize", {"protocolVersion": "2024-11-05", "capabilities": {},
            "clientInfo": {"name": "Corel-owned-selection-test", "version": "1"}})
        assert result["serverInfo"]["name"] == "craft-corel"
        names = {tool["name"] for tool in self.request("tools/list", {})["tools"]}
        assert len(names) == 11 and {"copy_selection", "paste_selection"} <= names, names

    def call(self, name, arguments=None, error=False):
        result = self.request("tools/call", {"name": name, "arguments": arguments or {}})
        assert result.get("isError", False) == error, {"tool": name, "result": result}
        self.calls.append({"tool": name, "expected_error": error, "passed": True})
        return result["content"][0]["text"]

    def saved(self, name):
        path = self.folder / name
        self.call("save", {"path": str(path)})
        return json.loads(path.read_text(encoding="utf-8"))

    def close(self):
        try:
            if self.process.stdin and not self.process.stdin.closed:
                self.process.stdin.close()
            assert self.process.wait(timeout=10) == 0, "El servidor MCP terminó con un error."
        finally:
            if self.process.poll() is None:
                self.process.kill()
                self.process.wait(timeout=10)
            self.stderr.close()
            (self.folder / "calls.json").write_text(json.dumps(self.calls, indent=2) + "\n", encoding="utf-8")


def pictures(document):
    result = []
    pending = [document]
    while pending:
        value = pending.pop()
        if isinstance(value, dict):
            if isinstance(value.get("kind"), dict) and value["kind"].get("type") == "bitmap":
                result.append(value)
            pending.extend(reversed(list(value.values())))
        elif isinstance(value, list):
            pending.extend(reversed(value))
    return result


def check_picture(shape, expected, profile, oracle, expected_rgba):
    width, height = oracle["width_px"], oracle["height_px"]
    inks = shape["picture_inks"]
    assert inks["inks"] == [{"process": index} for index in range(4)]
    with Image.open(io.BytesIO(base64.b64decode(inks["png"], validate=True))) as planes:
        assert planes.mode == "L" and planes.size == (width, height * 4)
        preview_bytes = base64.b64decode(shape["kind"]["png"], validate=True)
        with Image.open(io.BytesIO(preview_bytes)) as image:
            preview = image.convert("RGBA")
        assert preview.size == (width, height)
        assert list(preview.tobytes()) == expected_rgba, "El color completo de la vista previa difiere."
        assert (shape["kind"]["width_px"], shape["kind"]["height_px"]) == (width, height)
        actual = []
        for y in range(height):
            for x in range(width):
                actual.extend(planes.getpixel((x, y + channel * height)) for channel in range(4))
                actual.append(preview.getpixel((x, y))[3])
    assert actual == expected, {"actual": actual, "expected": expected}
    assert inks["profile"] == profile
    fingerprint = 0xcbf29ce484222325
    for byte in preview_bytes:
        fingerprint = ((fingerprint ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    assert inks["of"] == fingerprint


def without_allocator(document):
    document = copy.deepcopy(document)
    document.pop("ids", None)
    return document


def roots(document):
    return [shape for page in document["pages"] for layer in page["layers"] for shape in layer["shapes"]]


def check_vector(shape, original, colour, translation):
    actual, expected = copy.deepcopy(shape), copy.deepcopy(original)
    for value in [actual, expected]:
        value.pop("id", None)
        value.pop("uid", None)
        value.pop("fill", None)
        value.pop("transform", None)
    assert actual == expected, "La figura vectorial perdió propiedades."
    fill = shape["fill"]
    assert fill["fill"] == "solid" and fill["model"] == "cmyk"
    assert all(abs(fill[key] - expected) < 0.000001 for key, expected in zip(["c", "m", "y", "k"], colour))
    wanted = original["transform"].copy()
    wanted[4] += translation[0]
    wanted[5] += translation[1]
    assert all(abs(a - b) < 0.000001 for a, b in zip(shape["transform"], wanted))


def check_bitmap_properties(shape, original, translation, incoming):
    actual, expected = copy.deepcopy(shape), copy.deepcopy(original)
    if incoming:
        for value in [actual, expected]:
            value.pop("id", None)
            value.pop("uid", None)
    for value in [actual, expected]:
        value["kind"].pop("png", None)
        for key in ["png", "of", "profile"]:
            value["picture_inks"].pop(key, None)
    expected["transform"][4] += translation[0]
    expected["transform"][5] += translation[1]
    assert actual == expected, "La imagen perdió geometría, visibilidad u otras propiedades."


def check_drawing(document, before, source, selected_ids, policy, oracle, display):
    old = {shape["id"]: shape for shape in roots(before)}
    all_shapes = {shape["id"]: shape for shape in roots(document)}
    selected = set(selected_ids)
    assert len(old) == 2 and len(roots(document)) == len(all_shapes) == 4
    assert len(selected) == 2 and selected.isdisjoint(old) and set(all_shapes) == set(old) | selected
    incoming = [all_shapes[identifier] for identifier in selected_ids]
    old_picture = next(shape for shape in old.values() if shape["kind"]["type"] == "bitmap")
    old_vector = next(shape for shape in old.values() if shape["kind"]["type"] == "rect")
    new_picture = next(shape for shape in incoming if shape["kind"]["type"] == "bitmap")
    new_vector = next(shape for shape in incoming if shape["kind"]["type"] == "rect")
    source_picture = next(shape for shape in roots(source) if shape["kind"]["type"] == "bitmap")
    source_vector = next(shape for shape in roots(source) if shape["kind"]["type"] == "rect")
    old_profile = oracle["source_a"] if policy == 2 else old_picture["picture_inks"]["profile"]
    new_profile = source_picture["picture_inks"]["profile"] if policy == 2 else oracle["destination_b"]
    check_picture(all_shapes[old_picture["id"]], oracle["b_to_a_cmyka"] if policy == 2 else oracle["original_cmyka"], old_profile, oracle,
        display["b_to_a_rgba"] if policy == 2 else display["original_b_rgba"])
    check_picture(new_picture, oracle["a_to_b_cmyka"] if policy == 0 else oracle["original_cmyka"], new_profile, oracle,
        display["a_to_b_rgba"] if policy == 0 else display["original_a_rgba"] if policy == 2 else display["original_b_rgba"])
    assert all_shapes[old_picture["id"]]["transform"] == old_picture["transform"]
    assert new_picture["transform"][:4] == source_picture["transform"][:4]
    delta = [new_picture["transform"][i] - source_picture["transform"][i] for i in [4, 5]]
    check_bitmap_properties(all_shapes[old_picture["id"]], old_picture, [0, 0], False)
    check_bitmap_properties(new_picture, source_picture, delta, True)
    check_vector(all_shapes[old_vector["id"]], old_vector, display["vector_b_to_a"] if policy == 2 else [0.17, 0.33, 0.49, 0.65], [0, 0])
    assert all_shapes[old_vector["id"]].get("uid") == old_vector.get("uid")
    check_vector(new_vector, source_vector, display["vector_a_to_b"] if policy == 0 else [0.17, 0.33, 0.49, 0.65], delta)
    assert document["ids"]["next"] >= max(all_shapes)


def reject_false_results(document, before, source, selected_ids, policy, oracle, display):
    wrong = copy.deepcopy(document)
    bitmap = pictures(wrong)[0]
    png = base64.b64decode(bitmap["kind"]["png"], validate=True)
    with Image.open(io.BytesIO(png)) as image:
        rgba = image.convert("RGBA")
    rgba.putdata([(255, 0, 255, rgba.getpixel((x, y))[3])
        for y in range(rgba.height) for x in range(rgba.width)])
    encoded = io.BytesIO()
    rgba.save(encoded, format="PNG")
    png = encoded.getvalue()
    bitmap["kind"]["png"] = base64.b64encode(png).decode("ascii")
    fingerprint = 0xcbf29ce484222325
    for byte in png:
        fingerprint = ((fingerprint ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    bitmap["picture_inks"]["of"] = fingerprint
    missing = copy.deepcopy(document)
    for page in missing["pages"]:
        for layer in page["layers"]:
            layer["shapes"] = [shape for shape in layer["shapes"]
                if not (shape["id"] in selected_ids and shape["kind"]["type"] == "rect")]
    invisible = copy.deepcopy(document)
    pictures(invisible)[0]["visible"] = False
    collapsed = copy.deepcopy(document)
    rectangle = pictures(collapsed)[0]["kind"]["rect"]
    if isinstance(rectangle, list):
        rectangle[2] = rectangle[0]
    else:
        rectangle["x1"] = rectangle["x0"]
    for damaged in [wrong, missing, invisible, collapsed]:
        try:
            check_drawing(damaged, before, source, selected_ids, policy, oracle, display)
        except AssertionError:
            continue
        raise AssertionError("El comprobador aceptó un resultado alterado deliberadamente.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--fixtures", required=True, type=Path)
    parser.add_argument("--display-oracles", required=True, type=Path)
    args = parser.parse_args()
    manifest_path, fixtures = workspace(args.manifest), workspace(args.fixtures)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    runtime = workspace(Path(manifest["directory"]))
    executable = runtime / "craft-corel.exe"
    assert sha(executable) == KNOWN_LAUNCHER == manifest["launcher_sha256"]
    assert executable.stat().st_size < 2_000_000
    for item in manifest["files"]:
        if Path(item["path"]).suffix.lower() in {".exe", ".dll"}:
            assert sha(runtime / item["path"]) == item["sha256"]
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.SetErrorMode.argtypes = [ctypes.c_uint]
    kernel32.SetErrorMode.restype = ctypes.c_uint
    kernel32.SetErrorMode(0x0001 | 0x0002 | 0x8000)
    out = ROOT / "evidencias/craft-corel-nativo/portapapeles/procesos" / str(time.time_ns())
    out.mkdir(parents=True)
    verification = subprocess.run([str(executable), "--verify-runtime"], cwd=runtime,
        env=profile_env(out / "verify-config"), capture_output=True, text=True, encoding="utf-8",
        creationflags=FLAGS, timeout=20)
    assert verification.returncode == 0 and "ABI 1" in verification.stdout
    negative = out / "without-engine"
    negative.mkdir()
    shutil.copy2(executable, negative / executable.name)
    rejected = subprocess.run([str(negative / executable.name), "--verify-runtime"], cwd=negative,
        env=profile_env(out / "negative-config"), capture_output=True, text=True, encoding="utf-8",
        creationflags=FLAGS, timeout=20)
    assert rejected.returncode == 1 and "ABI 1" not in rejected.stdout
    oracle_path = fixtures / "oracles.json"
    oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
    display_path = workspace(args.display_oracles)
    display = json.loads(display_path.read_text(encoding="utf-8"))
    assert display["a_to_b_cmyka"] == oracle["a_to_b_cmyka"] != oracle["original_cmyka"]
    assert display["b_to_a_cmyka"] == oracle["b_to_a_cmyka"] != oracle["original_cmyka"]
    assert oracle["source_a"]["data"] != oracle["destination_b"]["data"]
    inputs = [fixtures / "owned-source.tdraw", fixtures / "owned-destination.tdraw", oracle_path]
    frozen_source = json.loads(inputs[0].read_text(encoding="utf-8"))
    frozen_target = json.loads(inputs[1].read_text(encoding="utf-8"))
    input_hashes = {str(path): sha(path) for path in inputs}
    results = []
    for policy in [0, 1, 2]:
        source = target = None
        try:
            source = Rpc(executable, out / f"policy-{policy}-source", 0)
            target = Rpc(executable, out / f"policy-{policy}-target", policy)
            source.initialize()
            target.initialize()
            assert source.process.pid != target.process.pid
            description = json.loads(source.call("open", {"path": str(inputs[0])}))
            ids = [obj["id"] for layer in description["pages"][0]["layers"] for obj in layer["objects"]]
            before_source = source.saved("before-copy.tdraw")
            before_source_selection = json.loads(source.call("document_info"))["selection"]
            assert before_source["cmyk_profile"] == oracle["source_a"]
            check_picture(pictures(before_source)[0], oracle["original_cmyka"], pictures(frozen_source)[0]["picture_inks"]["profile"], oracle, display["original_a_rgba"])
            packet = json.loads(source.call("copy_selection", {"ids": ids}))
            assert packet["format"] == "craft-corel-selection-v1"
            assert packet["selected_ids"] == ids
            wire = packet["svg"]
            (source.folder / "published.svg").write_text(wire, encoding="utf-8")
            assert source.saved("after-copy.tdraw") == before_source
            assert json.loads(source.call("document_info"))["selection"] == before_source_selection
            target.call("open", {"path": str(inputs[1])})
            before_target = target.saved("before-paste.tdraw")
            before_selection = json.loads(target.call("document_info"))["selection"]
            assert before_target["cmyk_profile"] == oracle["destination_b"]
            check_picture(pictures(before_target)[0], oracle["original_cmyka"], pictures(frozen_target)[0]["picture_inks"]["profile"], oracle, display["original_b_rgba"])
            pasted = json.loads(target.call("paste_selection", {"svg": wire}))
            selected_ids = pasted["selected_ids"]
            committed = target.saved("after-paste.tdraw")
            check_drawing(committed, before_target, before_source, selected_ids, policy, oracle, display)
            reject_false_results(committed, before_target, before_source, selected_ids, policy, oracle, display)
            assert json.loads(target.call("document_info"))["selection"] == selected_ids
            profile = oracle["source_a"] if policy == 2 else oracle["destination_b"]
            assert committed["cmyk_profile"] == profile
            malformed = ET.fromstring(wire)
            block = malformed.find(".//{urn:craft-corel:selection}selection")
            assert block is not None
            block.set("version", "99")
            target.call("paste_selection", {"svg": ET.tostring(malformed, encoding="unicode")}, error=True)
            assert target.saved("after-refusal.tdraw") == committed
            assert json.loads(target.call("document_info"))["selection"] == selected_ids
            target.call("undo")
            undone = target.saved("undone.tdraw")
            assert without_allocator(undone) == without_allocator(before_target)
            assert undone["ids"]["next"] >= committed["ids"]["next"]
            assert json.loads(target.call("document_info"))["selection"] == before_selection
            target.call("redo")
            redone = target.saved("redone.tdraw")
            assert without_allocator(redone) == without_allocator(committed)
            assert redone["ids"]["next"] >= committed["ids"]["next"]
            assert json.loads(target.call("document_info"))["selection"] == selected_ids
            results.append({"policy": policy, "passed": True, "source_pid": source.process.pid,
                "target_pid": target.process.pid, "wire_sha256": hashlib.sha256(wire.encode()).hexdigest(),
                "full_cmyka_rgba_profile_alpha_fingerprint": True, "undo_redo_and_selection": True,
                "both_vectors_and_four_roots": True, "allocator_monotonic": True,
                "wrong_rgb_with_valid_fingerprint_and_dropped_vector_rejected":True,
                "invisible_and_collapsed_bitmap_rejected":True,
                "malformed_native_rejected_atomically": True})
        finally:
            if source is not None:
                source.close()
            if target is not None:
                target.close()
    assert {str(path): sha(path) for path in inputs} == input_hashes
    report = {"passed": True, "manifest": str(manifest_path), "launcher_sha256": KNOWN_LAUNCHER,
        "engine_sha256": sha(runtime / "craft-corel-engine.dll"), "input_sha256": input_hashes,
        "display_oracles":str(display_path),"display_oracles_sha256":sha(display_path),
        "display_oracle_scope":"Separate ICC primitive calculations, sharing the ICC library; no clipboard/import planner or original Corel oracle.",
        "positive_and_negative_abi": True, "policies": results, "system_clipboard_used": False,
        "visible_gui_tested": False, "original_coreldraw_executed": False, "full_equivalence_verified": False}
    (out / "result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"passed": True, "evidence": str(out), "policies": len(results)}))


if __name__ == "__main__":
    main()
