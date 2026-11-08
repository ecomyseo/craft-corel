"""Empaqueta DLL reales siguiendo sus importaciones PE, con un lanzador nativo."""
from pathlib import Path
from datetime import datetime
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tomllib
import zipfile
import pefile

ROOT = Path(__file__).resolve().parents[1]


def collect_cargo_notices(source, supplement_file, scope_file):
    """Inventaría todos los paquetes del lock; incluye otros destinos y pruebas."""
    lock_file = source / "Cargo.lock"
    packages = tomllib.loads(lock_file.read_text(encoding="utf-8"))["package"]
    scope = json.loads(scope_file.read_text(encoding="utf-8")) if scope_file else None
    excluded = []
    if scope:
        included = set(scope["included_packages"])
        known = {package["name"] + "@" + package["version"] for package in packages}
        assert included <= known, "El grafo contiene paquetes ajenos al Cargo.lock."
        excluded = [package for package in packages if package["name"] + "@" + package["version"] not in included]
        packages = [package for package in packages if package["name"] + "@" + package["version"] in included]
    workspace_manifest = tomllib.loads((source / "Cargo.toml").read_text(encoding="utf-8"))
    workspace_defaults = workspace_manifest.get("workspace", {}).get("package", {})
    supplemental = json.loads(supplement_file.read_text(encoding="utf-8")) if supplement_file else {}
    supplemental = supplemental.get("packages", {})
    workspace = {}
    for manifest_file in source.rglob("Cargo.toml"):
        if any(part in {"target", ".git", "node_modules"} for part in manifest_file.relative_to(source).parts):
            continue
        try:
            manifest = tomllib.loads(manifest_file.read_text(encoding="utf-8"))
        except (ValueError, UnicodeError):
            continue
        package = manifest.get("package", {})
        if not package.get("name"):
            continue
        version = package.get("version")
        if isinstance(version, dict):
            version = workspace_defaults.get("version")
        workspace[(package["name"], version)] = manifest_file.parent
    rows, failures = [], []
    for locked in packages:
        name, version = locked["name"], locked["version"]
        key = name + "@" + version
        extra = supplemental.get(key, {})
        registry = locked.get("source", "").startswith("registry+")
        candidates = list((ROOT / ".tools/cargo/registry/src").glob("*/" + name + "-" + version)) if registry else []
        directory = Path(extra["source_directory"]) if extra.get("source_directory") else (candidates[0] if candidates else workspace.get((name, version)))
        if directory is None or not (directory / "Cargo.toml").is_file():
            failures.append({"package": key, "reason": "source_missing"})
            continue
        directory = directory.resolve()
        assert directory.is_relative_to(ROOT), "La fuente de licencias debe estar dentro del proyecto."
        manifest_file = directory / "Cargo.toml"
        package = tomllib.loads(manifest_file.read_text(encoding="utf-8"))["package"]
        license_expression = package.get("license", "")
        if isinstance(license_expression, dict):
            license_expression = workspace_defaults.get("license", "")
        authors = package.get("authors", [])
        if isinstance(authors, dict):
            authors = workspace_defaults.get("authors", [])
        files = {}
        main_legal = False
        legal_tokens = ("LICENSE", "LICENCE", "COPYRIGHT", "NOTICE", "COPYING", "UNLICENSE", "AUTHORS", "PATENTS")
        text_suffixes = {".rs", ".c", ".h", ".cpp", ".cc", ".md", ".txt", ".toml", ".wgsl", ".glsl", ".xml", ".json", ".py", ".sh"}
        for path in directory.rglob("*"):
            if not path.is_file() or path.is_symlink():
                continue
            relative = path.relative_to(directory)
            if any(part in {".git", "target", "node_modules"} for part in relative.parts):
                continue
            upper = path.name.upper()
            legal = any(token in upper for token in legal_tokens) or upper in {"OFL.TXT", "UFL.TXT"}
            readme = upper.startswith(("README", "ABOUT")) or upper in {"CARGO.TOML", "CARGO.TOML.ORIG", ".CARGO_VCS_INFO.JSON"}
            copyright_header = False
            if not legal and not readme and path.suffix.lower() in text_suffixes:
                with path.open("rb") as stream:
                    header = stream.read(32768).lower()
                copyright_header = b"copyright" in header or b"spdx-filecopyrighttext" in header
            if legal or readme or copyright_header:
                files[relative.as_posix()] = {"source": str(path), "reason": "legal_text" if legal else ("package_metadata" if readme else "copyright_source")}
            if legal and len(relative.parts) <= 2 and "font" not in relative.as_posix().lower() and any(token in upper for token in ("LICENSE", "LICENCE", "COPYING", "UNLICENSE")):
                main_legal = True
        declared = package.get("license-file")
        if isinstance(declared, dict):
            declared = workspace_defaults.get("license-file")
        if declared:
            path = (directory / declared).resolve()
            assert path.is_relative_to(ROOT) and path.is_file(), "No existe el license-file declarado."
            files["declared-license/" + path.name] = {"source": str(path), "reason": "declared_license_file"}
            main_legal = True
        if not registry and directory.is_relative_to(source) and "vendor" not in directory.relative_to(source).parts:
            for path in source.iterdir():
                if path.is_file() and path.name.upper().startswith(("LICENSE", "LICENCE", "NOTICE", "COPYING", "COPYRIGHT")):
                    files["workspace-license/" + path.name] = {"source": str(path), "reason": "workspace_license"}
                    main_legal = True
        registry_files_verified = False
        if registry and (directory / ".cargo-checksum.json").is_file():
            checksum = json.loads((directory / ".cargo-checksum.json").read_text(encoding="utf-8"))
            assert checksum.get("package") == locked.get("checksum"), "La fuente registry no coincide con Cargo.lock."
            for item in files.values():
                path = Path(item["source"])
                if path.is_relative_to(directory):
                    expected = checksum["files"].get(path.relative_to(directory).as_posix())
                    assert expected and hashlib.sha256(path.read_bytes()).hexdigest() == expected, "Cambió un aviso o metadato del paquete registry."
            registry_files_verified = True
        for item in extra.get("files", []):
            path = Path(item["source"]).resolve()
            assert path.is_relative_to(ROOT) and path.is_file()
            assert hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"], "Cambió un aviso legal suplementario."
            files["upstream/" + item["path"]] = {"source": str(path), "reason": "upstream_legal_text", "origin": item.get("origin")}
            main_legal = True
        if not license_expression and not declared:
            failures.append({"package": key, "reason": "license_declaration_missing"})
        if not main_legal:
            failures.append({"package": key, "reason": "main_license_text_missing", "expression": license_expression})
        rows.append({"name": name, "version": version, "source": locked.get("source", "workspace_or_vendor"),
                     "registry_checksum": locked.get("checksum"), "license": license_expression, "authors": authors,
                     "registry_notice_files_verified_against_cargo_checksum": registry_files_verified,
                     "repository": package.get("repository"), "files": files, "supplement_provenance": extra.get("provenance")})
    if failures:
        failure_file = ROOT / "evidencias/craft-corel-nativo/optimizacion/empaquetado/licencias-pendientes.json"
        failure_file.write_text(json.dumps({"packages_expected": len(packages), "failures": failures}, indent=2, ensure_ascii=False), encoding="utf-8")
        raise SystemExit("Faltan fuentes o textos legales; consulta " + str(failure_file))
    return {"scope": scope["scope"] if scope else "all_Cargo.lock_packages_including_other_targets_and_dev_dependencies",
            "graph": scope, "excluded_packages": excluded,
            "cargo_lock_sha256": hashlib.sha256(lock_file.read_bytes()).hexdigest(), "package_count": len(packages), "packages": rows}


parser = argparse.ArgumentParser()
parser.add_argument("--state-file", type=Path, default=ROOT / ".build/modular/actual.json")
parser.add_argument("--update-current", action="store_true", help="Actualiza el índice del paquete recomendado solo si se solicita expresamente.")
parser.add_argument("--cargo-license-supplement", type=Path)
parser.add_argument("--cargo-license-scope", type=Path)
parser.add_argument("--launcher-manifest", type=Path)
parser.add_argument("--manuals-dir", type=Path, default=ROOT / "craft-corel-nativo/docs/manual-craft-corel")
args = parser.parse_args()
manuals_source = args.manuals_dir.resolve(strict=True)
assert manuals_source.is_relative_to(ROOT.resolve()), "Los manuales deben estar dentro del espacio de trabajo."
manual_names = ["manual-usuario.html", "manual-usuario.pdf", "guia-tecnica.html", "guia-tecnica.pdf", "README.md", "LICENCIA-TIPOGRAFIA.txt"]
for manual_name in manual_names:
    manual_path = (manuals_source / manual_name).resolve(strict=True)
    assert manual_path.is_relative_to(manuals_source) and manual_path.is_file(), "Falta un manual o está fuera de su carpeta."
state_file = args.state_file.resolve(strict=True)
assert state_file.is_relative_to((ROOT / ".build/modular").resolve())
state = json.loads(state_file.read_text(encoding="utf-8"))
state_sha256 = hashlib.sha256(state_file.read_bytes()).hexdigest()
SOURCE = Path(state["source"])
BUILD = Path(state["target"]) / "x86_64-pc-windows-msvc/debug"
engine = BUILD / "tracedraw.dll"
assert state.get("build_success"), "La compilación modular aún no está verificada."
assert engine.is_file()
cargo_notices = collect_cargo_notices(SOURCE, args.cargo_license_supplement, args.cargo_license_scope)
toolchain = ROOT / ".tools/rustup/toolchains/1.97.0-x86_64-pc-windows-msvc"
rust_lib = toolchain / "lib/rustlib/x86_64-pc-windows-msvc/lib"
crt = Path("C:/Program/VC/Redist/MSVC/14.44.35112/x64/Microsoft.VC143.CRT")
search = [BUILD, BUILD / "deps", rust_lib, crt]
version = "modular-prueba-" + datetime.now().strftime("%Y%m%d-%H%M%S")
DEST = ROOT / "runtime" / ("craft-corel-" + version)
DEST.mkdir(exist_ok=False)
graph = []
pending = [(engine, "craft-corel-engine.dll")]
seen = set()
while pending:
    path, name = pending.pop()
    if name.lower() in seen:
        continue
    seen.add(name.lower())
    shutil.copy2(path, DEST / name)
    with pefile.PE(str(path), fast_load=True) as pe:
        assert pe.FILE_HEADER.Machine == 0x8664, path
        pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_IMPORT"]])
        imports = [item.dll.decode("ascii") for item in getattr(pe,"DIRECTORY_ENTRY_IMPORT",[])]
    record = {"file":name,"bytes":path.stat().st_size,"sha256":hashlib.sha256(path.read_bytes()).hexdigest(),"imports":imports}
    graph.append(record)
    for dependency in imports:
        local = next((folder / dependency for folder in search if (folder / dependency).is_file()),None)
        if local:
            pending.append((local,dependency))
        else:
            known_system = dependency.lower().startswith(("api-ms-win-","ext-ms-win-")) or (Path(os.environ["SystemRoot"]) / "System32" / dependency).is_file()
            assert known_system, "Falta una dependencia: " + dependency
            assert not dependency.lower().startswith(("tracedraw","std-","craft_corel")), dependency
expected_modules = []
for module in state.get("modules", []):
    manifest = tomllib.loads((SOURCE / "crates" / module / "Cargo.toml").read_text(encoding="utf-8"))
    library = manifest.get("lib", {}).get("name", manifest["package"]["name"].replace("-", "_"))
    expected_modules.append(library + ".dll")
assert expected_modules, "El estado no declara los módulos que deben acompañar al motor."
assert all(name.lower() in seen for name in expected_modules), "Faltan módulos declarados en la cadena PE."
assert len([name for name in seen if name.startswith("tracedraw_")]) >= len(expected_modules), "El motor no está dividido en los módulos declarados."
environment = dict(os.environ,CARGO_HOME=str(ROOT / ".tools/cargo"),RUSTUP_HOME=str(ROOT / ".tools/rustup"))
launcher_dir = ROOT / ".build/modular/launcher" / version
launcher_dir.mkdir(parents=True)
resources = list((BUILD / "build").glob("craft-corel-nativo-*/out/resource.res"))
assert resources
resource = max(resources,key=lambda p:p.stat().st_mtime_ns)
if args.launcher_manifest:
    prior = json.loads(args.launcher_manifest.read_text(encoding="utf-8"))
    known_launcher = Path(prior["directory"]) / "craft-corel.exe"
    expected_hash = prior.get("launcher_sha256") or next(row["sha256"] for row in prior["files"] if row["path"] == "craft-corel.exe")
    assert hashlib.sha256(known_launcher.read_bytes()).hexdigest() == expected_hash
    assert prior.get("runtime_abi_verified") is True
    assert prior.get("launcher_source_sha256") == hashlib.sha256((ROOT / "utiles/portable-launcher.rs").read_bytes()).hexdigest()
    shutil.copy2(known_launcher, launcher_dir / "craft-corel.exe")
else:
    subprocess.run([str(ROOT / ".tools/cargo/bin/rustc.exe"),"+1.97.0","--edition=2021","--crate-name","craft_corel_launcher",
                    str(ROOT / "utiles/portable-launcher.rs"),"-C","opt-level=z","-C","panic=abort","-C","strip=symbols","-C","lto=thin",
                    "-C","link-arg="+str(resource),"-o",str(launcher_dir / "craft-corel.exe")],check=True,env=environment,
                   stdin=subprocess.DEVNULL,creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
shutil.copy2(launcher_dir / "craft-corel.exe",DEST / "craft-corel.exe")
for name in ["LICENSE-MIT","LICENSE-APACHE","ATTRIBUTION.md"]:
    shutil.copy2(SOURCE / name,DEST / name)
(DEST / "Licencias").mkdir()
shutil.copy2(SOURCE / "apps/tracedraw/assets/fonts/Selawik-OFL.txt",DEST / "Licencias/Selawik-OFL.txt")
shutil.copy2(toolchain / "share/doc/rust/COPYRIGHT-library.html",DEST / "Licencias/Rust-COPYRIGHT-library.html")
shutil.copytree(toolchain / "share/doc/rust/licenses",DEST / "Licencias/licenses")
cargo_directory = DEST / "Licencias/Cargo"
cargo_directory.mkdir()
for package_index, package in enumerate(cargo_notices["packages"]):
    # Keep legal paths short on Windows; the index retains every original path.
    package_directory = cargo_directory / f"p{package_index:04d}"
    package_directory.mkdir()
    package["license_directory"] = package_directory.relative_to(DEST).as_posix()
    copied = []
    for file_index, (relative, item) in enumerate(sorted(package["files"].items())):
        target = package_directory / (f"f{file_index:04d}-" + Path(relative).name[:20])
        assert target.resolve().is_relative_to(package_directory.resolve()), "Ruta legal fuera del paquete."
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(item["source"], target)
        copied.append({"path": target.relative_to(DEST).as_posix(), "source_relative_path": relative, "bytes": target.stat().st_size,
                       "sha256": hashlib.sha256(target.read_bytes()).hexdigest(), "reason": item["reason"], "origin": item.get("origin")})
    package["files"] = copied
(DEST / "Licencias/CARGO-NOTICES.json").write_text(json.dumps(cargo_notices, indent=2, ensure_ascii=False), encoding="utf-8")
(DEST / "Licencias/CARGO-ALCANCE.txt").write_text(
    "Este inventario cubre todos los paquetes del grafo de dependencias declarado en\n"
    "CARGO-NOTICES.json, incluidos los componentes de compilación señalados en su alcance.\n"
    "Los paquetes de Cargo.lock excluidos se identifican por no pertenecer a ese grafo.\n"
    "Conserva los textos legales y avisos originales encontrados,\n"
    "además de metadatos y archivos fuente con avisos de copyright. El inventario indica\n"
    "la expresión de licencia, procedencia y huellas; no sustituye las licencias.\n", encoding="utf-8")
dependencies = list(state.get("shared_dependencies", []))
if state.get("encoding_data_import_fix"):
    fix = state["encoding_data_import_fix"]
    dependencies.append({"name": fix["dependency"], "version": fix["version"], "path": fix["path"]})
for dependency in dependencies:
    original = Path(dependency["path"])
    license_dir = DEST / "Licencias" / (dependency["name"] + "-" + dependency["version"])
    license_dir.mkdir()
    for file in original.iterdir():
        if file.is_file() and file.name.upper().startswith(("LICENSE", "COPYING", "NOTICE")):
            shutil.copy2(file, license_dir / file.name)
    assert any(license_dir.iterdir()), "Falta la licencia de " + dependency["name"]
shutil.copy2(Path(json.loads((ROOT / "evidencias/portable/actual.json").read_text(encoding="utf-8"))["directory"]) / "Licencias/Microsoft-runtime.txt",DEST / "Licencias/Microsoft-runtime.txt")
(DEST / "Ejemplos").mkdir()
for suffix in ["svg","cdr"]:
    shutil.copy2(ROOT / ("ejemplos/craft-corel/Bienvenida." + suffix),DEST / ("Ejemplos/Bienvenida." + suffix))
(DEST / "craft-corel-settings").mkdir()
(DEST / "craft-corel-settings/settings.json").write_text(json.dumps({"language":"es","show_splash":False},indent=2),encoding="utf-8")
(DEST / "Manuales").mkdir()
for manual_name in manual_names:
    shutil.copy2(manuals_source / manual_name, DEST / "Manuales" / manual_name)
(DEST / "LEEME.txt").write_text(
    "CRAFT COREL: PRUEBA PORTABLE MODULAR\n\nExtrae todo el ZIP y abre craft-corel.exe.\n"
    "Mantén todas las DLL junto al EXE. Cada módulo contiene código real y se ha\n"
    "compilado con el mismo Rust 1.97.0; conserva siempre el conjunto completo\n"
    "de esta versión. Las preferencias quedan en craft-corel-settings.\n\n"
    "Los manuales de usuario y técnico están en Manuales, en HTML y PDF.\n\n"
    "Compilación de desarrollo para Windows de 64 bits, basada en TraceDraw.\n"
    "Todavía no reproduce todas las herramientas ni verifica compatibilidad\n"
    "completa con CorelDRAW. La interfaz visible no se ha probado en este escritorio.\n"
    "Guarda las pruebas con nombres nuevos y conserva tus archivos originales.\n\n"
    "No requiere Python, Rust ni CorelDRAW instalados. Se incluye el runtime de\n"
    "Microsoft y la biblioteca estándar de Rust con sus licencias. Los módulos\n"
    "internos usan una interfaz privada de esta compilación y no son complementos\n"
    "intercambiables entre versiones. La actualización original está desactivada.\n\n"
    "Esta prueba local todavía no tiene firma de editor y Windows puede mostrar\n"
    "avisos. No se ha desactivado ninguna protección. Los hashes comprueban\n"
    "integridad y no sustituyen una firma ni una comprobación antivirus.\n",encoding="utf-8")
files = [{"path":p.relative_to(DEST).as_posix(),"bytes":p.stat().st_size,"sha256":hashlib.sha256(p.read_bytes()).hexdigest()}
         for p in sorted(DEST.rglob("*")) if p.is_file()]
(DEST / "SHA256SUMS.txt").write_text("".join(row["sha256"]+"  "+row["path"]+"\n" for row in files),encoding="utf-8")
preflight_prefix = ROOT / ".build/portable-verificacion" / (version + "-" + "9" * 19)
max_relative_path = max(len((DEST.name + "/" + path.relative_to(DEST).as_posix()).encode("utf-16-le")) // 2 for path in DEST.rglob("*") if path.is_file())
max_workspace_extraction = max(len(str(preflight_prefix / DEST.name / path.relative_to(DEST)).encode("utf-16-le")) // 2 for path in DEST.rglob("*") if path.is_file())
assert max_workspace_extraction < 250, "Las rutas del ZIP superan el margen de extracción Windows verificado."
assert (DEST / "craft-corel.exe").stat().st_size < 2 * 1024 * 1024, "El EXE no es el lanzador DLL esperado."
verified = subprocess.run([str(DEST / "craft-corel.exe"),"--verify-runtime"],check=True,capture_output=True,timeout=30,
                          text=True,encoding="utf-8",
                          creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
assert "ABI 1" in verified.stdout, "El lanzador no confirmó la ABI del motor."
archive = ROOT / "descargas" / (DEST.name + ".zip")
with zipfile.ZipFile(archive,"x",compression=zipfile.ZIP_DEFLATED,compresslevel=6,strict_timestamps=False) as package:
    for p in sorted(DEST.rglob("*")):
        if p.is_file(): package.write(p,DEST.name+"/"+p.relative_to(DEST).as_posix())
report = {"version":version,"directory":str(DEST),"zip":str(archive),"zip_bytes":archive.stat().st_size,
          "zip_sha256":hashlib.sha256(archive.read_bytes()).hexdigest(),"files":files,"dlls":graph,
          "runtime_abi_verified":True,"preview":True,"full_equivalence_verified":False,"private_rust_abi":True}
report["packaging_source"] = str(SOURCE)
report["runtime_abi_output"] = verified.stdout.strip()
report["launcher_bytes"] = (DEST / "craft-corel.exe").stat().st_size
report["launcher_source_sha256"] = hashlib.sha256((ROOT / "utiles/portable-launcher.rs").read_bytes()).hexdigest()
report["launcher_sha256"] = hashlib.sha256((DEST / "craft-corel.exe").read_bytes()).hexdigest()
report["build_state_file"] = str(state_file)
report["build_state_sha256"] = state_sha256
report["core_opt_level"] = state.get("core_opt_level", 0)
report["expected_module_dlls"] = expected_modules
report["build_command"] = state.get("build_command")
report["source_verification"] = state.get("source_verification")
report["cargo_license_package_count"] = cargo_notices["package_count"]
report["cargo_license_scope"] = cargo_notices["scope"]
report["cargo_notices_sha256"] = hashlib.sha256((DEST / "Licencias/CARGO-NOTICES.json").read_bytes()).hexdigest()
report["launcher_reused_from_manifest"] = str(args.launcher_manifest) if args.launcher_manifest else None
report["manuals_source"] = str(manuals_source)
report["manuals"] = [{"path": "Manuales/" + name, "sha256": hashlib.sha256((manuals_source / name).read_bytes()).hexdigest()} for name in manual_names]
report["max_relative_archive_path_utf16"] = max_relative_path
report["workspace_extraction_preflight_max_utf16"] = max_workspace_extraction
report["total_bytes"] = sum(path.stat().st_size for path in DEST.rglob("*") if path.is_file())
report["dll_total_bytes"] = sum(row["bytes"] for row in graph)
report["encoding_data_import_fix"] = state.get("encoding_data_import_fix")
report["source_updates"] = state.get("source_updates", [])
report["supersedes_package_version"] = state.get("supersedes_package_version")
evidence = ROOT / "evidencias/portable" / version
evidence.mkdir()
(evidence / "paquete.json").write_text(json.dumps(report,indent=2,ensure_ascii=False),encoding="utf-8")
if args.update_current:
    (ROOT / "evidencias/portable/actual-modular.json").write_text(json.dumps(report,indent=2,ensure_ascii=False),encoding="utf-8")
print(json.dumps({"directory":str(DEST),"zip":str(archive),"manifest":str(evidence / "paquete.json"),"core_opt_level":report["core_opt_level"],"dlls":[{"file":r["file"],"bytes":r["bytes"]} for r in graph]},ensure_ascii=False))
