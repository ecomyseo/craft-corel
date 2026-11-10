"""Comprueba inicialización, descubrimiento y llamada real en un servidor MCP stdio."""
import json
from pathlib import Path
import queue
import subprocess
import sys
import threading
import os

ROOT = Path(__file__).resolve().parents[1]
exe = Path(sys.argv[1])
name = sys.argv[2]
args = sys.argv[3:]
environment = dict(os.environ, TRACEDRAW_CONFIG_DIR=str(ROOT / "evidencias/craft-plumilla/perfil-mcp"))
process = subprocess.Popen([str(exe), *args], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                           stderr=subprocess.DEVNULL, text=True, encoding="utf-8",
                           env=environment,
                           creationflags=subprocess.CREATE_NO_WINDOW)
messages = queue.Queue()

def receive():
    for line in process.stdout:
        try:
            messages.put(json.loads(line))
        except json.JSONDecodeError:
            continue

threading.Thread(target=receive, daemon=True).start()

def request(identifier, method, params):
    process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": identifier, "method": method, "params": params}) + "\n")
    process.stdin.flush()
    while True:
        result = messages.get(timeout=35)
        if result.get("id") == identifier:
            if "error" in result:
                raise RuntimeError(result["error"])
            return result["result"]

try:
    initialized = request(1, "initialize", {"protocolVersion": "2024-11-05", "capabilities": {},
                          "clientInfo": {"name": "craft-local-verification", "version": "1.0"}})
    process.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    process.stdin.flush()
    listing = request(2, "tools/list", {})
    names = [tool["name"] for tool in listing["tools"]]
    if name == "craft-anything":
        assert "core_session_show" in names
        call = request(3, "tools/call", {"name": "core_session_show", "arguments": {}})
    elif "list_commands" in names:
        assert "list_commands" in names
        call = request(3, "tools/call", {"name": "list_commands", "arguments": {"filter": "shape.rectangle", "brief": True}})
    else:
        assert "new_document" in names
        call = request(3, "tools/call", {"name": "new_document", "arguments": {"width_mm": 100, "height_mm": 70, "title": "Prueba sin interfaz"}})
    assert call is None or not call.get("isError"), call
    report = {"application": name, "server": initialized.get("serverInfo"),
              "protocol": initialized.get("protocolVersion"), "tool_count": len(names),
              "tool_names": names, "real_call_ok": None if call is None else not call.get("isError", False)}
    destination = ROOT / "evidencias" / (name + "-mcp.json")
    destination.parent.mkdir(exist_ok=True)
    destination.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(json.dumps({k: v for k, v in report.items() if k != "tool_names"}, ensure_ascii=False))
finally:
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
