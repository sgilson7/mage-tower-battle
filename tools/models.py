"""Writes data/models.json. The mages are faceted parts with jointed arms
after Ape Race's apes (T34); the enemies and helpers are still spheres and
a few cones and cylinders (entry 1, decision 37), faceted by the mesh.

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
    """A mage of faceted parts after Ape Race's apes (T34): a robe that
    flares to the floor, a chest and belt, arms that turn at the shoulder
    with the hand and cuff on the arm, feet that step, a faceted head with
    eyes and a nose, and a hat with a bent tip. The archmage has a beard
    and a staff whose stone glows; a student has hair and a scarf."""
    out = [
        part("frustum", (0, 55, 0), (112, 100, 112), "robe", far=True),
        part("cyl", (0, 8, 0), (116, 10, 116), "trim"),
        part("cyl", (0, 125, 0), (62, 50, 58), "robe", far=True),
        part("cyl", (0, 103, 0), (64, 9, 60), "trim"),
        part("blob", (0, 168, 0), (46, 44, 46), "skin", "head", far=True),
        part("cube", (0, 165, 22), (10, 10, 10), "skin", "head"),
        part("cube", (-9, 173, 20), (5, 6, 3), "eye", "head"),
        part("cube", (9, 173, 20), (5, 6, 3), "eye", "head"),
        part("frustum", (0, 182, 0), (88, 5, 88), "hat", "head", far=True),
        part("cyl", (0, 189, 0), (46, 10, 46), "trim", "head"),
        part("frustum", (0, 210, 0), (46, 34, 46), "hat", "head"),
        part("cone", (5, 246, -4), (30, 46, 30), "hat", "head", tilt=(-40, -90)),
    ]
    if arch:
        out.append(part("cone", (0, 146, 10), (40, 44, 28), "beard", "head", tilt=(470, 0)))
        out.append(part("gem", (13, 268, -7), (12, 14, 12), "gold", "head"))
    else:
        out.append(part("blob", (0, 176, -5), (50, 38, 50), "hair", "head"))
        out.append(part("cyl", (0, 147, 0), (54, 14, 52), "trim"))
    for sx, node in ((-1, "arm_l"), (1, "arm_r")):
        q = [sx * 33, 142, 0]
        for prt in (
            part("frustum", (sx * 33, 115, 0), (30, 56, 30), "robe", node),
            part("cyl", (sx * 33, 88, 0), (32, 6, 32), "trim", node),
            part("blob", (sx * 33, 79, 0), (17, 17, 17), "skin", node),
        ):
            prt["q"] = q
            out.append(prt)
    for sx, node in ((-1, "foot_l"), (1, "foot_r")):
        out.append(part("chunk", (sx * 16, 5, 10), (16, 10, 28), "boots", node))
    if arch:
        for prt in (
            part("cyl", (33, 125, 8), (7, 200, 7), "wood", "grip_r"),
            part("cyl", (33, 223, 8), (12, 8, 12), "gold", "grip_r"),
            part("gem", (33, 238, 8), (20, 26, 20), "staff_orb", "grip_r"),
        ):
            out.append(prt)
        out[-1]["g"] = 450
    return scale(out, size)


def scale(parts, k):
    for p in parts:
        p["p"] = [round(v * k) for v in p["p"]]
        p["s"] = [round(v * k) for v in p["s"]]
        if "q" in p:
            p["q"] = [round(v * k) for v in p["q"]]
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
