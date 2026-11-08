"""Comparte dependencias duplicadas en la copia aislada para el enlace modular."""
from pathlib import Path
import json
import re
import shutil
import tomllib
import argparse

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--names", nargs="+")
parser.add_argument("--fix-encoding-statics", action="store_true")
parser.add_argument("--state-file", type=Path, default=root / ".build/modular/actual.json")
args = parser.parse_args()
state_file = args.state_file.resolve()
assert state_file.is_relative_to((root / ".build/modular").resolve())
state = json.loads(state_file.read_text(encoding="utf-8"))
source = Path(state["source"])
registry = next((root / ".tools/cargo/registry/src").iterdir())
if args.fix_encoding_statics:
    # MSVC cannot resolve direct imported-data addresses in chardetng's
    # static table. Keep the same Encoding instances behind non-inline
    # accessors owned by encoding_rs, without changing detector scores.
    encoding = next(row for row in state["shared_dependencies"] if row["name"] == "encoding_rs")
    encoding_lib = Path(encoding["path"]) / "src/lib.rs"
    dest = source / "vendor/modular-chardetng-0.1.17"
    if not dest.exists():
        shutil.copytree(registry / "chardetng-0.1.17", dest)
    data = dest / "src/data.rs"
    contents = data.read_text(encoding="utf-8")
    marker = "// Craft private DLL data import accessors."
    if marker not in encoding_lib.read_text(encoding="utf-8"):
        names = sorted(set(re.findall(r"use encoding_rs::([A-Z_0-9]+_INIT);", contents)))
        assert len(names) == 19, names
        text = encoding_lib.read_text(encoding="utf-8") + "\n" + marker + "\n"
        for name in names:
            getter = "__craft_get_" + name.lower()
            text += '#[doc(hidden)]\n#[inline(never)]\npub fn ' + getter + "() -> &'static Encoding { &" + name + " }\n"
            contents = contents.replace("use encoding_rs::" + name + ";", "use encoding_rs::" + getter + ";")
            contents = contents.replace("encoding: &" + name + ",", "encoding: " + getter + ",")
        assert contents.count("pub encoding: &'static Encoding,") == 1
        contents = contents.replace("pub encoding: &'static Encoding,", "pub encoding: fn() -> &'static Encoding,")
        lib = dest / "src/lib.rs"
        old = lib.read_text(encoding="utf-8")
        assert old.count("return c.data.encoding;") == 6
        lib.write_text(old.replace("return c.data.encoding;", "return (c.data.encoding)();"), encoding="utf-8")
        data.write_text(contents, encoding="utf-8")
        encoding_lib.write_text(text, encoding="utf-8")
    manifest = source / "Cargo.toml"
    text = manifest.read_text(encoding="utf-8")
    alias = 'chardetng-modular-0-1-17'
    if alias not in text:
        text += '\n"' + alias + '" = { package = "chardetng", path = "vendor/' + dest.name + '" }\n'
        manifest.write_text(text, encoding="utf-8")
    state["encoding_data_import_fix"] = {
        "dependency": "chardetng", "version": "0.1.17", "path": str(dest),
        "static_getters": 19, "detector_algorithm_changed": False,
        "license": "Apache-2.0 OR MIT", "normal_source_changed": False,
    }
    state["build_success"] = False
    state_file.write_text(json.dumps(state, indent=2), encoding="utf-8")
    print(json.dumps(state["encoding_data_import_fix"]))
    raise SystemExit(0)
log = (root / ".tools/build-modular.log").read_text(encoding="utf-8", errors="replace")
names = set(args.names or re.findall(r"cannot satisfy dependencies so `([^`]+)` only shows up once", log))
assert names, "No se han identificado dependencias duplicadas."
lock = tomllib.loads((source / "Cargo.lock").read_text(encoding="utf-8"))
manifest = source / "Cargo.toml"
text = manifest.read_text(encoding="utf-8")
assert text.rfind("[patch.crates-io]") > text.rfind("[profile.")
patched = state.setdefault("shared_dependencies", [])
for package in lock["package"]:
    name = package["name"]
    if name.replace("-", "_") not in names or "registry+" not in package.get("source", ""):
        continue
    version = package["version"]
    cached = registry / (name + "-" + version)
    if not cached.is_dir():
        continue
    dest = source / "vendor" / ("modular-" + name + "-" + version)
    assert not any(row["name"] == name and row["version"] == version for row in patched), "La dependencia ya estaba compartida: " + name
    if not dest.exists():
        shutil.copytree(cached, dest)
    cargo = dest / "Cargo.toml"
    contents = cargo.read_text(encoding="utf-8")
    assert "[lib]" in contents
    if "crate-type" not in contents:
        contents = contents.replace("[lib]", '[lib]\ncrate-type = ["rlib", "dylib"]', 1)
    cargo.write_text(contents, encoding="utf-8")
    alias = name + "-modular-" + version.replace(".", "-")
    text += '\n"' + alias + '" = { package = "' + name + '", path = "vendor/' + dest.name + '" }\n'
    patched.append({"name": name, "version": version, "path": str(dest)})
manifest.write_text(text, encoding="utf-8")
state["build_success"] = False
state_file.write_text(json.dumps(state, indent=2), encoding="utf-8")
print(json.dumps(patched))
