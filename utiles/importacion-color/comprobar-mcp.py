"""Prueba el motor Corel real con oráculos CMYKA generados independientemente."""
from pathlib import Path
import argparse
import base64
import copy
import hashlib
import io
import json
import os
import queue
import shutil
import subprocess
import threading
import time
import pefile
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser()
parser.add_argument("--engine", type=Path, required=True)
parser.add_argument("--fixtures", type=Path)
parser.add_argument("--launcher", type=Path)
parser.add_argument("--evidence-root", type=Path)
args = parser.parse_args()
engine = args.engine.resolve(strict=True)
assert engine.is_relative_to(ROOT)
fixture_root = args.fixtures or max(
    (p.parent for p in (ROOT / "evidencias/craft-plumilla/importacion-color/generated").glob("file-matrix-*/owned-destination.tdraw")),
    key=lambda p: p.stat().st_mtime,
)
fixture_root = fixture_root.resolve(strict=True)
assert fixture_root.is_relative_to(ROOT)
oracle = json.loads((fixture_root / "oracles.json").read_text(encoding="utf-8"))
run = str(time.time_ns())
evidence_root = (args.evidence_root or ROOT / "evidencias/craft-plumilla/importacion-color/mcp").resolve()
assert evidence_root.is_relative_to(ROOT)
out = evidence_root / run
out.mkdir(parents=True)
runtime = out / "runtime"
runtime.mkdir()
previous = json.loads((ROOT / "evidencias/portable/actual.json").read_text(encoding="utf-8"))
launcher = (args.launcher or Path(previous["directory"]) / "craft-corel.exe").resolve(strict=True)
assert launcher.is_relative_to(ROOT)
assert launcher.stat().st_size < 2_000_000, "Se requiere el launcher DLL pequeño, no un EXE monolítico."
shutil.copy2(launcher, runtime / "craft-corel.exe")
toolchain = ROOT / ".tools/rustup/toolchains/1.97.0-x86_64-pc-windows-msvc"
search = [engine.parent, engine.parent / "deps", toolchain / "lib/rustlib/x86_64-pc-windows-msvc/lib",
          Path("C:/Program/VC/Redist/MSVC/14.44.35112/x64/Microsoft.VC143.CRT")]
pending, seen, dependencies = [(engine, "craft-corel-engine.dll")], set(), []
while pending:
    source, name = pending.pop()
    if name.lower() in seen:
        continue
    seen.add(name.lower())
    shutil.copy2(source, runtime / name)
    with pefile.PE(str(source), fast_load=True) as pe:
        assert pe.FILE_HEADER.Machine == 0x8664
        pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_IMPORT"]])
        imports = [entry.dll.decode("ascii") for entry in getattr(pe, "DIRECTORY_ENTRY_IMPORT", [])]
    staged_hash = hashlib.sha256((runtime / name).read_bytes()).hexdigest()
    assert staged_hash == hashlib.sha256(source.read_bytes()).hexdigest(), "El artefacto cambió durante la copia."
    dependencies.append({"file": name, "sha256": staged_hash, "imports": imports})
    for item in imports:
        found = next((folder / item for folder in search if (folder / item).is_file()), None)
        if found:
            pending.append((found, item))
        else:
            assert item.lower().startswith(("api-ms-win-", "ext-ms-win-")) or (Path(os.environ["SystemRoot"]) / "System32" / item).is_file(), item
verify_profile = out / "verify-config"
verify_profile.mkdir()
verify_environment = dict(os.environ, TRACEDRAW_CONFIG_DIR=str(verify_profile), CRAFT_COREL_CONFIG_DIR=str(verify_profile))
verification = subprocess.run([str(runtime / "craft-corel.exe"), "--verify-runtime"], cwd=runtime, env=verify_environment,
                              capture_output=True, text=True, encoding="utf-8", check=True,
                              creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS, timeout=20)
assert "ABI 1" in verification.stdout, verification.stdout
(out / "runtime.json").write_text(json.dumps({"dependencies": dependencies, "verify_stdout": verification.stdout,
    "launcher_source":str(launcher), "launcher_sha256":hashlib.sha256((runtime / "craft-corel.exe").read_bytes()).hexdigest(),
    "frozen_oracles_sha256":hashlib.sha256((fixture_root / "oracles.json").read_bytes()).hexdigest()}, indent=2), encoding="utf-8")


class Mcp:
    def __init__(self, folder, policy):
        self.folder = folder
        folder.mkdir()
        profile = folder / "config"
        profile.mkdir()
        (profile / "settings.json").write_text(json.dumps({"color": {
            "import_cmyk": policy, "intent": "Saturation", "black_point_compensation": False,
            "preserve_pure_black": False, "cmyk_profile": "Generic CMYK (open)", "cmyk_profile_path": "",
        }}), encoding="utf-8")
        self.log = (folder / "stderr.log").open("wb")
        self.process = subprocess.Popen([str(runtime / "craft-corel.exe"), "--mcp"], cwd=runtime,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.log, text=True, encoding="utf-8",
            env=dict(os.environ, TRACEDRAW_CONFIG_DIR=str(profile), CRAFT_COREL_CONFIG_DIR=str(profile)),
            creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
        self.messages = queue.Queue()
        self.identifier = 0
        self.calls = []
        def receive():
            for line in self.process.stdout:
                try:
                    self.messages.put(json.loads(line))
                except json.JSONDecodeError:
                    pass
            self.messages.put({"closed": True})
        threading.Thread(target=receive, daemon=True).start()
        try:
            initialized = self.request("initialize", {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "Lumen-import-colour", "version": "1"}})
            assert initialized["serverInfo"]["name"] == "craft-corel", initialized
            names = {tool["name"] for tool in self.request("tools/list", {})["tools"]}
            assert "import" in names, "Este motor todavía no expone importación MCP."
        except BaseException:
            self.close()
            raise

    def request(self, method, params):
        self.identifier += 1
        self.process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": self.identifier, "method": method, "params": params}, allow_nan=False) + "\n")
        self.process.stdin.flush()
        while True:
            result = self.messages.get(timeout=90)
            assert not result.get("closed"), "El proceso MCP se cerró."
            if result.get("id") == self.identifier:
                assert "error" not in result, result
                return result["result"]

    def call(self, name, arguments=None, error=False):
        result = self.request("tools/call", {"name": name, "arguments": arguments or {}})
        assert result.get("isError", False) == error, result
        self.calls.append({"tool": name, "expected_error": error, "passed": True})
        return result["content"][0]["text"]

    def saved(self, path):
        self.call("save", {"path": str(path)})
        return json.loads(path.read_text(encoding="utf-8"))

    def close(self):
        self.process.stdin.close()
        try:
            assert self.process.wait(timeout=10) == 0
        finally:
            if self.process.poll() is None:
                self.process.terminate()
            self.log.close()
            (self.folder / "calls.json").write_text(json.dumps(self.calls, indent=2), encoding="utf-8")


def pictures(document):
    result = []
    def visit(value):
        if isinstance(value, dict):
            if isinstance(value.get("kind"), dict) and value["kind"].get("type") == "bitmap":
                result.append(value)
            for child in value.values():
                visit(child)
        elif isinstance(value, list):
            for child in value:
                visit(child)
    visit(document)
    return result


def fingerprint(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return value


def check_picture(shape, expected, profile):
    width, height = oracle["width_px"], oracle["height_px"]
    inks = shape["picture_inks"]
    assert inks["inks"] == [{"process": index} for index in range(4)]
    assert (shape["kind"]["width_px"], shape["kind"]["height_px"]) == (width, height)
    image = Image.open(io.BytesIO(base64.b64decode(inks["png"])))
    assert image.size == (width, height * 4) and image.mode == "L"
    preview_bytes = base64.b64decode(shape["kind"]["png"])
    preview = Image.open(io.BytesIO(preview_bytes)).convert("RGBA")
    actual = []
    for y in range(height):
        for x in range(width):
            actual.extend(image.getpixel((x, y + plane * height)) for plane in range(4))
            actual.append(preview.getpixel((x, y))[3])
    assert actual == expected, {"actual": actual, "expected": expected}
    assert base64.b64decode(inks["profile"]["data"]) == base64.b64decode(profile["data"])
    assert inks["of"] == fingerprint(preview_bytes)


def content_without_allocator(document):
    document = copy.deepcopy(document)
    document.pop("ids", None)
    return document


results = []
for policy in [0, 1, 2]:
    for filename in ["owned-source.tif", "owned-source-v27.cdr", "owned-source.tdraw"]:
        folder = out / (str(policy) + "-" + filename.replace(".", "-"))
        mcp = Mcp(folder, policy)
        try:
            source = folder / filename
            shutil.copy2(fixture_root / filename, source)
            original_bytes = source.read_bytes()
            destination = folder / "destination.tdraw"
            shutil.copy2(fixture_root / "owned-destination.tdraw", destination)
            mcp.call("open", {"path": str(destination)})
            mcp.call("run_script", {"source": "ActiveLayer.CreateRectangle(60, 40, 63, 43);"})
            current_path = folder / "current.tdraw"
            before = mcp.saved(current_path)
            pre_info = json.loads(mcp.call("document_info"))
            original_picture_id = pictures(before)[0]["id"]
            imported = json.loads(mcp.call("import", {"path": str(source)}))
            assert imported["imported_ids"]
            after = mcp.saved(current_path)
            target = oracle["source_a"] if policy == 2 else oracle["destination_b"]
            assert after["cmyk_profile"]["data"] == target["data"]
            old, new = None, None
            for picture in pictures(after):
                if picture["id"] == original_picture_id:
                    old = picture
                else:
                    assert new is None
                    new = picture
            assert old is not None and new is not None
            check_picture(old, oracle["b_to_a_cmyka"] if policy == 2 else oracle["original_cmyka"], target)
            check_picture(new, oracle["a_to_b_cmyka"] if policy == 0 else oracle["original_cmyka"], target)
            assert oracle["a_to_b_cmyka"] != oracle["original_cmyka"], "El control de transformación debe ser distinguible."
            if policy == 0:
                try:
                    check_picture(new, oracle["original_cmyka"], target)
                except AssertionError:
                    pass
                else:
                    raise AssertionError("El oráculo incorrecto aceptó tintas sin convertir.")
            mcp.call("undo")
            undone_info = json.loads(mcp.call("document_info"))
            undone = mcp.saved(current_path)
            assert content_without_allocator(undone) == content_without_allocator(before)
            assert undone_info["selection"] == pre_info["selection"]
            mcp.call("redo")
            redone_info = json.loads(mcp.call("document_info"))
            redone = mcp.saved(current_path)
            assert redone == after
            assert redone_info["selection"] == imported["imported_ids"]
            assert source.read_bytes() == original_bytes
            # Invalid placement, missing file, and a deliberately malformed late symbol.
            bad = copy.deepcopy(json.loads((fixture_root / "owned-source.tdraw").read_text(encoding="utf-8")))
            malformed = copy.deepcopy(pictures(bad)[0])
            malformed["id"] = bad["ids"]["next"]
            bad["ids"]["next"] += 1
            malformed["picture_inks"]["png"] = base64.b64encode(b"invalid ink PNG").decode("ascii")
            bad["symbols"] = [{"name": "Late malformed unused symbol", "shape": malformed}]
            bad_path = folder / "bad-late.tdraw"
            bad_path.write_text(json.dumps(bad), encoding="utf-8")
            for arguments in [{"path": str(source), "x_mm": 10}, {"path": str(folder / "missing.tif")}, {"path": str(bad_path)}]:
                info_before = mcp.call("document_info")
                mcp.call("import", arguments, error=True)
                assert mcp.call("document_info") == info_before
                assert mcp.saved(current_path) == redone
            assert source.read_bytes() == original_bytes
            results.append({"policy": policy, "format": filename, "passed": True,
                            "checks": "direct original CMYKA/ICC/fingerprint, distinct conversion control, undo/redo, selection, three atomic errors, source intact"})
        finally:
            mcp.close()
folder = out / "unknown-policy"
mcp = Mcp(folder, 255)
try:
    mcp.call("open", {"path": str(fixture_root / "owned-destination.tdraw")})
    path = folder / "current.tdraw"
    before = mcp.saved(path)
    info = mcp.call("document_info")
    mcp.call("import", {"path": str(fixture_root / "owned-source.tif")}, error=True)
    assert mcp.call("document_info") == info
    assert mcp.saved(path) == before
    results.append({"policy": 255, "passed": True, "checks": "unknown selector rejects import atomically"})
finally:
    mcp.close()
(out / "results.json").write_text(json.dumps({"passed": len(results), "cases": results,
    "engine_sha256": next(row["sha256"] for row in dependencies if row["file"] == "craft-corel-engine.dll"), "fixtures": str(fixture_root),
    "scope": "real native EXE/DLL MCP; no original Corel, desktop or physical output"}, indent=2, ensure_ascii=False), encoding="utf-8")
print(json.dumps({"passed": len(results), "evidence": str(out)}, ensure_ascii=False))
