"""Probe, not a check: the physics probe's cost a tick in wasm in Chromium.

    ./scripts/build.sh && .venv/bin/python tools/probe_wasm.py

Serves dist/ on a free port, imports the module in the page and times
StressProbe.run with the browser's clock. The timing code runs through
Playwright, not in the page's own JavaScript.
"""
import functools
import http.server
import pathlib
import threading

from playwright.sync_api import sync_playwright

DIST = pathlib.Path(__file__).resolve().parents[1] / "dist"
CASES = [(120, 24, 1), (120, 24, 12), (120, 48, 24), (200, 96, 48), (300, 160, 80)]


class Quiet(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm", ".js": "text/javascript"}

    def log_message(self, *a):
        pass


def main():
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=str(DIST)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}"
    with sync_playwright() as p:
        browser = p.chromium.launch()
        page = browser.new_page()
        page.goto(base + "/index.html")
        page.wait_for_selector("body[data-ready='true']", state="attached")
        for crowd, cap, burst in CASES:
            us, awake = page.evaluate(
                """async ([crowd, cap, burst]) => {
                    const m = await import('./pkg/tool_shim.js');
                    await m.default();
                    const s = new m.StressProbe(1, crowd, cap, burst);
                    s.run(120);
                    const t = performance.now();
                    s.run(900);
                    return [(performance.now() - t) / 900 * 1000, s.awake()];
                }""",
                [crowd, cap, burst],
            )
            print(f"crowd {crowd} x4, ragdoll cap {cap}/arena, burst {burst}: {us:.1f} us/tick wasm Chromium (awake at end {awake})")
        browser.close()
    server.shutdown()


if __name__ == "__main__":
    main()
