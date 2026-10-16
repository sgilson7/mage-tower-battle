//! The interview: a short note lane. Notes come down three lanes (keys J,
//! K and L) and the last is the closing spell (Space). A press within
//! `perfect` ticks of a note scores 100, within `good` ticks 60; a miss
//! scores 0 and a stray press costs 10. The score is the average, 0–100.
//! A group interview has more notes and tighter windows.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

pub const CAST_LANE: u8 = 3;
/// Ticks between the lane's start and its first note, so it can be read.
pub const LEAD: i32 = 90;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub at: i32,
    pub lane: u8,
    /// The score this note earned, once hit or missed.
    pub got: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lane {
    pub notes: Vec<Note>,
    pub tick: i32,
    pub strays: i32,
    pub perfect: i32,
    pub good: i32,
}

impl Lane {
    /// A lane for one applicant (`group` false) or many: eight notes a
    /// beat apart or twelve, half a beat tighter, then the closing spell.
    pub fn new(rng: &mut Rng, group: bool, perfect: i32, good: i32) -> Lane {
        let (n, beat) = if group { (12, 24) } else { (8, 30) };
        let mut notes = Vec::new();
        let mut at = LEAD;
        for _ in 0..n {
            notes.push(Note {
                at,
                lane: rng.below(3) as u8,
                got: None,
            });
            at += beat - if rng.chance(1, 4) { beat / 2 } else { 0 };
        }
        notes.push(Note {
            at: at + beat,
            lane: CAST_LANE,
            got: None,
        });
        let (perfect, good) = if group {
            (perfect * 3 / 4, good * 3 / 4)
        } else {
            (perfect, good)
        };
        Lane {
            notes,
            tick: 0,
            strays: 0,
            perfect,
            good,
        }
    }

    pub fn end(&self) -> i32 {
        self.notes.last().map(|n| n.at).unwrap_or(0) + self.good + 1
    }

    pub fn done(&self) -> bool {
        self.tick > self.end()
    }

    /// Advances a tick; notes left behind the window are marked missed.
    pub fn step(&mut self) {
        self.tick += 1;
        let (t, good) = (self.tick, self.good);
        for n in self.notes.iter_mut() {
            if n.got.is_none() && t > n.at + good {
                n.got = Some(0);
            }
        }
    }

    /// A key press in a lane at the current tick.
    pub fn press(&mut self, lane: u8) {
        let t = self.tick;
        let best = self
            .notes
            .iter_mut()
            .filter(|n| n.got.is_none() && n.lane == lane && (n.at - t).abs() <= self.good)
            .min_by_key(|n| (n.at - t).abs());
        match best {
            Some(n) => n.got = Some(if (n.at - t).abs() <= self.perfect { 100 } else { 60 }),
            None => self.strays += 1,
        }
    }

    pub fn score(&self) -> i32 {
        let n = self.notes.len().max(1) as i32;
        let sum: i32 = self.notes.iter().map(|x| x.got.unwrap_or(0)).sum();
        (sum / n - self.strays * 10).clamp(0, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play(lane: &mut Lane, offset: i32) {
        let hits: Vec<(i32, u8)> = lane.notes.iter().map(|n| (n.at + offset, n.lane)).collect();
        while !lane.done() {
            lane.step();
            for &(at, l) in &hits {
                if at == lane.tick {
                    lane.press(l);
                }
            }
        }
    }

    #[test]
    fn every_note_on_time_scores_100_and_none_scores_0() {
        let mut a = Lane::new(&mut Rng::new(1), false, 4, 9);
        play(&mut a, 0);
        assert_eq!(a.score(), 100);
        let mut b = Lane::new(&mut Rng::new(1), false, 4, 9);
        while !b.done() {
            b.step();
        }
        assert_eq!(b.score(), 0);
    }

    #[test]
    fn late_inside_good_scores_60_and_outside_it_misses() {
        let mut a = Lane::new(&mut Rng::new(2), false, 4, 9);
        play(&mut a, 6);
        assert_eq!(a.score(), 60);
        let mut b = Lane::new(&mut Rng::new(2), false, 4, 9);
        // ten ticks late: past the window, each press is a stray and each note a miss
        play(&mut b, 10);
        assert_eq!(b.score(), 0);
        assert_eq!(b.strays, 9);
    }

    #[test]
    fn a_stray_press_costs_ten() {
        let mut a = Lane::new(&mut Rng::new(3), false, 4, 9);
        a.step();
        a.press(0); // nothing near tick 1
        play(&mut a, 0);
        assert_eq!(a.score(), 90);
    }

    #[test]
    fn a_group_interview_has_more_notes_and_tighter_windows() {
        let one = Lane::new(&mut Rng::new(4), false, 4, 9);
        let group = Lane::new(&mut Rng::new(4), true, 4, 9);
        assert_eq!(one.notes.len(), 9);
        assert_eq!(group.notes.len(), 13);
        assert_eq!((group.perfect, group.good), (3, 6));
        assert_eq!(one.notes.last().unwrap().lane, CAST_LANE);
    }
}
