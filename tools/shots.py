"""Screenshots of the running game, for looking at it while building and
for the design journal. Not a check (docs/checks.md): it prints console
errors and saves pictures.

    .venv/bin/python tools/shots.py OUTDIR [script]

A script is a list of steps: ["click", selector], ["key", code, ms],
["wait", selector], ["sleep", ms], ["shot", name], ["eval", js].
The default script walks through a practice day.
"""
import functools
import http.server
import json
import pathlib
import sys
import threading

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parents[1]
DIST = ROOT / "dist"

DEFAULT = [
    ["wait", "body[data-ready='true']"],
    ["sleep", 600],
    ["shot", "title"],
    ["click", "button[data-action='match.start'][data-value='practice']"],
    ["wait", "body[data-phase='prep']"],
    ["sleep", 800],
    ["shot", "room"],
    ["key", "KeyA", 900],
    ["key", "KeyW", 1300],
    ["sleep", 300],
    ["shot", "library"],
]


class Quiet(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm", ".js": "text/javascript"}

    def log_message(self, *a):
        pass


def main(out, script):
    out = pathlib.Path(out)
    out.mkdir(parents=True, exist_ok=True)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=str(DIST)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}"
    errors = []
    with sync_playwright() as p:
        browser = p.chromium.launch(args=["--enable-gpu", "--ignore-gpu-blocklist", "--use-angle=metal"])
        page = browser.new_page(viewport={"width": 1280, "height": 720}, device_scale_factor=1.5)
        page.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.goto(base + "/index.html")
        for step in script:
            kind = step[0]
            if kind == "wait":
                page.wait_for_selector(step[1], state="attached", timeout=60000)
            elif kind == "click":
                page.click(step[1])
            elif kind == "key":
                page.keyboard.down(step[1])
                page.wait_for_timeout(step[2])
                page.keyboard.up(step[1])
            elif kind == "press":
                page.keyboard.press(step[1])
            elif kind == "sleep":
                page.wait_for_timeout(step[1])
            elif kind == "eval":
                print("eval:", page.evaluate(step[1]))
            elif kind == "mouse":
                page.mouse.move(step[1], step[2])
                if len(step) > 3:
                    getattr(page.mouse, step[3])()
            elif kind == "shot":
                page.screenshot(path=str(out / f"{step[1]}.jpg"), type="jpeg", quality=86)
                print("saved", step[1], "phase:", page.get_attribute("body", "data-phase"), "station:", page.get_attribute("body", "data-station"))
        browser.close()
    server.shutdown()
    for e in errors:
        print("ERROR:", e)


if __name__ == "__main__":
    script = json.loads(pathlib.Path(sys.argv[2]).read_text()) if len(sys.argv) > 2 else DEFAULT
    main(sys.argv[1], script)
