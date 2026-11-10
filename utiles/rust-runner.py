"""Ejecuta Rust sin consola visible y con prioridad reducida, dentro del proyecto."""
from pathlib import Path
import argparse
import ctypes
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--log", required=True)
parser.add_argument("--jobs", default="2")
parser.add_argument("arguments", nargs=argparse.REMAINDER)
args = parser.parse_args()
log = (ROOT / ".tools" / args.log).resolve()
if not log.is_relative_to((ROOT / ".tools").resolve()):
    raise SystemExit("El registro debe quedar dentro de .tools.")
env = dict(os.environ, CARGO_HOME=str(ROOT / ".tools/cargo"), RUSTUP_HOME=str(ROOT / ".tools/rustup"),
           CARGO_TARGET_DIR=str(ROOT / ".build/craft-plumilla"), CARGO_PROFILE_DEV_DEBUG="0",
           CARGO_PROFILE_DEV_OPT_LEVEL="0", CARGO_BUILD_JOBS=args.jobs)
env["PATH"] = str(ROOT / ".tools/cargo/bin") + os.pathsep + env.get("PATH", "")
command = [str(ROOT / ".tools/cargo/bin/cargo.exe"), "+1.97.0", *args.arguments]
flags = subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS
# Only this runner and its children: a failed hostile-input test must report in
# the log rather than displaying a Windows crash or critical-error dialog.
kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
kernel32.SetErrorMode.argtypes = [ctypes.c_uint]
kernel32.SetErrorMode.restype = ctypes.c_uint
kernel32.SetErrorMode(0x0001 | 0x0002 | 0x8000)
with log.open("wb") as stream:
    process = subprocess.Popen(command, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                               stdout=stream, stderr=subprocess.STDOUT, creationflags=flags)
    result = process.wait()
lines = log.read_text(encoding="utf-8", errors="replace").splitlines()
print("\n".join(lines[-35:]))
raise SystemExit(result)
