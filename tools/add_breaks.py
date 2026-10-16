"""Adds rows to tests/breaks.json by locating each break's line from a
fragment that must appear exactly once in the file, so a break is written
against the formatted source. Asserts every match (CLAUDE.md rule 8).

    python3 tools/add_breaks.py rows.json

rows.json: [{"name", "file", "fragment", "old", "new", "test"}]: in the one
line holding `fragment`, `old` is replaced by `new`.
"""
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main(path):
    rows = json.loads(pathlib.Path(path).read_text())
    breaks = json.loads((ROOT / "tests/breaks.json").read_text())
    names = {b["name"] for b in breaks}
    for r in rows:
        text = (ROOT / r["file"]).read_text()
        lines = [l for l in text.splitlines() if r["fragment"] in l]
        assert len(lines) == 1, f'{r["name"]}: fragment found on {len(lines)} lines'
        line = lines[0]
        assert text.count(line) == 1, f'{r["name"]}: its line is not unique'
        assert r["old"] in line, f'{r["name"]}: "{r["old"]}" not in its line'
        assert r["name"] not in names, f'{r["name"]}: already a break'
        breaks.append({"name": r["name"], "file": r["file"], "find": line, "replace": line.replace(r["old"], r["new"], 1), "test": r["test"]})
        names.add(r["name"])
    (ROOT / "tests/breaks.json").write_text(json.dumps(breaks, indent=2) + "\n")
    print(f"added {len(rows)} breaks")


if __name__ == "__main__":
    main(sys.argv[1])
