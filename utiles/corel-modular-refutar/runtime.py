"""Transporte MCP oculto, con configuración aislada y evidencia del artefacto."""
from pathlib import Path
import hashlib
import json
import os
import queue
import subprocess
import threading

ROOT = Path(__file__).resolve().parents[2]
FLAGS = subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def identity(runtime):
    runtime = Path(runtime).resolve(strict=True)
    assert runtime.is_relative_to(ROOT)
    return {file.name: sha(file) for file in runtime.iterdir() if file.suffix.lower() in {".dll", ".exe"}}


class Mcp:
    def __init__(self, runtime, folder, color):
        self.runtime = Path(runtime).resolve(strict=True)
        self.folder = Path(folder)
        self.folder.mkdir(parents=True, exist_ok=False)
        profile = self.folder / "config"
        profile.mkdir()
        (profile / "settings.json").write_text(json.dumps({"language": "es", "color": color}), encoding="utf-8")
        self.stderr = (self.folder / "stderr.log").open("wb")
        self.process = subprocess.Popen([str(self.runtime / "craft-corel.exe"), "--mcp"], cwd=self.runtime,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.stderr, text=True, encoding="utf-8",
            env=dict(os.environ, TRACEDRAW_CONFIG_DIR=str(profile), CRAFT_COREL_CONFIG_DIR=str(profile)), creationflags=FLAGS)
        self.queue = queue.Queue()
        self.counter = 0
        self.calls = []
        def read():
            for line in self.process.stdout:
                try:
                    self.queue.put(json.loads(line))
                except json.JSONDecodeError:
                    pass
            self.queue.put({"closed": True})
        threading.Thread(target=read, daemon=True).start()
        self.request("initialize", {"protocolVersion": "2024-11-05", "capabilities": {},
            "clientInfo": {"name": "Lumen-modular-refutation", "version": "1"}})

    def request(self, method, params):
        self.counter += 1
        self.process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": self.counter, "method": method, "params": params}) + "\n")
        self.process.stdin.flush()
        while True:
            reply = self.queue.get(timeout=90)
            assert not reply.get("closed"), "El proceso MCP se cerró."
            if reply.get("id") == self.counter:
                assert "error" not in reply, reply
                return reply["result"]

    def call(self, tool, arguments=None):
        reply = self.request("tools/call", {"name": tool, "arguments": arguments or {}})
        assert not reply.get("isError", False), reply
        self.calls.append(tool)
        return reply["content"][0]["text"]

    def save(self, filename):
        path = self.folder / filename
        self.call("save", {"path": str(path)})
        return json.loads(path.read_text(encoding="utf-8"))

    def close(self):
        self.process.stdin.close()
        try:
            assert self.process.wait(timeout=10) == 0
        finally:
            if self.process.poll() is None:
                self.process.terminate()
            self.stderr.close()
            (self.folder / "calls.json").write_text(json.dumps(self.calls, indent=2), encoding="utf-8")
