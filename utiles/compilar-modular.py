"""Prepara y compila una prueba modular sin modificar la portable entregada."""
from pathlib import Path
from datetime import datetime
import argparse
import json
import os
import shutil
import subprocess
import sys
import hashlib
import tomllib

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--prepare-only", action="store_true")
parser.add_argument("--resume", action="store_true")
parser.add_argument("--state-file", type=Path, default=ROOT / ".build/modular/actual.json")
parser.add_argument("--source-path", type=Path)
parser.add_argument("--compare-with-state", type=Path)
parser.add_argument("--core-opt-level", type=int, choices=(0, 3))
parser.add_argument("--log", default="build-modular.log")
args = parser.parse_args()
work = ROOT / ".build/modular"
state_file = args.state_file.resolve()
assert state_file.is_relative_to(work.resolve())
if args.resume:
    state = json.loads(state_file.read_text(encoding="utf-8"))
    source = Path(state["source"])
    if args.core_opt_level is not None:
        assert args.core_opt_level == state.get("core_opt_level", 0), "Prepare a new state for a different compiler profile."
else:
    source = (args.source_path or work / ("fuente-" + datetime.now().strftime("%Y%m%d-%H%M%S"))).resolve()
    assert source.is_relative_to(work.resolve())
    source.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(ROOT / "craft-corel-nativo", source,
                    ignore=shutil.ignore_patterns(".git", "target", "node_modules", "__pycache__", ".DS_Store"))
    modules = ["core", "cdr-format", "cdr", "render", "text", "io", "vba", "vba-project", "tracecut"]
    for name in modules:
        path = source / "crates" / name / "Cargo.toml"
        text = path.read_text(encoding="utf-8")
        assert "[lib]" not in text, name
        text += '\n[lib]\ncrate-type = ["dylib"]\n'
        path.write_text(text, encoding="utf-8")
    app = source / "apps/tracedraw/Cargo.toml"
    text = app.read_text(encoding="utf-8")
    assert 'crate-type = ["rlib", "cdylib"]' in text
    app.write_text(text.replace('crate-type = ["rlib", "cdylib"]', 'crate-type = ["cdylib"]'), encoding="utf-8")
    state = {"source": str(source), "target": str(work / "target"), "modules": modules,
             "rust": "1.97.0", "private_rust_abi": True, "published_portable_changed": False}
    state["core_opt_level"] = args.core_opt_level if args.core_opt_level is not None else 0
    state_file.parent.mkdir(parents=True, exist_ok=True)
    state_file.write_text(json.dumps(state, indent=2), encoding="utf-8")
    # These steps are part of the default recipe, not manual edits of
    # a successful snapshot. Discarded sharing experiments stay inactive.
    commands = [
        [sys.executable, str(ROOT / "utiles/enlazar-modular.py"), "--state-file", str(state_file)],
        [sys.executable, str(ROOT / "utiles/ajustar-modular.py"), "--state-file", str(state_file), "--names", "encoding_rs"],
    ]
    for command in commands:
        subprocess.run(command, check=True, cwd=ROOT)
    state = json.loads(state_file.read_text(encoding="utf-8"))
    encoding = Path(next(row["path"] for row in state["shared_dependencies"] if row["name"] == "encoding_rs"))
    manifest = encoding / "Cargo.toml"
    text = manifest.read_text(encoding="utf-8")
    assert 'crate-type = ["rlib", "dylib"]' in text
    manifest.write_text(text.replace('crate-type = ["rlib", "dylib"]', 'crate-type = ["dylib"]'), encoding="utf-8")
    lib = encoding / "src/lib.rs"
    text = lib.read_text(encoding="utf-8")
    text += '\n// This private Windows DLL uses the shared Rust runtime.\nextern crate std;\n'
    lib.write_text(text, encoding="utf-8")
    core_manifest = source / "crates/core/Cargo.toml"
    text = core_manifest.read_text(encoding="utf-8")
    if "encoding_rs" not in tomllib.loads(text)["dependencies"]:
        text = text.replace("[dependencies]", "[dependencies]\nencoding_rs.workspace = true", 1)
        core_manifest.write_text(text, encoding="utf-8")
    lib = source / "crates/core/src/lib.rs"
    text = lib.read_text(encoding="utf-8")
    text += '\n// Private runtime foundation for the Windows module set.\n#[doc(hidden)]\npub use encoding_rs as __craft_runtime_encoding;\n'
    lib.write_text(text, encoding="utf-8")
    command = [sys.executable, str(ROOT / "utiles/ajustar-modular.py"), "--state-file", str(state_file), "--fix-encoding-statics"]
    subprocess.run(command, check=True, cwd=ROOT)
    commands.append(command)
    state = json.loads(state_file.read_text(encoding="utf-8"))
    state["private_link_order"].insert(0, "encoding_rs")
    state["preparation_recipe_version"] = 1
    state["preparation_commands"] = commands
    state["normal_source_changed"] = False
    state["wasm_source_changed"] = False
    state["discarded_experiments_active"] = False
    if args.compare_with_state:
        reference = json.loads(args.compare_with_state.read_text(encoding="utf-8"))
        previous = Path(reference["source"])
        paths = [Path("Cargo.toml"), Path("apps/tracedraw/Cargo.toml")]
        for name in modules:
            paths.extend([Path("crates") / name / "Cargo.toml", Path("crates") / name / "src/lib.rs"])
        paths.extend(Path("vendor") / name / suffix for name, suffix in [
            ("modular-encoding_rs-0.8.42", "Cargo.toml"),
            ("modular-encoding_rs-0.8.42", "src/lib.rs"),
            ("modular-chardetng-0.1.17", "Cargo.toml"),
            ("modular-chardetng-0.1.17", "src/lib.rs"),
            ("modular-chardetng-0.1.17", "src/data.rs"),
        ])
        compared = []
        for relative in paths:
            old, new = previous / relative, source / relative
            a, b = old.read_text(encoding="utf-8"), new.read_text(encoding="utf-8")
            same = tomllib.loads(a) == tomllib.loads(b) if relative.suffix == ".toml" else a.rstrip() == b.rstrip()
            compared.append({"path": relative.as_posix(), "equivalent": same,
                             "comparison": "parsed TOML" if relative.suffix == ".toml" else "source text, normalised line endings/trailing space",
                             "reference_sha256": hashlib.sha256(old.read_bytes()).hexdigest(),
                             "prepared_sha256": hashlib.sha256(new.read_bytes()).hexdigest()})
        state["preparation_comparison"] = {"reference_state": str(args.compare_with_state.resolve()), "files": compared,
                                            "all_equivalent": all(row["equivalent"] for row in compared)}
    state_file.write_text(json.dumps(state, indent=2), encoding="utf-8")
print(json.dumps(state), flush=True)
if args.prepare_only:
    raise SystemExit(0 if state.get("preparation_comparison", {}).get("all_equivalent", True) else 1)
environment = dict(os.environ, CARGO_HOME=str(ROOT / ".tools/cargo"), RUSTUP_HOME=str(ROOT / ".tools/rustup"),
                   CARGO_TARGET_DIR=state["target"], CARGO_BUILD_JOBS="2", CARGO_INCREMENTAL="0",
                   CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_DEV_OPT_LEVEL="0", RUSTFLAGS="-C prefer-dynamic")
environment["PATH"] = str(ROOT / ".tools/cargo/bin") + os.pathsep + environment.get("PATH", "")
command = [str(ROOT / ".tools/cargo/bin/cargo.exe"), "+1.97.0", "build", "--offline", "--manifest-path", str(source / "Cargo.toml"),
           "-p", "craft-corel-nativo", "--lib", "--target", "x86_64-pc-windows-msvc"]
if state.get("core_opt_level", 0) == 3:
    command += ["--config", 'profile.dev.package.tracedraw-core.opt-level=3', "--verbose"]
log = (ROOT / ".tools" / args.log).resolve()
assert log.is_relative_to((ROOT / ".tools").resolve())
state["build_command"] = command
state["build_log"] = str(log)
state_file.write_text(json.dumps(state, indent=2), encoding="utf-8")
with log.open("wb") as output:
    result = subprocess.run(command, env=environment, cwd=source, stdin=subprocess.DEVNULL,
                            stdout=output, stderr=subprocess.STDOUT,
                            creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
print("\n".join(log.read_text(encoding="utf-8", errors="replace").splitlines()[-35:]))
state["build_success"] = result.returncode == 0
state["build_exit_code"] = result.returncode
state_file.write_text(json.dumps(state, indent=2), encoding="utf-8")
raise SystemExit(result.returncode)
