"""Prueba el ZIP portable real mediante su EXE y DLL, sin abrir ventanas."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import queue
import shutil
import subprocess
import threading
import time
import zipfile
import pefile
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--manifest", type=Path, default=ROOT / "evidencias/portable/actual.json")
parser.add_argument("--expected-tools", type=int, default=8)
parser.add_argument("--verify-headless-diagnostics", action="store_true")
args = parser.parse_args()
state = json.loads(args.manifest.read_text(encoding="utf-8"))
EVIDENCE = ROOT / "evidencias/portable" / state["version"]
run_id = str(time.time_ns())
OUT = EVIDENCE / ("prueba-" + run_id)
OUT.mkdir(parents=True, exist_ok=False)
EXTRACTED = ROOT / ".build/portable-verificacion" / (state["version"] + "-" + run_id)
EXTRACTED.mkdir(parents=True, exist_ok=False)
archive = Path(state["zip"])
assert hashlib.sha256(archive.read_bytes()).hexdigest() == state["zip_sha256"]
with zipfile.ZipFile(archive) as package:
    for item in package.infolist():
        target = (EXTRACTED / item.filename).resolve()
        assert target.is_relative_to(EXTRACTED.resolve())
    package.extractall(EXTRACTED)
folder = EXTRACTED / Path(state["directory"]).name
for row in state["files"]:
    assert hashlib.sha256((folder / row["path"]).read_bytes()).hexdigest() == row["sha256"]

profile = OUT / "perfil"
profile.mkdir()
shutil.copy2(folder / "craft-plumilla-settings/settings.json", profile / "settings.json")
env = dict(os.environ, TRACEDRAW_CONFIG_DIR=str(profile), CRAFT_COREL_CONFIG_DIR=str(profile))
assert (folder / "craft-plumilla.exe").stat().st_size < 2 * 1024 * 1024, "El EXE no es el lanzador DLL esperado."
verified = subprocess.run([str(folder / "craft-plumilla.exe"), "--verify-runtime"], cwd=folder,
                          stdin=subprocess.DEVNULL, capture_output=True, text=True, encoding="utf-8",
                          timeout=30, check=True, env=env,
                          creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
assert "ABI 1" in verified.stdout, "El lanzador no confirmó la ABI del motor."
(OUT / "runtime-verificado.txt").write_text(verified.stdout + verified.stderr, encoding="utf-8")
negative = OUT / "negativo-sin-motor"
negative.mkdir()
launcher = folder / "craft-plumilla.exe"
shutil.copy2(launcher, negative / launcher.name)
assert hashlib.sha256((negative / launcher.name).read_bytes()).hexdigest() == hashlib.sha256(launcher.read_bytes()).hexdigest()
with pefile.PE(str(launcher), fast_load=True) as pe:
    pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_IMPORT"]])
    launcher_imports = [item.dll.decode("ascii") for item in getattr(pe, "DIRECTORY_ENTRY_IMPORT", [])]
assert "craft-plumilla-engine.dll" not in {name.lower() for name in launcher_imports}, "El motor debe cargarse por la ruta de verificación explícita."
for dependency in launcher_imports:
    if (folder / dependency).is_file():
        shutil.copy2(folder / dependency, negative / dependency)
assert not (negative / "craft-plumilla-engine.dll").exists()
without_engine = subprocess.run([str(negative / launcher.name), "--verify-runtime"], cwd=negative,
                                stdin=subprocess.DEVNULL, capture_output=True, text=True, encoding="utf-8",
                                timeout=30, env=env,
                                creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
assert without_engine.returncode != 0 and "craft-plumilla-engine.dll" in without_engine.stderr
assert "ABI 1" not in without_engine.stdout
(OUT / "negativo-sin-motor.json").write_text(json.dumps({
    "launcher_sha256": hashlib.sha256(launcher.read_bytes()).hexdigest(), "engine_absent": True,
    "exit_code": without_engine.returncode, "stdout": without_engine.stdout, "stderr": without_engine.stderr,
    "directory": str(negative), "launcher_imports": launcher_imports,
}, indent=2, ensure_ascii=False), encoding="utf-8")
diagnostic_errors = []
if args.verify_headless_diagnostics:
    for flag in ["--profile-export", "--profile-cdr-open"]:
        result = subprocess.run([str(negative / launcher.name), flag], cwd=negative,
                                stdin=subprocess.DEVNULL, capture_output=True, text=True, encoding="utf-8",
                                timeout=5, env=env,
                                creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
        assert result.returncode != 0 and "craft-plumilla-engine.dll" in result.stderr
        diagnostic_errors.append({"flag": flag, "exit_code": result.returncode, "stderr": result.stderr})
    (OUT / "negativos-diagnosticos.json").write_text(
        json.dumps(diagnostic_errors, indent=2, ensure_ascii=False), encoding="utf-8")
process = subprocess.Popen([str(folder / "craft-plumilla.exe"), "--mcp"], cwd=folder,
                           stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                           text=True, encoding="utf-8", env=env,
                           creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
messages = queue.Queue()


def receive():
    for line in process.stdout:
        try:
            messages.put(json.loads(line))
        except json.JSONDecodeError:
            pass


threading.Thread(target=receive, daemon=True).start()
identifier = 0
checks = []


def request(method, params):
    global identifier
    identifier += 1
    process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": identifier, "method": method, "params": params}) + "\n")
    process.stdin.flush()
    while True:
        reply = messages.get(timeout=40)
        if reply.get("id") == identifier:
            assert "error" not in reply, reply
            return reply["result"]


def call(name, args=None, expect_error=False):
    reply = request("tools/call", {"name": name, "arguments": args or {}})
    assert reply.get("isError", False) == expect_error, reply
    checks.append({"tool": name, "expected_error": expect_error, "ok": True})
    return reply["content"][0]["text"]


def count(text):
    doc = json.loads(text)
    return sum(len(layer["objects"]) for layer in doc["pages"][0]["layers"])


try:
    initialized = request("initialize", {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "portable-verification", "version": "1"}})
    assert initialized["serverInfo"]["name"] == "craft-plumilla"
    tools = request("tools/list", {})["tools"]
    assert len(tools) == args.expected_tools
    if args.expected_tools >= 9:
        assert "import" in {tool["name"] for tool in tools}
    call("new_document", {"width_mm": 100, "height_mm": 70, "title": "Prueba portable"})
    text = call("run_script", {"source": "var s = ActiveLayer.CreateRectangle(10, 10, 40, 30); s.Fill.ApplyUniformFill(Color.RGB(25, 163, 95)); print(ActivePage.Shapes.Count);"})
    assert text == "1"
    assert count(call("undo")) == 0
    assert count(call("redo")) == 1
    call("save", {"path": str(OUT / "dibujo.tdraw")})
    for suffix in ["png", "jpg", "webp", "pdf", "svg"]:
        path = OUT / ("dibujo." + suffix)
        call("export", {"path": str(path), "dpi": 96, "webp_lossless": True})
        data = path.read_bytes()
        assert len(data) > 0
        if suffix in ["png", "jpg", "webp"]:
            with Image.open(path) as image:
                assert image.format == {"png": "PNG", "jpg": "JPEG", "webp": "WEBP"}[suffix]
                rgb = image.convert("RGB")
                pixels = rgb.tobytes()
                assert sum(pixels[i + 1] > pixels[i] + 30 for i in range(0, len(pixels), 3)) > 500
        elif suffix == "pdf":
            assert data.startswith(b"%PDF-")
        else:
            assert b"<svg" in data
    call("save", {"path": str(OUT / "dibujo.cdr"), "version": 27})
    # A different path forces a file read instead of selecting a saved tab.
    read_path = OUT / "dibujo-leido.cdr"
    shutil.copy2(OUT / "dibujo.cdr", read_path)
    call("new_document", {"width_mm": 10, "height_mm": 10})
    assert count(call("open", {"path": str(read_path)})) == 1
    if args.expected_tools >= 9:
        imported = json.loads(call("import", {"path": str(OUT / "dibujo.tdraw"), "x_mm": 50, "y_mm": 35}))
        assert len(imported["imported_ids"]) == 1
        assert count(json.dumps(imported["document"])) == 2
        assert count(call("undo")) == 1
        assert count(call("redo")) == 2
        assert count(call("undo")) == 1
    bad = OUT / "invalido.tdraw"
    bad.write_bytes(b"{")
    call("open", {"path": str(bad)}, expect_error=True)
    assert count(call("document_info")) == 1
    report = {"zip_sha256": state["zip_sha256"], "extracted_files_match": True,
              "runtime_abi_verified_after_zip_extraction": True,
              "negative_without_engine_verified": True,
              "headless_diagnostic_errors": diagnostic_errors,
              "extracted_directory": str(folder), "profile_directory": str(profile),
              "tool_count": len(tools), "expected_tools": args.expected_tools,
              "server": initialized["serverInfo"], "checks": checks,
              "cdr_read_from_previously_unopened_path": True,
              "original_coreldraw_executed": False, "visible_gui_tested": False,
              "full_equivalence_verified": False}
    (OUT / "prueba-real.json").write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    (EVIDENCE / "prueba-real.json").write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(json.dumps({"zip_integrity": True, "real_checks_passed": len(checks), "visible_gui_tested": False}, ensure_ascii=False))
finally:
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
