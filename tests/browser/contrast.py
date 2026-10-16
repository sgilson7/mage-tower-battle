#!/usr/bin/env python3
"""Contrast check (T6): every button, chip and row on the title, the
glossary and the spell laboratory has text that stands out from what is
behind it, in the light and the dark colour scheme.

The rule is WCAG's contrast ratio, at least 3:1 (large or bold text), from
each element's computed text colour and the first solid background behind
it. A disabled button is skipped: it is meant to fade.

    .venv/bin/python tests/browser/contrast.py [chromium firefox webkit]
"""
import functools
import http.server
import pathlib
import sys
import threading

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parents[2]
DIST = ROOT / "dist"
MIN_RATIO = 3.0

# In the page: the contrast of each visible text element against the first
# opaque background behind it.
MEASURE = """() => {
  const rgb = (s) => (s.match(/[\\d.]+/g) || []).map(Number);
  const lum = ([r, g, b]) => {
    const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; };
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
  };
  const back = (el) => {
    for (let e = el; e; e = e.parentElement) {
      const c = rgb(getComputedStyle(e).backgroundColor);
      if (c.length >= 3 && (c.length === 3 || c[3] > 0.6)) return c.slice(0, 3);
    }
    return [27, 22, 38];
  };
  const out = [];
  for (const el of document.querySelectorAll("#panels button, #panels .chip, #panels .row-label, #panels .row-detail, #panels h2, #panels h3, #panels p, #panels .mana, #panels .card-kind, #panels .card-number")) {
    if (el.disabled || !el.offsetParent || !el.textContent.trim()) continue;
    const fg = lum(rgb(getComputedStyle(el).color).slice(0, 3));
    const bg = lum(back(el));
    const ratio = (Math.max(fg, bg) + 0.05) / (Math.min(fg, bg) + 0.05);
    out.push([el.textContent.trim().slice(0, 40), Math.round(ratio * 100) / 100]);
  }
  return out;
}"""

failures = []


class Quiet(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm", ".js": "text/javascript"}

    def log_message(self, *args):
        pass


def look(page, where, scheme):
    low = [(t, r) for t, r in page.evaluate(MEASURE) if r < MIN_RATIO]
    ok = not low
    print(("  PASS " if ok else "  FAIL ") + f"{scheme} {where}: every text stands out {low[:4]}")
    if not ok:
        failures.append(f"{scheme} {where}: {low[:4]}")


def hold_until(page, keys, station, limit_ms=10000):
    for k in keys:
        page.keyboard.down(k)
    waited = 0
    while waited < limit_ms and page.get_attribute("body", "data-station") != station:
        page.wait_for_timeout(50)
        waited += 50
    for k in keys:
        page.keyboard.up(k)


def run(p, engine, origin):
    browser = getattr(p, engine).launch()
    for scheme in ("light", "dark"):
        page = browser.new_page(viewport={"width": 1280, "height": 720}, color_scheme=scheme)
        page.goto(origin + "/index.html")
        page.wait_for_selector("body[data-ready='true']", state="attached", timeout=60000)
        print(f"\n[{engine}, {scheme}]")
        look(page, "title", scheme)
        page.click("#glossary-button")
        page.wait_for_selector("#panels .panel-glossary .card", state="attached")
        look(page, "glossary", scheme)
        page.click("button[data-action='glossary.close']")
        page.click("button[data-action='match.start'][data-value='practice']")
        page.wait_for_selector("body[data-phase='prep']", state="attached")
        hold_until(page, ["KeyD", "KeyW"], "lab")
        page.locator("button[data-action='lab.add']").first.click()
        page.locator("button[data-action='lab.add']").last.click()
        page.wait_for_timeout(200)
        look(page, "spell laboratory with a draft", scheme)
        page.close()
    browser.close()


def main():
    if not (DIST / "index.html").exists():
        sys.exit("dist/ is missing: run scripts/build.sh first")
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=str(DIST)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    origin = "http://127.0.0.1:%d" % server.server_address[1]
    engines = sys.argv[1:] or ["chromium"]
    with sync_playwright() as p:
        for engine in engines:
            run(p, engine, origin)
    server.shutdown()
    if failures:
        print(f"\nCONTRAST CHECK FAILED: {len(failures)} problem(s)")
        sys.exit(1)
    print(f"\nCONTRAST CHECK PASSED in {', '.join(engines)}")


if __name__ == "__main__":
    main()
