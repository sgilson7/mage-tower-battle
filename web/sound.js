// Plays the chip sounds the core renders (crates/core/src/sound.rs). The
// core sends each sound once as WAV bytes and names which to play each
// frame; this file decodes and plays them and decides nothing. Audio
// starts on the first key or click, as browsers require.
export function makeSound(game, volume) {
  let ctx = null;
  let gain = null;
  const buffers = [];

  function wake() {
    if (ctx) return;
    try {
      ctx = new AudioContext();
    } catch (e) {
      return;
    }
    gain = ctx.createGain();
    gain.gain.value = volume;
    gain.connect(ctx.destination);
    const n = game.sound_count();
    for (let i = 0; i < n; i += 1) {
      const bytes = game.sound_wav(i);
      ctx.decodeAudioData(bytes.buffer).then((b) => {
        buffers[i] = b;
      }, () => {});
    }
  }
  window.addEventListener("keydown", wake);
  window.addEventListener("pointerdown", wake);

  return {
    play(ids) {
      if (!ctx) return;
      for (const id of ids) {
        const b = buffers[id];
        if (!b) continue;
        const source = ctx.createBufferSource();
        source.buffer = b;
        source.connect(gain);
        source.start();
      }
    },
  };
}
