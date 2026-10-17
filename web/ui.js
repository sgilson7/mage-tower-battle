// Builds the page's panels from the core's view model (ui()) without
// knowing what any panel means (after Peter's Mandible). A panel is redrawn
// only when its description changes. Labels over the 3D view arrive with
// their positions already worked out by the core, as CSS text.
export function makeUi(panelsEl, labelsEl, toastsEl, cardEl, act) {
  const drawn = new Map();
  let shownCard = "";
  let hints = [];

  const make = (tag, cls, text) => {
    const node = document.createElement(tag);
    if (cls) node.className = cls;
    if (text !== undefined && text !== null && text !== "") node.textContent = text;
    return node;
  };

  // a heading or label that is a glossary term shows its definition on hover
  function mark(node) {
    const hint = hints.find((h) => h.term === node.textContent);
    if (hint) {
      node.dataset.term = hint.term;
      node.title = hint.definition;
    }
  }

  // a glyph as an icon: fixed markup parsed as HTML, which places <svg> in
  // its namespace without naming it; the core's values go in as attributes
  function iconOf(d, view, cls) {
    const holder = make("div");
    holder.innerHTML = "<svg aria-hidden=\"true\"><path></path></svg>";
    const svg = holder.firstChild;
    svg.setAttribute("class", cls);
    svg.setAttribute("viewBox", view);
    svg.firstChild.setAttribute("d", d);
    return svg;
  }

  function button(a) {
    const b = make("button", a.on ? "act on" : "act", a.label);
    if (a.num) b.prepend(make("i", "num", a.num));
    if (a.icon) b.prepend(iconOf(a.icon, a.view, "act-icon"));
    if (a.hint) b.title = a.hint;
    if (a.mana) b.append(manaOf(a.mana));
    if (a.badge) b.append(make("span", "badge", a.badge));
    b.disabled = !a.enabled;
    b.dataset.action = a.name;
    b.dataset.value = a.value;
    grabbing(b, a);
    return b;
  }

  // drag and drop (T32): a button with a grab acts the moment it is
  // pressed; letting go over a button marked drop acts that button
  function grabbing(b, a) {
    if (a.grab) {
      b.dataset.grabAction = a.grab.name;
      b.dataset.grabValue = a.grab.value;
    }
    if (a.drop) b.dataset.drop = "1";
  }

  function buttons(list, cls) {
    const box = make("div", cls);
    for (const a of list || []) box.append(button(a));
    return box;
  }

  // a mana cost: the mana symbol and its number (T13)
  function manaOf(m) {
    const span = make("span", "mana", m.n);
    span.title = m.label;
    span.setAttribute("aria-label", m.label);
    return span;
  }

  function rowOf(r) {
    const li = make("li", `${r.school ? `row school-${r.school}` : "row"}${r.on ? " on" : ""}${r.dim ? " dim" : ""}`);
    if (r.press) {
      const b = button(r.press);
      b.className = `act row-press${r.on ? " on" : ""}`;
      b.textContent = "";
      li.append(b);
      fillRow(b, r);
      return li;
    }
    fillRow(li, r);
    return li;
  }

  // a small ? that shows a panel's instructions on hover (T49)
  function helpOf(text) {
    const q = make("span", "help", "?");
    q.title = text;
    q.tabIndex = 0;
    q.setAttribute("aria-label", text);
    return q;
  }

  function fillRow(li, r) {
    if (r.swatch) li.append(make("span", `sw sw-${r.swatch}`));
    const text = make("div", "row-text");
    const label = make("span", "row-label", r.label);
    mark(label);
    text.append(label);
    if (r.detail) text.append(make("span", "row-detail", r.detail));
    if (r.list) {
      const ul = make("ul", "row-list");
      for (const item of r.list) ul.append(make("li", "", item));
      text.append(ul);
    }
    li.append(text);
    if (r.bar) {
      const bar = make("span", "bar");
      const fill = make("i");
      fill.style.width = r.bar;
      bar.append(fill);
      li.append(bar);
    }
    if (r.mana) li.append(manaOf(r.mana));
    if (r.chip) li.append(make("span", r.chip_tone ? `chip tone-${r.chip_tone}` : "chip", r.chip));
    if (r.actions) li.append(buttons(r.actions, "row-actions"));
  }

  // a bar with a label and a number; a leaning bar fills from the middle
  function metersOf(list) {
    const box = make("div", "meters");
    for (const m of list) {
      const label = make("span", "m-label", m.label);
      mark(label);
      const track = make("span", m.centre ? "track centre" : "track");
      const fill = make("i", m.tone ? `tone-${m.tone}` : "");
      fill.style.left = m.left;
      fill.style.width = m.width;
      track.append(fill);
      box.append(label, track, make("b", "m-value", m.value));
    }
    return box;
  }

  function cellOf(c) {
    const td = make("td");
    if (c.mana) td.append(manaOf(c.mana));
    else if (c.b) td.append(make("b", "", c.b));
    else td.textContent = c.text;
    return td;
  }

  function tableOf(t) {
    const table = make("table", "table");
    const head = make("tr");
    for (const h of t.head) head.append(make("th", "", h));
    table.append(head);
    for (const r of t.rows) {
      const tr = make("tr");
      for (const c of r) tr.append(cellOf(c));
      table.append(tr);
    }
    return table;
  }

  // a picture of the field made of boxes the core placed, in percent
  function boardOf(b) {
    const box = make("div", "board");
    box.style.aspectRatio = b.aspect;
    for (const it of b.items) {
      const node = it.action ? button(it.action) : make("div", "", it.text);
      node.classList.add("bi", `bi-${it.kind}`);
      if (it.on) node.classList.add("on");
      if (it.tone) node.classList.add(it.tone);
      if (it.action) node.textContent = it.text || "";
      if (it.hint) node.title = it.hint;
      node.style.left = it.left;
      node.style.top = it.top;
      node.style.width = it.w;
      node.style.height = it.h;
      box.append(node);
    }
    return box;
  }

  function keysOf(parts) {
    const p = make("p", "keys");
    for (const k of parts) p.append(k.key ? make("kbd", "", k.key) : document.createTextNode(k.text));
    return p;
  }

  function noteOf(b) {
    const d = make("div", "note");
    if (b.lead) d.append(make("b", "", b.lead), document.createTextNode(" "));
    d.append(document.createTextNode(b.text));
    return d;
  }

  function progressOf(b) {
    const d = make("div", "progress");
    const top = make("div", "progress-top");
    top.append(make("b", "", b.label), make("b", "", b.value));
    const track = make("span", "track");
    const fill = make("i", "tone-good");
    fill.style.width = b.width;
    track.append(fill);
    d.append(top, track);
    return d;
  }

  function chipsOf(list) {
    const d = make("div", "chip-row");
    for (const c of list) d.append(make("span", c.tone ? `chip tone-${c.tone}` : "chip", c.text));
    return d;
  }

  function bigOf(b) {
    const p = make("p", "big", b.text);
    if (b.small) p.append(make("small", "", b.small));
    return p;
  }

  function pileOf(b) {
    const d = make("div", "pile");
    d.append(make("b", "", b.label));
    const books = make("div", "pile-books");
    for (const k of b.books) books.append(make("span", `pile-book school-${k}`));
    d.append(books);
    return d;
  }

  // the sorting game's shelves: a button each, with the books shelved on it
  function shelvesOf(list) {
    const d = make("div", "shelves");
    for (const s of list) {
      const b = button(s.action);
      b.className = `act shelf school-${s.school}${s.on ? " on" : ""}`;
      b.textContent = "";
      const spines = make("span", "spines");
      for (const h of s.spines) {
        const sp = make("span", "spine");
        sp.style.height = h;
        spines.append(sp);
      }
      b.append(spines, make("span", "shelf-name", s.label), make("kbd", "", s.key));
      d.append(b);
    }
    return d;
  }

  const blocks = {
    title: (b) => {
      const h = make("h2", "", b.text);
      mark(h);
      if (b.help) h.append(helpOf(b.help));
      return h;
    },
    h: (b) => {
      const h = make("h3", "", b.text);
      if (b.help) h.append(helpOf(b.help));
      return h;
    },
    p: (b) => make("p", b.tone || "", b.text),
    note: noteOf,
    rows: (b) => {
      const ul = make("ul", "rows");
      for (const r of b.rows) ul.append(rowOf(r));
      return ul;
    },
    meters: (b) => metersOf(b.rows),
    table: tableOf,
    board: boardOf,
    actions: (b) => buttons(b.actions, b.style || "actions"),
    keys: (b) => keysOf(b.parts),
    progress: progressOf,
    chips: (b) => chipsOf(b.chips),
    big: bigOf,
    mana: (b) => {
      const p = make("p", "panel-mana");
      p.append(manaOf(b.mana));
      return p;
    },
    pile: pileOf,
    shelves: (b) => shelvesOf(b.shelves),
    lane: (b) => laneOf(b.lane),
    tray: (b) => trayOf(b.tray),
    // a numbered list, each item led by a bold name (T38)
    ol: (b) => {
      const ol = make("ol", "steps-list");
      for (const it of b.items) {
        const li = make("li");
        li.append(make("b", "", it.lead), document.createTextNode(" "), document.createTextNode(it.text));
        ol.append(li);
      }
      return ol;
    },
    // a tree read top to bottom (T41): each line indented as the core
    // says, with what it answered
    lines: (b) => {
      const box = make("div", "tree-lines");
      for (const l of b.lines) {
        const line = make("div", l.tone ? `tree-line ${l.tone}` : "tree-line");
        line.style.paddingLeft = l.pad;
        line.append(make("span", "tree-mark", l.mark), make("span", "", l.text));
        box.append(line);
      }
      return box;
    },
  };

  // a panel's blocks, gathered into the columns they name, in the order
  // each column first appears
  function columnsOf(list) {
    const cols = new Map();
    for (const b of list) {
      if (!cols.has(b.col)) cols.set(b.col, make("div", `col col-${b.col}`));
      cols.get(b.col).append(blocks[b.t](b));
    }
    return [...cols.values()];
  }

  // a spell piece's card (T10): its glyph, name, number, school and kind,
  // what it does and its numbers; the core sends every part
  // the icon is fixed markup parsed as HTML, which places <svg> in its
  // namespace without naming it; the core's values go in as attributes
  function cardOf(c) {
    const box = make("article", `card school-${c.school}`);
    const svg = iconOf(c.icon, c.view, "card-icon");
    const body = make("div", "card-body");
    const top = make("div", "card-top");
    top.append(make("span", "card-number", c.number), make("span", "card-kind", c.kind));
    if (c.chip) top.append(make("span", "chip", c.chip));
    if (c.mana) top.append(manaOf(c.mana));
    body.append(top, make("h3", "card-name", c.name), make("p", "card-text", c.text), make("p", "card-facts", c.facts));
    box.append(svg, body);
    return box;
  }

  // the spell builder (T17): rows of sets, each modifiers, a payload and a
  // trigger, left to right; a placed piece is a button that takes it out
  function slotOf(t, kind, label, caption) {
    if (!t) {
      // an open slot takes a tile dropped on it
      const b = button({ name: "lab.drop", value: kind, label: "", enabled: true, drop: true });
      b.className = `act slot ${kind} empty`;
      b.append(make("small", "", caption), make("span", "slot-name", label));
      return b;
    }
    const b = make("button", `act slot ${kind} filled school-${t.school}`);
    b.dataset.action = "lab.remove";
    b.dataset.value = t.i;
    b.append(make("small", "", caption));
    if (kind !== "mod") b.append(iconOf(t.icon, t.view, "slot-icon"));
    b.append(make("span", "slot-name", t.name));
    // what the piece does, in its words and its numbers (T38)
    if (t.text) b.append(make("span", "slot-does", t.text));
    if (t.facts) b.append(make("span", "slot-facts", t.facts));
    if (t.mana) b.append(manaOf(t.mana));
    return b;
  }

  function laneOf(lane) {
    const box = make("div", "lane");
    const L = lane.labels;
    const head = make("div", "slot spell");
    head.append(make("small", "", L.your), make("b", "", lane.spell));
    if (lane.mana) head.append(manaOf(lane.mana));
    lane.rows.forEach((row, r) => {
      const line = make("div", "lane-row");
      if (r === 0) line.append(head);
      for (const set of row) {
        if (set.branch) {
          line.append(make("div", "branch-from", L.branch));
          continue;
        }
        const s = make("div", set.open ? "set open" : "set");
        s.append(make("span", "set-label", set.label));
        const mods = make("div", "mods");
        for (const m of set.mods) mods.append(slotOf(m, "mod", "", L.mods));
        if (set.open && !set.payload) mods.append(slotOf(null, "mod", L.mod, L.mods));
        s.append(mods);
        if (set.payload || set.open) s.append(slotOf(set.payload, "pay", L.pay, L.payload));
        if (set.trigger || (set.open && set.payload)) s.append(slotOf(set.trigger, "trig", L.trig, L.trigger));
        line.append(s);
      }
      box.append(line);
    });
    return box;
  }

  // the pieces to add (T17, T32): a tab for each kind, the tiles of the
  // open tab, and a blank for each piece of that kind not read yet
  function trayOf(tray) {
    const box = make("div", "tray");
    box.append(buttons(tray.tabs, "tabs"));
    const tiles = make("div", "tiles");
    for (const t of tray.tiles) {
      const b = button({ name: "lab.add", value: t.id, label: "", enabled: true, grab: { name: "lab.hold", value: t.id } });
      b.className = `act tile ${t.kind} school-${t.school}`;
      b.append(iconOf(t.icon, t.view, "slot-icon"), make("span", "slot-name", t.name));
      if (t.mana) b.append(manaOf(t.mana));
      if (t.text) b.append(make("span", "slot-does", t.text));
      if (t.facts) b.append(make("span", "slot-facts", t.facts));
      tiles.append(b);
    }
    if (tray.unread_label) tiles.append(make("div", `tile ${tray.kind} unread`, tray.unread_label));
    box.append(tiles);
    return box;
  }

  function panel(p) {
    const box = make("section", `panel panel-${p.kind}`);
    box.id = p.id;
    if (p.box) {
      box.style.left = p.box.left;
      box.style.top = p.box.top;
      box.style.width = p.box.width;
      box.style.height = p.box.height;
    }
    if (p.blocks) {
      box.append(...columnsOf(p.blocks));
      return box;
    }
    if (p.items) {
      for (const it of p.items) {
        const cell = make("div", `hud-item ${it.kind || ""}`);
        cell.append(make("span", "hud-label", it.label), make("span", "hud-value", it.value));
        if (it.hint) cell.title = it.hint;
        // the rehearsal's change since the last plan (T48)
        if (it.note) cell.append(make("span", it.tone ? `hud-note tone-${it.tone}` : "hud-note", it.note));
        if (it.bar) {
          const bar = make("span", "bar");
          const fill = make("i");
          fill.style.width = it.bar;
          bar.append(fill);
          // the part of the maximum the field holds (T61), at the bar's end
          if (it.held) {
            const held = make("b", "held");
            held.style.width = it.held;
            bar.append(held);
          }
          cell.append(bar);
        }
        box.append(cell);
      }
    }
    if (p.heading) {
      const h = make("h2", "", p.heading);
      mark(h);
      // the panel's instructions, on hover rather than on the panel (T49)
      if (p.help) h.append(helpOf(p.help));
      box.append(h);
    }
    if (p.text) box.append(make("p", "", p.text));
    if (p.mana) {
      const cost = make("p", "panel-mana");
      cost.append(manaOf(p.mana));
      box.append(cost);
    }
    for (const f of p.inputs || []) {
      const label = make("label", "field", f.label);
      const input = make("input");
      input.id = f.id;
      input.value = f.value;
      // each keystroke reaches the core, so nothing is left to send when a
      // button is pressed (a redraw between press and release loses a click)
      input.addEventListener("input", () => act(f.action, input.value));
      label.append(input);
      box.append(label);
    }
    if (p.lane) box.append(laneOf(p.lane));
    else if (p.draft && p.draft.length) box.append(buttons(p.draft, "chain"));
    if (p.actions && p.actions.length) box.append(buttons(p.actions, "actions"));
    if (p.saved_actions && p.saved_actions.length) box.append(buttons(p.saved_actions, "chips"));
    if (p.saved && p.saved.length) {
      const ul = make("ul", "rows saved");
      for (const r of p.saved) ul.append(rowOf(r));
      box.append(ul);
    }
    if (p.rows && p.rows.length) {
      const ul = make("ul", "rows");
      for (const r of p.rows) ul.append(rowOf(r));
      box.append(ul);
    }
    if (p.lesson) {
      // what each student got from the last lesson (T48)
      const card = make("div", "lesson");
      card.append(make("h3", "", p.lesson.heading));
      const ul = make("ul", "rows");
      for (const r of p.lesson.rows) ul.append(rowOf(r));
      card.append(ul);
      box.append(card);
    }
    if (p.tray) box.append(trayOf(p.tray));
    if (p.cards && p.cards.length) {
      const grid = make("div", "cards");
      for (const c of p.cards) grid.append(cardOf(c));
      box.append(grid);
    }
    return box;
  }

  // A press and a release on the same action count as a click, even when
  // the panel was redrawn between them: a panel that changes every frame
  // replaced its buttons under the pointer, and the browser's own click
  // never came (TRIAGE T9). A key press on a focused button clicks with
  // detail 0.
  const target = (e) => {
    const b = e.target.closest && e.target.closest("button.act");
    return b && !b.disabled ? b : null;
  };
  let pressed = null;
  // what you picked up, drawn in your hand under the pointer until you let
  // go (T53): a copy of the book, tile or student, which takes no clicks
  let ghost = null;
  const follow = (e) => {
    if (!ghost) return;
    ghost.style.left = `${e.clientX}px`;
    ghost.style.top = `${e.clientY}px`;
  };
  const dropGhost = () => {
    if (ghost) ghost.remove();
    ghost = null;
  };
  panelsEl.addEventListener("pointerdown", (e) => {
    const b = target(e);
    pressed = b ? { action: b.dataset.action, value: b.dataset.value, grab: b.dataset.grabAction } : null;
    if (b && b.dataset.grabAction) {
      // the pointer stays free, so letting go over another button is seen there
      if (b.hasPointerCapture && b.hasPointerCapture(e.pointerId)) b.releasePointerCapture(e.pointerId);
      act(b.dataset.grabAction, b.dataset.grabValue);
      dropGhost();
      ghost = b.cloneNode(true);
      ghost.classList.add("in-hand");
      ghost.removeAttribute("data-action");
      ghost.style.width = `${b.offsetWidth}px`;
      ghost.style.height = `${b.offsetHeight}px`;
      document.body.append(ghost);
      follow(e);
    }
    document.body.dataset.dragging = b && b.dataset.grabAction ? "on" : "off";
  });
  window.addEventListener("pointermove", follow);
  window.addEventListener("pointerup", dropGhost);
  window.addEventListener("pointercancel", dropGhost);
  panelsEl.addEventListener("pointerup", (e) => {
    const b = target(e);
    if (pressed && b && b.dataset.action === pressed.action && b.dataset.value === pressed.value) act(pressed.action, pressed.value);
    else if (pressed && pressed.grab && b && b.dataset.drop) act(b.dataset.action, b.dataset.value);
    if (pressed && pressed.grab) act("drag.end", "");
    pressed = null;
    document.body.dataset.dragging = "off";
  });
  panelsEl.addEventListener("click", (e) => {
    const b = target(e);
    if (b && e.detail === 0) act(b.dataset.action, b.dataset.value);
  });

  return {
    draw(ui) {
      hints = ui.hints || [];
      const seen = new Set();
      for (const p of ui.panels) {
        const key = JSON.stringify(p);
        seen.add(p.id);
        const old = drawn.get(p.id);
        if (old && old.key === key) continue;
        const node = panel(p);
        // a redrawn text box keeps the focus and the caret it had
        const focused = old && old.node.contains(document.activeElement) ? document.activeElement : null;
        if (old) old.node.replaceWith(node);
        const again = focused && focused.id ? node.querySelector(`#${focused.id}`) : null;
        if (again) {
          again.focus();
          again.setSelectionRange(focused.selectionStart, focused.selectionEnd);
        }
        else panelsEl.append(node);
        drawn.set(p.id, { key, node });
      }
      for (const [id, old] of drawn) {
        if (!seen.has(id)) {
          old.node.remove();
          drawn.delete(id);
        }
      }
      labelsEl.replaceChildren(
        ...(ui.labels || []).map((l) => {
          const d = make("div", `label3d ${l.kind}`, l.text);
          d.style.left = l.left;
          d.style.top = l.top;
          return d;
        }),
      );
      toastsEl.replaceChildren(...(ui.toasts || []).map((t) => make("div", "toast", t)));
      const cardKey = JSON.stringify(ui.card || null);
      if (cardKey !== shownCard) {
        shownCard = cardKey;
        cardEl.replaceChildren(...(ui.card ? [cardOf(ui.card)] : []));
      }
      const body = document.body.dataset;
      body.screen = ui.screen;
      body.phase = ui.phase || "";
      body.station = ui.station || "";
      body.busy = ui.busy || "";
      body.overlay = ui.overlay ? "on" : "off";
      for (const [k, v] of Object.entries(ui.data || {})) body[k] = v;
      document.title = ui.title;
    },
  };
}
