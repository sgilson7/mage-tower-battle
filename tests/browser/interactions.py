"""Mage Tower Battle's browser interactions, run by check.py in every
engine after load and idle. They play a practice day the way a person
would: walk to the library and read, save a spell at the laboratory, teach
it at the podium by drawing, start the battle, and check the result.

Every wait is on a data-* attribute the page sets, never a fixed time: on
CI the page draws on a CPU and runs slowly (docs/checks.md). Walking is
done by holding a key until the archmage's position (data-x, data-z, in
decimetres) or station says it has arrived.

Expected values come from data/content.json and the rule, not from the
page: the score after the day is the starting score minus the damage the
tower took.
"""

import json
import pathlib
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
CONTENT = json.loads((ROOT / "data" / "content.json").read_text())
STRINGS = json.loads((ROOT / "data" / "strings.json").read_text())
LONG = 240_000


def attr(page, name):
    return page.get_attribute("body", f"data-{name}") or ""


def hold_until(page, key, done, limit_s=90):
    """Holds a key (or none) until done() or the limit."""
    if key:
        page.keyboard.down(key)
    start = time.time()
    try:
        while not done() and time.time() - start < limit_s:
            page.wait_for_timeout(40)
    finally:
        if key:
            page.keyboard.up(key)


def run(page, check, engine):
    check(STRINGS["game.name"] in page.inner_text("#panels"), "the title screen names the game")
    page.click("button[data-action='match.start'][data-value='practice']")
    page.wait_for_selector("body[data-phase='prep']", state="attached", timeout=LONG)
    first = len(CONTENT["balance"]["start_pieces"])
    check(attr(page, "known") == str(first), f"a new archmage knows the {first} starting pieces in content.json")

    # the map table: place a glass wall, then the map's buttons still work
    # while it rises, and Esc closes the map (TRIAGE T3, T9)
    hold_until(page, "KeyW", lambda: attr(page, "station") == "map")
    page.keyboard.press("KeyE")
    page.wait_for_selector("body[data-overlay='on']", state="attached", timeout=LONG)
    page.click("button[data-action='layout.feature'][data-value='glass_wall']")
    box = page.locator("#overlay").bounding_box()
    page.mouse.click(box["x"] + box["width"] * 0.4, box["y"] + box["height"] * 0.25)
    page.wait_for_selector("#rising", state="attached", timeout=LONG)
    # a slow press and release, as a person clicks, while the wall rises
    pick = page.locator("button[data-action='layout.feature'][data-value='sub_tower']").bounding_box()
    page.mouse.move(pick["x"] + pick["width"] / 2, pick["y"] + pick["height"] / 2)
    page.mouse.down()
    page.wait_for_timeout(150)
    page.mouse.up()
    page.wait_for_timeout(200)
    picked = STRINGS["map.selected"] in page.locator("#layout li", has_text=CONTENT["features"][-1]["name"]).inner_text().lower()
    check(picked, "a map button still works after placing something")
    page.keyboard.press("Escape")
    page.wait_for_selector("body[data-overlay='off']", state="attached", timeout=LONG)
    check(True, "Esc closes the map")

    # walk to the library: left, then up
    hold_until(page, "KeyA", lambda: int(attr(page, "x") or 0) <= -60)
    hold_until(page, "KeyW", lambda: attr(page, "station") == "library")
    check(attr(page, "station") == "library", "the archmage reached the library on foot")
    # finding a book is a sorting game: shelve each falling book by its
    # school with keys 1 to 5 (T20)
    # some books in the pile teach nothing new: sort again until one does
    for _ in range(6):
        page.keyboard.press("KeyE")
        page.wait_for_selector("body[data-busy='sorting']", state="attached", timeout=LONG)
        while attr(page, "busy") == "sorting":
            shelf = attr(page, "book")
            if shelf != "":
                page.keyboard.press(f"Digit{int(shelf) + 1}")
            page.wait_for_timeout(40)
        page.wait_for_selector("body[data-busy='']", state="attached", timeout=LONG)
        if int(attr(page, "known")) > first:
            break
    # one piece, or a whole spell's pieces when the book was a tome
    check(int(attr(page, "known")) >= first + 1, "reading a book taught at least one more piece")

    # the laboratory: two Fire Bolts make a spell
    hold_until(page, "KeyD", lambda: attr(page, "station") == "lab")
    check(attr(page, "station") == "lab", "the archmage reached the laboratory")
    # each click waits for the panel to redraw with the piece added, so the
    # next click does not land on the panel it replaced (SECOND-ORDER R4.11)
    for n in (1, 2):
        page.click("button[data-action='lab.add'][data-value='fire_bolt']")
        # polled from here: WebKit runs no page function under the page's CSP
        hold_until(page, None, lambda: page.locator("#station .lane .slot.filled").count() == n, limit_s=LONG / 1000)
    # one more spell than before: a book of a whole spell may already have
    # saved one (SECOND-ORDER R4.11)
    before = int(attr(page, "spells") or 0)
    page.click("button[data-action='lab.save']")
    # the seal: turn each ring back to the mark with A, lock it with Space (T22)
    page.wait_for_selector("body[data-busy='sealing']", state="attached", timeout=LONG)
    while attr(page, "busy") == "sealing":
        page.keyboard.press("Space" if attr(page, "ring") == "0" else "KeyA")
        page.wait_for_timeout(40)
    page.wait_for_selector(f"body[data-spells='{before + 1}']", state="attached", timeout=LONG)
    check(page.is_visible("#station"), "the laboratory's panel is visible")

    # the podium: teach the class by drawing a stroke across the board
    hold_until(page, "KeyS", lambda: attr(page, "station") == "podium")
    check(attr(page, "station") == "podium", "the archmage reached the podium")
    page.click("button[data-action='podium.teach'][data-value='all']")
    page.wait_for_selector("body[data-busy='drawing']", state="attached", timeout=LONG)
    check(page.is_visible("#overlay"), "the drawing board is shown")
    box = page.locator("#overlay").bounding_box()
    y = box["y"] + box["height"] / 2
    page.mouse.move(box["x"] + box["width"] / 4, y)
    page.mouse.down()
    for k in range(1, 21):
        page.mouse.move(box["x"] + box["width"] / 4 + k * box["width"] / 40, y)
    page.mouse.up()
    page.wait_for_selector("body[data-busy='']", state="attached", timeout=LONG)
    check(not page.is_visible("#overlay"), "the board closes when the stroke ends")

    # the battle, at four times speed, to the day's summary and the result
    page.click("button[data-action='day.ready']")
    page.wait_for_selector("body[data-phase='battle']", state="attached", timeout=LONG)
    page.click("button[data-action='battle.speed'][data-value='4']")
    page.wait_for_selector("body[data-phase='summary']", state="attached", timeout=LONG)
    page.click("button[data-action='day.next']")
    page.wait_for_selector("body[data-phase='results']", state="attached", timeout=LONG)
    score = int(attr(page, "score"))
    taken = int(attr(page, "taken"))
    start = CONTENT["balance"]["score_start"]
    check(score == start - taken, f"the score fell by exactly the damage taken ({start} - {taken} = {score})")
    check(taken >= 0 and score <= start, "the score never rose")
