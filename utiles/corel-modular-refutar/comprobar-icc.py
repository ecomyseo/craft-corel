"""Refuta apertura ICC y render calibrado del artefacto modular concreto."""
from pathlib import Path
import argparse
import base64
import io
import json
import struct
import time
from PIL import Image
from runtime import ROOT, Mcp, identity, sha

parser = argparse.ArgumentParser()
parser.add_argument("--runtime", type=Path, required=True)
parser.add_argument("--reference-runtime", type=Path, required=True)
parser.add_argument("--fixtures", type=Path, required=True)
args = parser.parse_args()
args.fixtures = args.fixtures.resolve(strict=True)
assert args.fixtures.is_relative_to(ROOT)
reference_identity = identity(args.reference_runtime)
candidate_identity = identity(args.runtime)
assert reference_identity["craft-corel-engine.dll"] == "4010ba1c00244197e9c5ea85081fabc138ef38b3d857c9edec11b7bf7dcd71ec"
assert candidate_identity["craft-corel-engine.dll"] != reference_identity["craft-corel-engine.dll"], "La referencia no puede ser el propio candidato."
assert "encoding_rs.dll" in candidate_identity, "Se requiere el artefacto modular nuevo."
out = ROOT / "evidencias/craft-corel-nativo/optimizacion/refutacion/icc" / str(time.time_ns())
out.mkdir(parents=True)
oracle = json.loads((args.fixtures / "oracles.json").read_text(encoding="utf-8"))
profiles = {}
for key in ["source_a", "destination_b"]:
    path = out / (key + ".icc")
    path.write_bytes(base64.b64decode(oracle[key]["data"]))
    profiles[key] = path
source_hashes = {p.name: sha(p) for p in args.fixtures.iterdir() if p.suffix in {".tdraw", ".cdr", ".tif"}}


def pictures(value):
    result = []
    if isinstance(value, dict):
        if isinstance(value.get("kind"), dict) and value["kind"].get("type") == "bitmap":
            result.append(value)
        for child in value.values():
            result.extend(pictures(child))
    elif isinstance(value, list):
        for child in value:
            result.extend(pictures(child))
    return result


def image_bytes(data):
    with Image.open(io.BytesIO(data)) as image:
        return image.size, image.convert("RGBA").tobytes()


def inks(shape):
    width, height = shape["kind"]["width_px"], shape["kind"]["height_px"]
    assert shape["picture_inks"]["inks"] == [{"process": i} for i in range(4)]
    with Image.open(io.BytesIO(base64.b64decode(shape["picture_inks"]["png"]))) as image:
        assert image.size == (width, height * 4)
        planes = image.convert("L").tobytes()
    _, preview = image_bytes(base64.b64decode(shape["kind"]["png"]))
    count = width * height
    return bytes(v for i in range(count) for v in [planes[i], planes[count+i], planes[count*2+i], planes[count*3+i], preview[i*4+3]])


def preferences(policy=0, target="destination_b", builtin=False):
    return {"open_cmyk": policy, "cmyk_profile": oracle[target]["name"] if target else "Generic CMYK (open)",
            "cmyk_profile_path": str(profiles[target]) if target else "", "rgb_profile_path": "",
            "intent": "Saturation", "black_point_compensation": False, "preserve_pure_black": False,
            "builtin_engine": builtin, "warn_open_mismatch": False}


def render(mcp, name):
    path = mcp.folder / name
    mcp.call("export", {"path": str(path), "dpi": 96})
    return image_bytes(path.read_bytes())


# Remove only the ICC tag from our own classic little-endian TIFF, preserving
# every pixel, its alpha and all other tags. The original file is never written.
untagged_bytes = bytearray((args.fixtures / "owned-source.tif").read_bytes())
assert untagged_bytes[:4] == b"II*\x00"
offset = struct.unpack_from("<I", untagged_bytes, 4)[0]
count = struct.unpack_from("<H", untagged_bytes, offset)[0]
tags = []
for index in range(count):
    position = offset + 2 + index * 12
    tag = struct.unpack_from("<H", untagged_bytes, position)[0]
    if tag == 34675:
        tags.append(position)
assert len(tags) == 1
struct.pack_into("<H", untagged_bytes, tags[0], 65000)
untagged_path = out / "owned-untagged.tif"
untagged_path.write_bytes(untagged_bytes)

results = []
opening = {}
vectors = {}
native_observations = []
invalid_defaults = []
for name, runtime in [("reference", args.reference_runtime), ("candidate", args.runtime)]:
    for policy in [0, 1, 2]:
        for filename in ["owned-source.tif", "owned-source.tdraw", "owned-source-v27.cdr"]:
            mcp = Mcp(runtime, out / f"{name}-open-{policy}-{filename.replace('.', '-')}", preferences(policy))
            try:
                mcp.call("open", {"path": str(args.fixtures / filename)})
                doc = mcp.save("opened.tdraw")
                # try_open_path applies opening policy only to CDR and CMYK
                # raster files. Native TDRAW intentionally restores its own
                # calibration regardless of that external-file preference.
                applies = not filename.endswith(".tdraw")
                target = oracle["source_a"] if policy == 0 or not applies else oracle["destination_b"]
                assert doc["cmyk_profile"]["data"] == target["data"]
                found = pictures(doc)
                assert len(found) == 1
                expected = oracle["a_to_b_cmyka"] if policy == 2 and applies else oracle["original_cmyka"]
                assert inks(found[0]) == bytes(expected)
                assert found[0]["picture_inks"]["profile"]["data"] == target["data"]
                if not applies and policy != 0:
                    native_observations.append({"runtime":name,"preference":policy,
                        "observed_document_profile":"source A retained", "observed_inks":"original unchanged",
                        "status":"Observation of current native route, not proof that this preference is implemented for TDRAW."})
                opening[(name, policy, filename)] = render(mcp, "opened.png")
            finally:
                mcp.close()
    for policy in [0, 1, 2]:
        for filename in ["owned-source.tif", "owned-source-v27.cdr"]:
            invalid = preferences(policy)
            invalid["cmyk_profile_path"] = str(out / "missing-default-b.icc")
            mcp = Mcp(runtime, out / f"{name}-invalid-default-{policy}-{filename.replace('.', '-')}", invalid)
            try:
                mcp.call("open", {"path":str(args.fixtures / filename)})
                doc = mcp.save("retained-source.tdraw")
                assert doc["cmyk_profile"]["data"] == oracle["source_a"]["data"]
                found = pictures(doc)
                assert len(found) == 1
                assert inks(found[0]) == bytes(oracle["original_cmyka"])
                assert found[0]["picture_inks"]["profile"]["data"] == oracle["source_a"]["data"]
                assert render(mcp, "retained-source.png") == opening[(name,0,filename)]
                invalid_defaults.append({"runtime":name,"policy":policy,"format":filename,"source_calibration_and_inks_retained":True})
            finally:
                mcp.close()
    for builtin in [False, True]:
        mcp = Mcp(runtime, out / f"{name}-vector-builtin-{builtin}", preferences(0, "source_a", builtin))
        try:
            mcp.call("open", {"path": str(args.fixtures / "owned-source.tdraw")})
            mcp.call("run_script", {"source": "var s = ActiveLayer.CreateRectangle(0, 0, 2, 2); s.Fill.ApplyUniformFill(Color.CMYK(17,53,91,23));"})
            vectors[(name, builtin)] = render(mcp, "vector.png")
        finally:
            mcp.close()
    controls = []
    for after_tagged in [False, True]:
        mcp = Mcp(runtime, out / f"{name}-untagged-after-{after_tagged}", preferences(0, None))
        try:
            if after_tagged:
                mcp.call("open", {"path": str(args.fixtures / "owned-source.tif")})
            mcp.call("open", {"path": str(untagged_path)})
            doc = mcp.save("untagged.tdraw")
            assert doc.get("cmyk_profile") is None
            found = pictures(doc)
            assert len(found) == 1
            assert found[0]["picture_inks"].get("profile") is None
            assert inks(found[0]) == bytes(oracle["original_cmyka"])
            controls.append((found[0]["kind"]["png"], render(mcp, "untagged.png")))
        finally:
            mcp.close()
    assert controls[0] == controls[1], "La calibración del documento previo se ha filtrado al archivo sin ICC."
    results.append({"runtime": name, "external_opening_policy_cases": 6, "native_keep_policy0_case": 1,
        "native_other_preferences_observed_only":2,"invalid_default_source_retention_cases":6,"untagged_after_tagged_matches_fresh": True})
for policy in [0, 1, 2]:
    for filename in ["owned-source.tif", "owned-source.tdraw", "owned-source-v27.cdr"]:
        assert opening[("reference", policy, filename)] == opening[("candidate", policy, filename)]
assert vectors[("reference", False)] == vectors[("candidate", False)], "El render vectorial ICC difiere."
for name in ["reference", "candidate"]:
    assert vectors[(name, False)] != vectors[(name, True)], "El control builtin no distingue una calibración real."
assert {p.name: sha(p) for p in args.fixtures.iterdir() if p.name in source_hashes} == source_hashes
report = {"passed": True, "cases": results, "candidate": candidate_identity, "reference": reference_identity,
          "native_policy_observations":native_observations,"invalid_default_cases":invalid_defaults,
          "native_policy_limit":"TDRAW preferences 1/2 do not assign or convert in either artifact. CDR/TIFF do. No equivalence claim for native TDRAW policy handling.",
          "frozen_oracles_sha256": sha(args.fixtures / "oracles.json"), "calibrated_rgba_exact": True,
          "builtin_negative_control_distinct": True, "source_files_unchanged": True,
          "scope": "Own files, native MCP and full RGBA comparisons; no original Corel, Windows clipboard or visible GUI."}
(out / "results.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
print(json.dumps({"passed": True, "evidence": str(out)}))
