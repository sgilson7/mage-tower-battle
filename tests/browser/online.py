#!/usr/bin/env python3
"""Online check: two browsers meet by pasted codes and play a day on one origin.

From the online kit (kits/online/README.md), after Galactic Survivors' copy.
It sits beside the browser check, not inside it, because that check fails on
any WebSocket or off-origin request and must keep doing so.

How it stays on the origin:

- `?ice=none` makes web/net/config.js drop the STUN servers, so the two
  browsers connect on host candidates alone;
- the pasted-code path carries the whole handshake in the codes, so no relay
  is contacted;
- host addresses are not hidden behind mDNS names, which a CI runner may not
  resolve.

It fails on: a page or console error, any request off the origin, any
WebSocket (the room-code path must not start), a browser that never reaches
the match, the two browsers' day summaries differing, or a desync toast.

What it does not cover: the room-code path over public relays, STUN, TURN,
and two real networks. Those are checked by hand before a release and the
result written in HANDOFF.md.

    .venv/bin/python tests/browser/online.py [chromium firefox ...]   (default: chromium)
"""
import functools
import http.server
import json
import pathlib
import sys
import threading

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parents[2]
DIST = ROOT / "dist"
STRINGS = json.loads((ROOT / "data" / "strings.json").read_text())

# The page's online flow: the Online screen, its "Host by pasted codes" and
# "Join by pasted codes" buttons, the joiner's Ready, and the one text box
# (#ask-text) that shows or takes a code. The room's status is on <body> as
# data-net, the match's phase as data-phase.
SETUP = "button[data-action='screen.show'][data-value='setup']"
ONLINE = "button[data-action='screen.show'][data-value='online']"
INVITE = "button[data-action='page.invite']"
REPLY = "button[data-action='page.reply']"
READY = "button[data-action='online.ready']"
START = "button[data-action='online.start']:not([disabled])"
DAY_READY = "button[data-action='day.ready']"
TIMEOUT_MS = 15000

failures = []


def check(ok, what):
    print(("  PASS " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


class Quiet(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
    }

    def log_message(self, *args):
        pass


def serve():
    handler = functools.partial(Quiet, directory=str(DIST))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server


# Both browsers share this machine, so plain local addresses are what lets
# them meet without STUN.
LAUNCH = {
    "chromium": {"args": ["--disable-features=WebRtcHideLocalIpsWithMdns"]},
    "firefox": {"firefox_user_prefs": {"media.peerconnection.ice.obfuscate_host_addresses": False}},
}


def open_page(browser, origin, who, seen):
    context = browser.new_context()
    page = context.new_page()
    page.on("console", lambda m: seen["errors"].append(f"{who}: {m.text}") if m.type == "error" else None)
    page.on("pageerror", lambda e: seen["errors"].append(f"{who}: {e}"))
    page.on("websocket", lambda ws: seen["sockets"].append(f"{who}: {ws.url}"))
    page.on(
        "request",
        lambda r: None if r.url.startswith((origin + "/", "blob:", "data:")) else seen["offsite"].append(r.url),
    )
    page.goto(origin + "/index.html?ice=none")
    page.wait_for_selector("body[data-ready='true']", state="attached", timeout=TIMEOUT_MS)
    return page


def run(p, engine, origin):
    print(f"\n[{engine}] online, two browsers, pasted codes, one day")
    browser = getattr(p, engine).launch(**LAUNCH.get(engine, {}))
    seen = {"errors": [], "sockets": [], "offsite": []}
    host = open_page(browser, origin, "host", seen)
    joiner = open_page(browser, origin, "joiner", seen)

    # the shortest match: one day, the shortest preparation and battle
    host.click(SETUP)
    for action, times in (("setup.days", 8), ("setup.prep", 16), ("setup.battle", 8)):
        for _ in range(times):
            host.click(f"button[data-action='{action}'][data-value='down']")
    host.click("button[data-action='screen.show'][data-value='title']")
    for page in (host, joiner):
        page.click(ONLINE)
    host.click(INVITE)
    host.wait_for_selector("#ask-text", timeout=TIMEOUT_MS)
    offer = ""
    for _ in range(150):
        offer = host.input_value("#ask-text")
        if offer.startswith("O"):
            break
        host.wait_for_timeout(100)
    check(offer.startswith("O"), "the host shows an invitation to send")

    joiner.click(REPLY)
    joiner.fill("#ask-text", offer)
    joiner.click("#ask button >> nth=0")
    joiner.wait_for_selector("#ask p", timeout=TIMEOUT_MS)
    answer = ""
    for _ in range(100):
        answer = joiner.input_value("#ask-text")
        if answer.startswith("A"):
            break
        joiner.wait_for_timeout(100)
    check(answer.startswith("A"), "the joiner shows a reply code")

    host.fill("#ask-text", answer)
    host.click("#ask button >> nth=1")
    joiner.click(READY, timeout=TIMEOUT_MS)
    host.wait_for_selector(START, timeout=TIMEOUT_MS)
    check(True, "the partner pressed Ready and the host may start")
    host.click(START)
    for who, page in (("host", host), ("joiner", joiner)):
        try:
            page.wait_for_selector("body[data-phase='prep'][data-net='online.playing']", state="attached", timeout=TIMEOUT_MS)
            ok = True
        except Exception:
            ok = False
        check(ok, f"the {who} is in the match")
    # the joiner ends its preparation first and waits for the host's plan
    joiner.click(DAY_READY)
    joiner.wait_for_selector("body[data-phase='waiting']", state="attached", timeout=TIMEOUT_MS)
    check(True, "the joiner waits for the host's plan")
    host.click(DAY_READY)
    # a battle lasts until its enemies are gone (T54): both browsers watch
    # at four times speed, and a slow CI machine gets ten minutes
    for page in (host, joiner):
        try:
            page.wait_for_selector("body[data-phase='battle']", state="attached", timeout=TIMEOUT_MS * 8)
            page.click("button[data-action='battle.speed'][data-value='4']", timeout=TIMEOUT_MS)
        except Exception:
            pass
    summaries = []
    for who, page in (("host", host), ("joiner", joiner)):
        try:
            page.wait_for_selector("body[data-phase='summary']", state="attached", timeout=600000)
            summaries.append(page.inner_text("#summary"))
        except Exception:
            summaries.append("")
    check(all(summaries), "both browsers reach the day summary")
    check(summaries[0] == summaries[1], "the two day summaries agree")
    toasts = [p.inner_text("#toasts") for p in (host, joiner)]
    desync = STRINGS["online.desync"]
    check(not any(desync in t for t in toasts), "no browser reports a desync")
    statuses = [p.get_attribute("body", "data-net") for p in (host, joiner)]
    check(all(st == "online.playing" for st in statuses), f"no browser reports a drop {statuses}")
    check(not seen["errors"], f"no page or console error {seen['errors'][:3]}")
    check(not seen["offsite"], f"no request off the origin {seen['offsite'][:3]}")
    check(not seen["sockets"], f"no WebSocket, so no relay was contacted {seen['sockets'][:3]}")
    browser.close()


def main():
    if not (DIST / "index.html").exists():
        sys.exit("dist/ is missing: run scripts/build.sh first")
    engines = sys.argv[1:] or ["chromium"]
    server = serve()
    origin = "http://127.0.0.1:%d" % server.server_address[1]
    with sync_playwright() as p:
        for engine in engines:
            run(p, engine, origin)
    server.shutdown()
    if failures:
        print(f"\nONLINE CHECK FAILED: {len(failures)} problem(s)")
        for f in failures:
            print("  - " + f)
        sys.exit(1)
    print(f"\nONLINE CHECK PASSED in {', '.join(engines)}")


if __name__ == "__main__":
    main()
