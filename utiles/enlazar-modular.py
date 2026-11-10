"""Ordena el enlace privado de las DLL en una copia de empaquetado aislada."""
from pathlib import Path
import json
import shutil
import tomllib
import argparse

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--state-file", type=Path, default=root / ".build/modular/actual.json")
args = parser.parse_args()
state_file = args.state_file.resolve()
assert state_file.is_relative_to((root / ".build/modular").resolve())
state = json.loads(state_file.read_text(encoding="utf-8"))
assert not state.get("build_success")
source = Path(state["source"])
manifest = source / "Cargo.toml"
text = manifest.read_text(encoding="utf-8")
text = "\n".join(line for line in text.splitlines() if not ('-modular-' in line and 'path = "vendor/modular-' in line)) + "\n"
manifest.write_text(text, encoding="utf-8")
state["unused_external_experiment"] = state.pop("shared_dependencies", [])
state["shared_dependencies"] = []
order = ["core", "cdr-format", "render", "text", "tracecut", "cdr", "io", "vba-project", "vba"]
for previous, current in zip(order, order[1:]):
    cargo = source / "crates" / current / "Cargo.toml"
    text = cargo.read_text(encoding="utf-8")
    package = "tracedraw-" + previous
    dependencies = tomllib.loads(text)["dependencies"]
    if package not in dependencies:
        text = text.replace("[dependencies]", "[dependencies]\n" + package + ".workspace = true", 1)
        cargo.write_text(text, encoding="utf-8")
    lib = source / "crates" / current / "src/lib.rs"
    text = lib.read_text(encoding="utf-8")
    assert "__craft_runtime_predecessor" not in text
    text += '\n// Private packaging link order; the normal source tree is unchanged.\n#[doc(hidden)]\npub use ' + package.replace("-", "_") + ' as __craft_runtime_predecessor;\n'
    lib.write_text(text, encoding="utf-8")
state["private_link_order"] = order
state["source_layering_changed_only_in_packaging_snapshot"] = True
state["build_success"] = False
state_file.write_text(json.dumps(state, indent=2), encoding="utf-8")
shutil.copy2(root / "craft-plumilla/Cargo.lock", source / "Cargo.lock")
if state["unused_external_experiment"]:
    saved_log = source / "discarded-external-experiment.log"
    if not saved_log.exists():
        shutil.copy2(root / ".tools/build-modular.log", saved_log)
print(json.dumps({"link_order": order, "active_source_changed": False}))
