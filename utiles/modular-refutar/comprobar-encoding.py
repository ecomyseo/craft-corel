"""Ejecuta los 19 casos no ASCII sobre encoding_rs del paquete nuevo."""
from pathlib import Path
import argparse
import json
import os
import shutil
import subprocess
import time
import pefile
from runtime import ROOT, FLAGS, sha

parser = argparse.ArgumentParser()
parser.add_argument("--state-file", type=Path, required=True)
parser.add_argument("--manifest", type=Path, required=True)
args = parser.parse_args()
state_path = args.state_file.resolve(strict=True)
manifest_path = args.manifest.resolve(strict=True)
state = json.loads(state_path.read_text(encoding="utf-8"))
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
assert state["build_success"] and state["core_opt_level"] == 3
assert Path(manifest["build_state_file"]).resolve() == state_path
assert manifest["build_state_sha256"] == sha(state_path)
assert manifest["core_opt_level"] == 3
assert sha(manifest["zip"]) == manifest["zip_sha256"]
package = Path(manifest["directory"]).resolve(strict=True)
assert package.is_relative_to(ROOT)
for row in manifest["files"]:
    assert sha(package / row["path"]) == row["sha256"], row["path"]
assert (package / "craft-corel.exe").stat().st_size < 2_000_000
assert sha(package / "craft-corel-engine.dll") != "4010ba1c00244197e9c5ea85081fabc138ef38b3d857c9edec11b7bf7dcd71ec"
deps = Path(state["target"]) / "x86_64-pc-windows-msvc/debug/deps"
encoding = deps / "encoding_rs.dll"
assert sha(encoding) == sha(package / "encoding_rs.dll"), "La DLL enlazada debe ser la que se entrega."
detector = max(deps.glob("libchardetng-*.rlib"), key=lambda path: path.stat().st_mtime_ns)
out = ROOT / "evidencias/craft-plumilla/optimizacion/refutacion/encoding" / str(time.time_ns())
out.mkdir(parents=True)
runtime = out / "runtime"
runtime.mkdir()
for path in package.glob("*.dll"):
    shutil.copy2(path, runtime / path.name)
toolchain = ROOT / ".tools/rustup/toolchains/1.97.0-x86_64-pc-windows-msvc"
standard = toolchain / "lib/rustlib/x86_64-pc-windows-msvc/lib"
exe = runtime / "probe-encoding.exe"
source = Path(__file__).with_name("verify-encoding.rs")
command = [str(toolchain / "bin/rustc.exe"), "--edition=2021", "--target", "x86_64-pc-windows-msvc", str(source),
    "-C", "prefer-dynamic", "-L", "dependency=" + str(deps),
    "-L", "dependency=" + str(Path(state["target"]) / "debug/deps"),
    "--extern", "encoding_rs=" + str(encoding), "--extern", "chardetng=" + str(detector), "-o", str(exe)]
environment = dict(os.environ)
environment["PATH"] = os.pathsep.join([str(runtime), str(standard), str(Path(os.environ["SystemRoot"]) / "System32")])
compile_environment = dict(os.environ)
compile_environment["PATH"] = os.pathsep.join([str(deps), str(standard), str(Path("C:/Program/VC/Redist/MSVC/14.44.35112/x64/Microsoft.VC143.CRT")), compile_environment.get("PATH", "")])
compiled = subprocess.run(command, env=compile_environment, creationflags=FLAGS, stdin=subprocess.DEVNULL,
    capture_output=True, text=True, encoding="utf-8", timeout=60)
(out / "compile.log").write_text(compiled.stdout + compiled.stderr, encoding="utf-8")
assert compiled.returncode == 0, compiled.stderr
with pefile.PE(str(exe)) as pe:
    imports = [entry.dll.decode("ascii") for entry in pe.DIRECTORY_ENTRY_IMPORT]
assert "encoding_rs.dll" in imports
result = subprocess.run([str(exe)], cwd=runtime, env=environment, creationflags=FLAGS,
    stdin=subprocess.DEVNULL, capture_output=True, text=True, encoding="utf-8", timeout=30)
(out / "stdout.log").write_text(result.stdout, encoding="utf-8")
(out / "stderr.log").write_text(result.stderr, encoding="utf-8")
assert result.returncode == 0, result.stderr
assert len([line for line in result.stdout.splitlines() if line.startswith("non-ASCII input ")]) == 19
assert "19 accessors preserve identity, ASCII and non-ASCII round trips and detection" in result.stdout
report = {"passed": 19, "state_file": str(state_path), "state_sha256": sha(state_path),
    "package_manifest": str(manifest_path), "package_engine_sha256": sha(package / "craft-corel-engine.dll"),
    "package_encoding_dll_sha256": sha(package / "encoding_rs.dll"), "executed_encoding_dll_sha256": sha(runtime / "encoding_rs.dll"),
    "detector_rlib": str(detector), "detector_sha256": sha(detector), "probe_source_sha256": sha(source),
    "probe_exe_sha256": sha(exe), "probe_imports": imports,
    "scope": "Actual shipped encoding DLL with identity, non-ASCII encode/decode and detector checks; no desktop or clipboard."}
(out / "results.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
print(json.dumps({"passed": 19, "evidence": str(out)}))
