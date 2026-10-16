//! Chip sounds in the manner of early NES games (the person: "basic
//! satisfying chip noises based on early nes games for all sounds",
//! SECOND-ORDER R3.4). Four voices like the console's: two pulse waves
//! with a duty of 12.5, 25 or 50%, a stepped triangle, and noise from a
//! 15-bit shift register. Each sound is a recipe in `data/sounds.json`:
//! a few notes, each a voice sweeping from one pitch to another with a
//! falling volume. The core renders every recipe to 8-bit WAV bytes once;
//! the page decodes them and plays the ones the core names each frame.

use serde::Deserialize;

pub const RATE: i32 = 22_050;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    /// "pulse12", "pulse25", "pulse50", "triangle" or "noise"
    pub voice: String,
    /// Start time and length in ms.
    pub at: i32,
    pub ms: i32,
    /// Pitch in Hz at the start and end; for noise, the shift rate.
    pub from: i32,
    pub to: i32,
    /// Peak volume 0–15, as the console's envelope.
    pub vol: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub id: String,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SoundFile {
    pub sounds: Vec<Recipe>,
}

pub fn parse(json: &str) -> Result<Vec<Recipe>, String> {
    let f: SoundFile =
        serde_json::from_str(json).map_err(|e| format!("The sounds file could not be read: {e}"))?;
    for r in &f.sounds {
        for n in &r.notes {
            if !["pulse12", "pulse25", "pulse50", "triangle", "noise"].contains(&n.voice.as_str()) {
                return Err(format!(
                    "The sound \"{}\" uses a voice \"{}\" the chip does not have.",
                    r.id, n.voice
                ));
            }
            if n.ms <= 0 || n.from <= 0 || n.to <= 0 || !(0..=15).contains(&n.vol) {
                return Err(format!("The sound \"{}\" has a note out of range.", r.id));
            }
        }
    }
    Ok(f.sounds)
}

/// The NES triangle's 32 steps: 15 down to 0 and back up.
fn triangle(step: u32) -> i32 {
    let s = (step % 32) as i32;
    if s < 16 {
        15 - s
    } else {
        s - 16
    }
}

/// Renders a recipe to signed samples in −128..=127 at `RATE`.
pub fn samples(r: &Recipe) -> Vec<i32> {
    let len = r
        .notes
        .iter()
        .map(|n| (n.at + n.ms) as i64 * RATE as i64 / 1000)
        .max()
        .unwrap_or(0) as usize;
    let mut mix = vec![0i32; len];
    for n in &r.notes {
        let start = (n.at as i64 * RATE as i64 / 1000) as usize;
        let count = (n.ms as i64 * RATE as i64 / 1000) as usize;
        // phase in 1/2^16 of a cycle
        let mut phase: u64 = 0;
        let mut lfsr: u32 = 1;
        let mut noise_acc: u64 = 0;
        let mut noise_out = 1;
        for k in 0..count {
            let hz = n.from as i64 + (n.to as i64 - n.from as i64) * k as i64 / count.max(1) as i64;
            let step = (hz.max(1) as u64) << 16;
            // the console's envelope falls in sixteen steps
            let vol = n.vol * (16 - (k * 16 / count.max(1)) as i32) / 16;
            let level = match n.voice.as_str() {
                "noise" => {
                    noise_acc += step / RATE as u64;
                    while noise_acc >= 1 << 16 {
                        noise_acc -= 1 << 16;
                        let bit = (lfsr ^ (lfsr >> 1)) & 1;
                        lfsr = (lfsr >> 1) | (bit << 14);
                        noise_out = if lfsr & 1 == 0 { 1 } else { -1 };
                    }
                    noise_out * vol
                }
                "triangle" => (triangle(((phase * 32) >> 16) as u32) * 2 - 15) * n.vol / 2,
                v => {
                    let duty: u64 = match v {
                        "pulse12" => 1,
                        "pulse25" => 2,
                        _ => 4,
                    };
                    let eighth = (phase * 8) >> 16;
                    if eighth < duty {
                        vol
                    } else {
                        -vol
                    }
                }
            };
            phase = (phase + step / RATE as u64) & 0xffff;
            if let Some(m) = mix.get_mut(start + k) {
                *m += level * 3;
            }
        }
    }
    mix.into_iter().map(|v| v.clamp(-128, 127)).collect()
}

/// An 8-bit mono WAV file of the recipe.
pub fn wav(r: &Recipe) -> Vec<u8> {
    let s = samples(r);
    let data = s.len() as u32;
    let mut out = Vec::with_capacity(44 + s.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(RATE as u32).to_le_bytes());
    out.extend_from_slice(&(RATE as u32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&8u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    out.extend(s.into_iter().map(|v| (v + 128) as u8));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(voice: &str, hz: i32) -> Recipe {
        Recipe {
            id: "t".into(),
            notes: vec![Note {
                voice: voice.into(),
                at: 0,
                ms: 100,
                from: hz,
                to: hz,
                vol: 15,
            }],
        }
    }

    #[test]
    fn a_pulse_spends_its_duty_high_and_the_rest_low() {
        // 441 Hz at 22,050 Hz is 50 samples a cycle; 25% duty: the first 12 or 13 high
        let s = samples(&note("pulse25", 441));
        assert_eq!(s.len(), 2205);
        let high = s[..50].iter().filter(|&&v| v > 0).count();
        assert!((12..=13).contains(&high), "{high}");
        // the phase step truncates to 441 × 65536 / 22050 = 1310, so sample 25
        // is at 32,750, still under half a cycle (32,768): 26 high
        let half = samples(&note("pulse50", 441));
        assert_eq!(half[..50].iter().filter(|&&v| v > 0).count(), 26);
    }

    #[test]
    fn the_triangle_steps_and_noise_is_the_same_every_time() {
        assert_eq!((0..32).map(triangle).collect::<Vec<_>>()[..4], [15, 14, 13, 12]);
        assert_eq!(triangle(16), 0);
        let a = samples(&note("noise", 8000));
        assert_eq!(a, samples(&note("noise", 8000)));
        assert!(a.iter().any(|&v| v > 0) && a.iter().any(|&v| v < 0));
    }

    #[test]
    fn a_wav_has_its_header_and_one_byte_a_sample() {
        let w = wav(&note("pulse50", 441));
        assert_eq!(&w[..4], b"RIFF");
        assert_eq!(&w[8..16], b"WAVEfmt ");
        assert_eq!(w.len(), 44 + 2205);
        assert_eq!(u32::from_le_bytes([w[24], w[25], w[26], w[27]]), RATE as u32);
    }

    #[test]
    fn the_shipped_sounds_parse_and_a_bad_voice_is_refused() {
        let s = parse(include_str!("../../../data/sounds.json")).unwrap();
        assert!(s.len() >= 20);
        let bad = include_str!("../../../data/sounds.json").replacen("\"pulse50\"", "\"kazoo\"", 1);
        assert!(parse(&bad).unwrap_err().contains("kazoo"));
    }
}
