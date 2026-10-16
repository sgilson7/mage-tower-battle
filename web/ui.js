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

  function button(a) {
    const b = make("button", "act", a.label);
    b.disabled = !a.enabled;
    b.dataset.action = a.name;
    b.dataset.value = a.value;
    return b;
  }

  function buttons(list, cls) {
    const box = make("div", cls);
    for (const a of list || []) box.append(button(a));
    return box;
  }

  function rowOf(r) {
    const li = make("li", r.school ? `row school-${r.school}` : "row");
    const text = make("div", "row-text");
    const label = make("span", "row-label", r.label);
    mark(label);
    text.append(label);
    if (r.detail) text.append(make("span", "row-detail", r.detail));
    li.append(text);
    if (r.bar) {
      const bar = make("span", "bar");
      const fill = make("i");
      fill.style.width = r.bar;
      bar.append(fill);
      li.append(bar);
    }
    if (r.chip) li.append(make("span", "chip", r.chip));
    if (r.actions) li.append(buttons(r.actions, "row-actions"));
    return li;
  }

  // a spell piece's card (T10): its glyph, name, number, school and kind,
  // what it does and its numbers; the core sends every part
  // the icon is fixed markup parsed as HTML, which places <svg> in its
  // namespace without naming it; the core's values go in as attributes
  function cardOf(c) {
    const box = make("article", `card school-${c.school}`);
    const holder = make("div");
    holder.innerHTML = "<svg class=\"card-icon\" aria-hidden=\"true\"><path></path></svg>";
    const svg = holder.firstChild;
    svg.setAttribute("viewBox", c.view);
    svg.firstChild.setAttribute("d", c.icon);
    const body = make("div", "card-body");
    const top = make("div", "card-top");
    top.append(make("span", "card-number", c.number), make("span", "card-kind", c.kind));
    if (c.chip) top.append(make("span", "chip", c.chip));
    body.append(top, make("h3", "card-name", c.name), make("p", "card-text", c.text), make("p", "card-facts", c.facts));
    box.append(svg, body);
    return box;
  }

  function panel(p) {
    const box = make("section", `panel panel-${p.kind}`);
    box.id = p.id;
    if (p.items) {
      for (const it of p.items) {
        const cell = make("div", `hud-item ${it.kind || ""}`);
        cell.append(make("span", "hud-label", it.label), make("span", "hud-value", it.value));
        if (it.bar) {
          const bar = make("span", "bar");
          const fill = make("i");
          fill.style.width = it.bar;
          bar.append(fill);
          cell.append(bar);
        }
        box.append(cell);
      }
    }
    if (p.heading) {
      const h = make("h2", "", p.heading);
      mark(h);
      box.append(h);
    }
    if (p.text) box.append(make("p", "", p.text));
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
    if (p.draft && p.draft.length) box.append(buttons(p.draft, "chain"));
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
  panelsEl.addEventListener("pointerdown", (e) => {
    const b = target(e);
    pressed = b ? [b.dataset.action, b.dataset.value] : null;
  });
  panelsEl.addEventListener("pointerup", (e) => {
    const b = target(e);
    if (pressed && b && b.dataset.action === pressed[0] && b.dataset.value === pressed[1]) act(pressed[0], pressed[1]);
    pressed = null;
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
