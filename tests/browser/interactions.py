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
    page.keyboard.down(key)
    start = time.time()
    try:
        while not done() and time.time() - start < limit_s:
            page.wait_for_timeout(40)
    finally:
        page.keyboard.up(key)


def run(page, check, engine):
    check(STRINGS["game.name"] in page.inner_text("#panels"), "the title screen names the game")
    page.click("button[data-action='match.start'][data-value='practice']")
    page.wait_for_selector("body[data-phase='prep']", state="attached", timeout=LONG)
    first = len(CONTENT["balance"]["start_pieces"])
    check(attr(page, "known") == str(first), f"a new archmage knows the {first} starting pieces in content.json")

    # walk to the library: left, then up
    hold_until(page, "KeyA", lambda: int(attr(page, "x") or 0) <= -60)
    hold_until(page, "KeyW", lambda: attr(page, "station") == "library")
    check(attr(page, "station") == "library", "the archmage reached the library on foot")
    page.keyboard.press("KeyE")
    page.wait_for_selector("body[data-busy='reading']", state="attached", timeout=LONG)
    page.wait_for_selector("body[data-busy='']", state="attached", timeout=LONG)
    # one piece, or a whole spell's pieces when the book was a tome
    check(int(attr(page, "known")) >= first + 1, "reading a book taught at least one more piece")

    # the laboratory: two Fire Bolts make a spell
    hold_until(page, "KeyD", lambda: attr(page, "station") == "lab")
    check(attr(page, "station") == "lab", "the archmage reached the laboratory")
    page.click("button[data-action='lab.add'][data-value='fire_bolt']")
    page.click("button[data-action='lab.add'][data-value='fire_bolt']")
    page.click("button[data-action='lab.save']")
    page.wait_for_selector("body[data-spells='2']", state="attached", timeout=LONG)
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
