// Keys to the action bits the core defines (numbers().actions), after the
// realtime kit's input.js without its pointer lock: this game never locks
// the mouse. A key pressed and let go between two frames still reaches the
// core once (`tapped`). Typing in a text box sets nothing.
export function makeKeys(bindings, actions) {
  const state = { held: 0, tapped: 0 };
  const bitOf = (code) => actions[bindings[code]] || 0;
  const typing = (e) => e.target && (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA");

  window.addEventListener("keydown", (e) => {
    if (typing(e) || e.repeat) return;
    const bit = bitOf(e.code);
    if (!bit) return;
    state.held |= bit;
    state.tapped |= bit;
    e.preventDefault();
  });
  window.addEventListener("keyup", (e) => {
    state.held &= ~bitOf(e.code);
  });
  window.addEventListener("blur", () => {
    state.held = 0;
  });

  return {
    read() {
      const bits = state.held | state.tapped;
      state.tapped = 0;
      return bits;
    },
  };
}
