// The page: loads the module and the data, then each display frame passes
// the keys to the core, uploads what it returns to WebGL, replays the 2D
// overlay, plays the sounds it names and redraws the panels that changed.
// It decides nothing (Builder R6).
import init, { Game, numbers, mesh, uniforms, draw_assets } from "./pkg/tool_shim.js";
import { makeRenderer } from "./gl.js";
import { createRenderer } from "./render2d.js";
import { makeKeys } from "./keys.js";
import { makeUi } from "./ui.js";
import { makeSound } from "./sound.js";
import { readSave, writeSave } from "./store.js";

const el = (id) => document.getElementById(id);
const text = async (path) => (await fetch(path)).text();

async function main() {
  await init();
  const files = ["strings.json", "content.json", "palette.json", "models.json", "sounds.json", "glossary.md", "controls.json", "display.json", "shaders/scene.vert", "shaders/scene.frag"];
  const [strings, content, palette, models, sounds, glossary, controls, display, vert, frag] = await Promise.all(files.map((f) => text(`data/${f}`)));
  const seed = new Uint32Array(1);
  crypto.getRandomValues(seed);
  const game = new Game(strings, content, palette, models, sounds, glossary, readSave(), seed[0]);
  const N = JSON.parse(numbers());
  const D = JSON.parse(display);
  const C = JSON.parse(controls);
  const renderer = makeRenderer(el("gl"), N, { vertex: vert, fragment: frag, software: D.software }, (id) => mesh(id), uniforms());
  if (!renderer) el("message").textContent = JSON.parse(strings)["page.no_webgl"];
  const ov = el("overlay");
  const over = createRenderer(ov, JSON.parse(draw_assets()));
  const keys = makeKeys(C.bits, N.actions);
  const sound = makeSound(game, D.volume);
  const S = JSON.parse(strings);
  const message = (t) => (el("message").textContent = t);

  // online play: the transport loads only when a player hosts or joins
  let online = null;
  async function transport() {
    if (!online) {
      const { makeOnline } = await import("./online.js");
      online = makeOnline(game);
    }
    return online;
  }

  // a small box over the screen: a prompt, a text box, and buttons
  function ask(key, value, buttons) {
    const box = el("ask");
    const p = document.createElement("p");
    p.textContent = S[key];
    const area = document.createElement("textarea");
    area.id = "ask-text";
    area.value = value || "";
    const row = document.createElement("div");
    row.className = "actions";
    for (const [label, fn] of [...buttons, [S["page.close"], closeAsk]]) {
      const b = document.createElement("button");
      b.className = "act";
      b.textContent = label;
      b.addEventListener("click", () => fn(area.value, area));
      row.append(b);
    }
    box.replaceChildren(p, area, row);
    box.hidden = false;
    area.focus();
  }
  function closeAsk() {
    el("ask").hidden = true;
  }
  function copy(t) {
    const failed = () => message(S["page.copy_failed"]);
    if (!navigator.clipboard) return failed();
    navigator.clipboard.writeText(t).then(() => message(S["page.copied"]), failed);
  }

  // the page's own actions: the ones that need the transport or the clipboard
  async function page(name, value) {
    if (name === "page.copy") return copy(value);
    const net = await transport();
    if (name === "page.host") net.host();
    if (name === "page.join" && !net.join(value)) message(S["page.bad_code"]);
    if (name === "page.invite") {
      message(S["page.loading"]);
      const invitation = await net.invite();
      message("");
      ask("page.invite_prompt", invitation, [
        [S["page.copy"], () => copy(invitation)],
        [S["page.connect"], async (t) => {
          try {
            await net.accept(t.replace(invitation, "").trim());
            closeAsk();
          } catch (e) {
            message(S["page.signal_failed"]);
          }
        }],
      ]);
    }
    if (name === "page.reply") {
      ask("page.reply_prompt", "", [
        [S["page.make_reply"], async (t) => {
          try {
            const reply = await net.reply(t.trim());
            ask("page.reply_ready", reply, [[S["page.copy"], () => copy(reply)]]);
          } catch (e) {
            message(S["page.signal_failed"]);
          }
        }],
      ]);
    }
  }

  function act(name, value) {
    if (name.startsWith("page.")) return page(name, value);
    const r = JSON.parse(game.act(name, value));
    message(r.error || "");
  }
  const ui = makeUi(el("panels"), el("labels"), el("toasts"), act);

  window.addEventListener("keydown", (e) => {
    const a = C.acts[e.code];
    if (a && !(e.target && (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA"))) act(a[0], a[1]);
  });
  el("mute").addEventListener("click", () => act("sound.mute", ""));

  const point = (name, e) => act(name, [e.offsetX, e.offsetY, ov.clientWidth, ov.clientHeight].join(","));
  ov.addEventListener("pointerdown", (e) => {
    try {
      ov.setPointerCapture(e.pointerId);
    } catch (err) {
      // a refused capture still draws while the pointer stays on the board
    }
    point("pointer.down", e);
  });
  ov.addEventListener("pointermove", (e) => point("pointer.move", e));
  ov.addEventListener("pointerup", (e) => point("pointer.up", e));
  ov.addEventListener("pointercancel", (e) => point("pointer.up", e));

  function resize() {
    const stage = el("stage");
    const [w, h] = game.resize(stage.clientWidth, stage.clientHeight, window.devicePixelRatio || 1);
    if (renderer) renderer.resize(w, h);
  }
  resize();
  window.addEventListener("resize", resize);

  let version = null;
  let last = "";
  function frame(now) {
    game.frame(now, keys.read());
    if (renderer) {
      const v = game.static_version();
      if (v !== version) {
        renderer.setStatic(game.static_values());
        version = v;
      }
      renderer.frame(game.views(), game.values(), game.commands());
    }
    const o = game.overlay();
    ov.hidden = o.length === 0;
    if (o.length) over.draw(o);
    sound.play(game.sounds());
    const u = game.ui();
    if (u !== last) {
      last = u;
      const view = JSON.parse(u);
      ui.draw(view);
      el("mute").textContent = view.mute_label;
    }
    if (online) {
      online.pump();
      document.body.dataset.net = game.net_status();
    }
    const save = game.take_save();
    if (save) writeSave(save);
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
  document.body.dataset.ready = "true";
}

main();
