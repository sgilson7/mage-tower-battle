"""Writes data/models.json: every character as spheres and a few cones and
cylinders, after the journal's mockups (entry 1, decision 37: sphere-built,
like Chumbles, not cubes).

    python3 tools/models.py

Each part: mesh, position and size in cm (size is the diameter on each
axis), a tilt in thousandths of a turn (pitch, roll), a colour role from
data/palette.json, the body node it moves with, and whether the far view
(the four-tower watch screen) draws it.
"""
import json
import pathlib

OUT = pathlib.Path(__file__).resolve().parents[1] / "data" / "models.json"


def part(mesh, p, s, c, node="body", tilt=(0, 0), far=False):
    return {"m": mesh, "p": [round(v) for v in p], "s": [round(v) for v in s], "c": c, "n": node, "t": list(tilt), "far": far}


def ball(r, c, x, y, z, sx=1, sy=1, sz=1, node="body", far=False):
    return part("sphere", (x * 100, y * 100, z * 100), (2 * r * sx * 100, 2 * r * sy * 100, 2 * r * sz * 100), c, node, far=far)


def face(y, z, r, node="head"):
    return [
        ball(r * 0.13, "eye", -r * 0.36, y + r * 0.1, z + r * 0.86, node=node),
        ball(r * 0.13, "eye", r * 0.36, y + r * 0.1, z + r * 0.86, node=node),
        ball(r * 0.05, "white", -r * 0.32, y + r * 0.16, z + r * 0.97, node=node),
        ball(r * 0.05, "white", r * 0.4, y + r * 0.16, z + r * 0.97, node=node),
        ball(r * 0.14, "blush", -r * 0.62, y - r * 0.18, z + r * 0.72, 1, 0.6, 0.5, node=node),
        ball(r * 0.14, "blush", r * 0.62, y - r * 0.18, z + r * 0.72, 1, 0.6, 0.5, node=node),
    ]


def mage(arch=False, size=1.0):
    r = 0.5
    hy = r * 2.25
    out = [
        ball(r * 1.05, "robe", 0, r * 0.75, 0, 1, 0.8, 1, far=True),
        ball(r * 0.82, "robe", 0, r * 1.35, 0),
        ball(r * 0.86, "trim", 0, r * 0.16, 0, 1.08, 0.18, 1.08),
        ball(r * 0.74, "skin", 0, hy, 0, node="head", far=True),
    ]
    out += face(hy, 0, r * 0.74)
    if arch:
        for x, y, z, s in [(0, -0.42, 0.5, 0.3), (-0.2, -0.35, 0.48, 0.22), (0.2, -0.35, 0.48, 0.22), (0, -0.62, 0.42, 0.22), (-0.12, -0.56, 0.46, 0.18), (0.12, -0.56, 0.46, 0.18)]:
            out.append(ball(s * r, "beard", x * r * 1.3, hy + y * r, z * r, node="head"))
    hr = r * 0.74
    hat_y = hy + r * 0.55
    tall = 1.9 if arch else 1.3
    out.append(ball(hr * 1.35, "hat", 0, hat_y, 0, 1, 0.16, 1, node="head", far=True))
    out.append(part("cone", (0, (hat_y + hr * tall * 0.8) * 100, 0), (hr * 1.56 * 100, hr * tall * 1.6 * 100, hr * 1.56 * 100), "hat", "head", tilt=(-30, 50 if arch else 25)))
    out.append(ball(hr * 0.16, "gold", -hr * 0.4, hat_y + hr * tall * 1.5, 0.05, node="head"))
    out.append(ball(r * 0.2, "skin", -r * 0.95, r * 1.1, r * 0.35, node="hand_l"))
    out.append(ball(r * 0.2, "skin", r * 0.95, r * 1.1, r * 0.35, node="hand_r"))
    if arch:
        out.append(part("cyl", (r * 105, r * 160, r * 35), (10, 240, 10), "wood", "hand_r"))
        out.append(ball(0.17, "staff_orb", r * 1.05, r * 1.6 + 1.25, r * 0.35, node="hand_r"))
    return scale(out, size)


def scale(parts, k):
    for p in parts:
        p["p"] = [round(v * k) for v in p["p"]]
        p["s"] = [round(v * k) for v in p["s"]]
    return parts


def penguin():
    out = [
        ball(0.55, "penguin", 0, 0.6, 0, 1, 1.15, 1, far=True),
        ball(0.42, "white", 0, 0.55, 0.18, 1, 1.15, 0.9),
        ball(0.42, "penguin", 0, 1.35, 0, node="head", far=True),
        ball(0.3, "white", 0, 1.33, 0.16, node="head"),
    ]
    out += face(1.38, 0.0, 0.42)
    out.append(part("cone", (0, 125, 44), (20, 28, 20), "beak", "head", tilt=(250, 0)))
    out += [ball(0.16, "beak", -0.2, 0.06, 0.1, 1.2, 0.4, 1.6, node="foot_l"), ball(0.16, "beak", 0.2, 0.06, 0.1, 1.2, 0.4, 1.6, node="foot_r")]
    out += [ball(0.2, "penguin", 0.55, 1.0, 0.05, 0.5, 1.4, 0.8, node="hand_r"), ball(0.2, "penguin", -0.52, 0.7, 0.05, 0.5, 1.4, 0.8, node="hand_l")]
    return out


def golem():
    return [
        ball(0.8, "stone", 0, 1.2, 0, 1, 0.9, 0.85, far=True),
        ball(0.5, "stone", 0, 2.1, 0.1, node="head", far=True),
        ball(0.09, "golem_eye", -0.18, 2.15, 0.55, node="head"),
        ball(0.09, "golem_eye", 0.18, 2.15, 0.55, node="head"),
        ball(0.42, "stone", -0.95, 1.1, 0.25, node="hand_l"),
        ball(0.42, "stone", 0.95, 1.1, 0.25, node="hand_r"),
        ball(0.36, "stone", -0.42, 0.35, 0, node="foot_l"),
        ball(0.36, "stone", 0.42, 0.35, 0, node="foot_r"),
        ball(0.3, "moss", 0.2, 2.5, 0, 1.4, 0.4, 1.2, node="head"),
    ]


def wisp(role="wisp_pink"):
    out = [ball(0.45, role, 0, 1.4, 0, node="head", far=True), ball(0.3, role, 0, 0.85, -0.05, far=True), ball(0.18, role, 0.05, 0.5, -0.1)]
    out += face(1.42, 0, 0.45)
    return out


def trainer():
    out = mage(size=0.9)
    for p in out:
        if p["c"] == "robe":
            p["c"] = "trainer"
    out.append(ball(0.08, "gold", 0, 1.0, 0.35, node="body"))
    return out


def giant(skin="giant"):
    out = [
        ball(1.3, skin, 0, 2.0, 0, 1, 1.05, 0.9, far=True),
        ball(0.75, "giant_face", 0, 3.5, 0.2, node="head", far=True),
    ]
    out += face(3.55, 0.2, 0.75)
    out += [ball(0.5, "giant_face", -1.4, 2.2, 0.2, node="hand_l"), ball(0.5, "giant_face", 1.4, 2.2, 0.2, node="hand_r")]
    out += [ball(0.55, skin, -0.6, 0.55, 0, node="foot_l"), ball(0.55, skin, 0.6, 0.55, 0, node="foot_r")]
    out.append(part("cyl", (170, 300, 40), (40, 260, 40), "club", "hand_r", tilt=(0, -80)))
    return out


def skeleton(armour=False):
    out = [ball(0.32, "bone", 0, 1.45, 0, node="head", far=True), ball(0.08, "eye", -0.11, 1.48, 0.27, node="head"), ball(0.08, "eye", 0.11, 1.48, 0.27, node="head")]
    for i, y in enumerate([1.05, 0.9, 0.75]):
        out.append(ball(0.22 - i * 0.02, "bone", 0, y, 0, 1.4, 0.45, 1, far=(i == 0)))
    out += [ball(0.1, "bone", -0.32, 0.95, 0.1, node="hand_l"), ball(0.1, "bone", 0.32, 0.95, 0.1, node="hand_r")]
    out += [ball(0.12, "bone", -0.14, 0.3, 0, node="foot_l"), ball(0.12, "bone", 0.14, 0.3, 0, node="foot_r")]
    if armour:
        out += [ball(0.36, "armour", 0, 1.0, 0, 1.2, 0.9, 1.0, far=True), ball(0.34, "armour", 0, 1.55, 0, 1, 0.5, 1, node="head")]
    return out


def elf():
    out = [ball(0.42, "elf_robe", 0, 0.6, 0, 1, 1.2, 1, far=True), ball(0.36, "elf_skin", 0, 1.25, 0, node="head", far=True)]
    out += face(1.27, 0, 0.36)
    out.append(part("cone", (38, 133, 0), (16, 36, 16), "elf_skin", "head", tilt=(0, -190)))
    out.append(part("cone", (-38, 133, 0), (16, 36, 16), "elf_skin", "head", tilt=(0, 190)))
    out.append(ball(0.3, "leaf_hat", 0, 1.55, 0, 1.2, 0.5, 1.2, node="head"))
    return out


def imp():
    out = [ball(0.32, "ember", 0, 0.5, 0, 1, 1.1, 1, far=True), ball(0.3, "ember", 0, 1.0, 0.05, node="head", far=True)]
    out += face(1.02, 0.05, 0.3)
    out.append(part("cone", (14, 128, 0), (10, 22, 10), "horn", "head", tilt=(0, -60)))
    out.append(part("cone", (-14, 128, 0), (10, 22, 10), "horn", "head", tilt=(0, 60)))
    return out


def troll():
    out = [ball(0.95, "frost_fur", 0, 1.5, 0, 1, 1.05, 0.9, far=True), ball(0.55, "frost_skin", 0, 2.55, 0.15, node="head", far=True)]
    out += face(2.58, 0.15, 0.55)
    out += [ball(0.38, "frost_skin", -1.0, 1.5, 0.2, node="hand_l"), ball(0.38, "frost_skin", 1.0, 1.5, 0.2, node="hand_r")]
    out += [ball(0.4, "frost_fur", -0.45, 0.4, 0, node="foot_l"), ball(0.4, "frost_fur", 0.45, 0.4, 0, node="foot_r")]
    return out


def slime():
    out = [ball(0.62, "slime", 0, 0.45, 0, 1, 0.72, 1, far=True), ball(0.3, "slime", 0.1, 0.75, 0.1, node="head")]
    out += face(0.55, 0.3, 0.4, node="head")
    return out


def boar():
    out = [ball(0.55, "boar", 0, 0.65, 0, 0.9, 0.85, 1.25, far=True), ball(0.4, "boar", 0, 0.75, 0.6, node="head", far=True)]
    out += face(0.8, 0.62, 0.4)
    out.append(ball(0.16, "snout", 0, 0.68, 0.98, 1, 0.8, 0.6, node="head"))
    out.append(part("cone", (16, 64, 92), (8, 20, 8), "white", "head", tilt=(200, 0)))
    out.append(part("cone", (-16, 64, 92), (8, 20, 8), "white", "head", tilt=(200, 0)))
    out += [ball(0.16, "boar", -0.3, 0.16, 0.35, node="foot_l"), ball(0.16, "boar", 0.3, 0.16, 0.35, node="foot_r"), ball(0.16, "boar", -0.3, 0.16, -0.35, node="foot_l"), ball(0.16, "boar", 0.3, 0.16, -0.35, node="foot_r")]
    return out


def ghost():
    out = [ball(0.5, "ghost", 0, 1.3, 0, node="head", far=True), ball(0.4, "ghost", 0, 0.85, -0.05, far=True), ball(0.25, "ghost", 0.1, 0.5, -0.15)]
    out += [ball(0.08, "eye", -0.16, 1.35, 0.44, node="head"), ball(0.08, "eye", 0.16, 1.35, 0.44, node="head")]
    return out


def lava_elemental():
    out = [ball(0.8, "lava_rock", 0, 1.3, 0, 1, 1.1, 0.95, far=True), ball(0.45, "lava_glow", 0, 1.3, 0.35, 1, 1, 0.6), ball(0.45, "lava_rock", 0, 2.25, 0.1, node="head", far=True)]
    out += face(2.28, 0.1, 0.45)
    out += [ball(0.36, "lava_glow", -0.9, 1.3, 0.2, node="hand_l"), ball(0.36, "lava_glow", 0.9, 1.3, 0.2, node="hand_r")]
    return out


def whelp():
    out = [ball(0.6, "whelp", 0, 1.0, 0, 0.9, 0.85, 1.3, far=True), ball(0.42, "whelp", 0, 1.55, 0.7, node="head", far=True)]
    out += face(1.6, 0.72, 0.42)
    out.append(part("cone", (0, 190, 60), (12, 30, 12), "horn", "head"))
    out += [ball(0.45, "wing", -0.75, 1.4, -0.1, 1.2, 0.2, 0.9, node="hand_l"), ball(0.45, "wing", 0.75, 1.4, -0.1, 1.2, 0.2, 0.9, node="hand_r")]
    out += [ball(0.2, "whelp", -0.3, 0.25, 0.2, node="foot_l"), ball(0.2, "whelp", 0.3, 0.25, 0.2, node="foot_r")]
    return out


MODELS = {
    "archmage": mage(arch=True, size=1.3),
    "student": mage(size=0.78),
    "penguin": penguin(),
    "golem": golem(),
    "wisp_helper": wisp(),
    "trainer": trainer(),
    "giant": giant(),
    "elf": elf(),
    "skeleton": skeleton(),
    "bone_knight": [dict(p, p=[round(v * 1.25) for v in p["p"]], s=[round(v * 1.25) for v in p["s"]]) for p in skeleton(armour=True)],
    "imp": imp(),
    "troll": troll(),
    "slime": slime(),
    "wisp": wisp("wisp_blue"),
    "boar": boar(),
    "ghost": ghost(),
    "lava_elemental": lava_elemental(),
    "whelp": whelp(),
}

if __name__ == "__main__":
    OUT.write_text(json.dumps({"_about": "Characters as parts: mesh, position and size in cm, tilt in thousandths of a turn, colour role, node, and whether the far view draws it. Written by tools/models.py.", "models": MODELS}, indent=1) + "\n")
    print(f"wrote {OUT} with {len(MODELS)} models, {sum(len(v) for v in MODELS.values())} parts")
