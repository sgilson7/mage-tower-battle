// Builds the page's panels from the core's view model (ui()) without
// knowing what any panel means (after Peter's Mandible). A panel is redrawn
// only when its description changes. Labels over the 3D view arrive with
// their positions already worked out by the core, as CSS text.
export function makeUi(panelsEl, labelsEl, toastsEl, act) {
  const drawn = new Map();
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
    b.addEventListener("click", () => act(a.name, a.value));
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
    return box;
  }

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
        if (old) old.node.replaceWith(node);
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
