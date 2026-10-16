//! Glyphs and the drawing mini-game. Every spell piece has a glyph made of
//! named strokes; a spell's incantation is its pieces' glyphs joined left
//! to right in one line, like cursive. A drawing is scored against that
//! guide inside a band whose width shrinks as the audience grows.
//!
//! Glyph space: each glyph fills a box `BOX` units square, y down. Boxes
//! sit side by side `GAP` units apart and are joined by a straight stroke.

/// The strokes a glyph may use.
pub const STROKES: &[&str] = &[
    "loop", "ripple", "zig", "hook", "spiral", "vee", "peak", "line", "arch", "bolt", "eight", "square",
    "tri",
];

pub const BOX: i32 = 1000;
pub const GAP: i32 = 250;
/// Points every this many units along the guide when scoring.
pub const STEP: i32 = 25;

type P = (i32, i32);

/// Integer sine and cosine in thousandths for a circle of `n` points, from
/// the kit's fixed-point maths.
fn circle(n: i32, cx: i32, cy: i32, r: i32, start_turn: i32, turns_num: i32, turns_den: i32) -> Vec<P> {
    (0..=n)
        .map(|i| {
            let a = start_turn
                + (crate::fx::TURN as i64 * i as i64 * turns_num as i64 / (n as i64 * turns_den as i64))
                    as i32;
            let s = (crate::fx::sin(a) as i64 * r as i64) >> 16;
            let c = (crate::fx::cos(a) as i64 * r as i64) >> 16;
            (cx + s as i32, cy - c as i32)
        })
        .collect()
}

/// One stroke's points inside the box (0..BOX each way).
pub fn stroke(name: &str) -> Vec<P> {
    let b = BOX;
    match name {
        "loop" => circle(24, b / 2, b / 2, b * 2 / 5, crate::fx::HALF_TURN, 1, 1),
        "ripple" => (0..=24)
            .map(|i| {
                let x = b * i / 24;
                let a = (crate::fx::TURN as i64 * i as i64 * 2 / 24) as i32;
                (
                    x,
                    b / 2 - ((crate::fx::sin(a) as i64 * (b as i64 * 3 / 10)) >> 16) as i32,
                )
            })
            .collect(),
        "zig" => vec![
            (0, b * 3 / 4),
            (b / 6, b / 4),
            (b / 3, b * 3 / 4),
            (b / 2, b / 4),
            (b * 2 / 3, b * 3 / 4),
            (b * 5 / 6, b / 4),
            (b, b * 3 / 4),
        ],
        "hook" => {
            let mut v = vec![(b / 3, b / 10), (b / 3, b * 3 / 5)];
            v.extend(
                circle(10, b / 2, b * 3 / 5, b / 6, crate::fx::QUARTER_TURN * 3, 1, 2)
                    .into_iter()
                    .rev(),
            );
            v
        }
        "spiral" => (0..=36)
            .map(|i| {
                let r = b * 2 / 5 * i / 36;
                let a = (crate::fx::TURN as i64 * i as i64 * 3 / 72) as i32;
                (
                    b / 2 + ((crate::fx::sin(a) as i64 * r as i64) >> 16) as i32,
                    b / 2 - ((crate::fx::cos(a) as i64 * r as i64) >> 16) as i32,
                )
            })
            .collect(),
        "vee" => vec![(b / 10, b / 5), (b / 2, b * 4 / 5), (b * 9 / 10, b / 5)],
        "peak" => vec![(b / 10, b * 4 / 5), (b / 2, b / 5), (b * 9 / 10, b * 4 / 5)],
        "line" => vec![(0, b / 2), (b, b / 2)],
        "arch" => circle(16, b / 2, b * 3 / 4, b * 2 / 5, crate::fx::QUARTER_TURN * 3, 1, 2),
        "bolt" => vec![(b * 3 / 5, 0), (b / 4, b / 2), (b * 3 / 4, b / 2), (b * 2 / 5, b)],
        "eight" => {
            let mut v = circle(14, b / 2, b * 3 / 10, b / 5, crate::fx::HALF_TURN, 1, 1);
            v.extend(circle(14, b / 2, b * 7 / 10, b / 5, 0, 1, 1));
            v
        }
        "square" => vec![
            (b / 5, b / 5),
            (b * 4 / 5, b / 5),
            (b * 4 / 5, b * 4 / 5),
            (b / 5, b * 4 / 5),
            (b / 5, b / 5),
        ],
        "tri" => vec![
            (b / 2, b / 8),
            (b * 7 / 8, b * 4 / 5),
            (b / 8, b * 4 / 5),
            (b / 2, b / 8),
        ],
        _ => vec![(0, b / 2), (b, b / 2)],
    }
}

/// A glyph's strokes in order, each scaled into its share of the box.
pub fn glyph(strokes: &[String]) -> Vec<P> {
    let n = strokes.len().max(1) as i32;
    let mut out = Vec::new();
    for (k, s) in strokes.iter().enumerate() {
        let x0 = BOX * k as i32 / n;
        for (x, y) in stroke(s) {
            out.push((x0 + x / n, if n == 1 { y } else { BOX / 4 + y / 2 }));
        }
    }
    out
}

/// A spell's glyphs joined into one line, left to right.
pub fn incantation(glyphs: &[Vec<String>]) -> Vec<P> {
    let mut out: Vec<P> = Vec::new();
    for (k, g) in glyphs.iter().enumerate() {
        let x0 = k as i32 * (BOX + GAP);
        // the joining stroke is just the straight segment from the last point
        out.extend(glyph(g).into_iter().map(|(x, y)| (x0 + x, y)));
    }
    out
}

/// The width an incantation of `n` glyphs spans.
pub fn width(n: usize) -> i32 {
    n as i32 * BOX + (n as i32 - 1).max(0) * GAP
}

fn seg_dist2(p: P, a: P, b: P) -> i64 {
    let (px, py, ax, ay, bx, by) = (
        p.0 as i64, p.1 as i64, a.0 as i64, a.1 as i64, b.0 as i64, b.1 as i64,
    );
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0 {
        0
    } else {
        (((px - ax) * dx + (py - ay) * dy) * 1000 / len2).clamp(0, 1000)
    };
    let (qx, qy) = (ax + dx * t / 1000, ay + dy * t / 1000);
    (px - qx) * (px - qx) + (py - qy) * (py - qy)
}

fn dist2_to_path(p: P, path: &[P]) -> i64 {
    if path.len() == 1 {
        let (dx, dy) = ((p.0 - path[0].0) as i64, (p.1 - path[0].1) as i64);
        return dx * dx + dy * dy;
    }
    path.windows(2)
        .map(|w| seg_dist2(p, w[0], w[1]))
        .min()
        .unwrap_or(i64::MAX)
}

/// Points every `STEP` units along a path, so long and short segments
/// count by their length.
pub fn resample(path: &[P]) -> Vec<P> {
    let mut out = Vec::new();
    if let Some(&first) = path.first() {
        out.push(first);
    }
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (dx, dy) = ((b.0 - a.0) as i64, (b.1 - a.1) as i64);
        let len = crate::geom::isqrt((dx * dx + dy * dy) as u64) as i32;
        let n = (len / STEP).max(1);
        for i in 1..=n {
            out.push((
                a.0 + (dx * i as i64 / n as i64) as i32,
                a.1 + (dy * i as i64 / n as i64) as i32,
            ));
        }
    }
    out
}

/// How much nearness counts: a line half a band off on average loses this
/// share, halved, of its score (T26).
pub const NEAR_WEIGHT: i64 = 40;

/// The score of a drawing, 0 to 100: how much of the drawing lies inside
/// the band around the guide, times how much of the guide the drawing
/// passes within the band of, times how near to the guide's centre it runs
/// (T26). All matter: a dot on the line covers nothing, a scribble over
/// everything is mostly outside, and a steady line at the band's edge is
/// not a perfect one.
pub fn score(guide: &[P], drawn: &[P], band: i32) -> i32 {
    if guide.is_empty() || drawn.len() < 2 {
        return 0;
    }
    let g = resample(guide);
    let d = resample(drawn);
    let b2 = band as i64 * band as i64;
    let inside = d.iter().filter(|&&p| dist2_to_path(p, guide) <= b2).count() as i64;
    let covered = g.iter().filter(|&&p| dist2_to_path(p, drawn) <= b2).count() as i64;
    let precision = inside * 100 / d.len() as i64;
    let coverage = covered * 100 / g.len() as i64;
    let band = (band as i64).max(1);
    let off: i64 = d
        .iter()
        .map(|&p| (crate::geom::isqrt(dist2_to_path(p, guide).max(0) as u64) as i64).min(band))
        .sum::<i64>()
        / d.len() as i64;
    let near = 100 - off * NEAR_WEIGHT / band;
    (precision * coverage * near / 10_000) as i32
}

/// Effectiveness from a drawing's score (T26): 5 + 95 × (score ÷ 100)⁴ in
/// the balance's range, so 90 gives 66 and 95 gives 81. Only a flawless
/// drawing, 100, reaches the top.
pub fn effectiveness(score: i32, min: i32, max: i32) -> i32 {
    let s = score.clamp(0, 100) as i64;
    (min as i64 + (max - min) as i64 * s * s * s * s / 100_000_000) as i32
}

/// The band's half-width in glyph units for an audience, from the
/// balance file's (students, px) pairs, read as straight lines between
/// them. `px_per_box` says how many overlay pixels one glyph box spans.
pub fn band(curve: &[[i32; 2]], audience: i32, units_per_px: i32) -> i32 {
    let a = audience.max(1);
    let px = match curve.iter().position(|c| c[0] >= a) {
        None => curve.last().map(|c| c[1]).unwrap_or(20),
        Some(0) => curve[0][1],
        Some(i) => {
            let (lo, hi) = (curve[i - 1], curve[i]);
            lo[1] + (hi[1] - lo[1]) * (a - lo[0]) / (hi[0] - lo[0]).max(1)
        }
    };
    px * units_per_px
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stroke_stays_inside_its_box() {
        for s in STROKES {
            for (x, y) in stroke(s) {
                assert!(
                    (-2..=BOX + 2).contains(&x) && (-2..=BOX + 2).contains(&y),
                    "{s} at ({x},{y})"
                );
            }
        }
    }

    #[test]
    fn a_drawing_on_the_guide_scores_100_and_a_dot_scores_0() {
        let g = vec![(0, 500), (1000, 500)];
        assert_eq!(score(&g, &g, 50), 100);
        // a dash from 0 to 100 on the line: all of it inside; the band reaches
        // 50 past its end, so it covers guide points 0..=150: 7 of 41 → 17
        assert_eq!(score(&g, &[(0, 500), (100, 500)], 50), 17);
        assert_eq!(score(&g, &[(500, 500)], 50), 0);
    }

    #[test]
    fn a_drawing_off_by_more_than_the_band_scores_0() {
        let g = vec![(0, 500), (1000, 500)];
        assert_eq!(score(&g, &[(0, 600), (1000, 600)], 50), 0);
        // inside a wider band, but every point 100 off the centre:
        // nearness 100 − 100 × 40 / 120 = 67, so 100 × 100 × 67 / 10,000
        assert_eq!(score(&g, &[(0, 600), (1000, 600)], 120), 67, "inside, off centre");
    }

    #[test]
    fn effectiveness_climbs_steeply_and_only_a_flawless_drawing_reaches_the_top() {
        // 5 + 95 × s⁴ / 10⁸, by hand
        assert_eq!(effectiveness(100, 5, 100), 100);
        assert_eq!(effectiveness(95, 5, 100), 82); // 95⁴ = 81,450,625 → 5 + 77
        assert_eq!(effectiveness(90, 5, 100), 67); // 90⁴ = 65,610,000 → 5 + 62
        assert_eq!(effectiveness(50, 5, 100), 10); // 50⁴ = 6,250,000 → 5 + 5
        assert_eq!(effectiveness(0, 5, 100), 5);
        assert_eq!(effectiveness(99, 5, 100), 96); // 99⁴ = 96,059,601 → 5 + 91
    }

    #[test]
    fn half_on_and_half_off_scores_by_hand() {
        // guide 0..1000 at y 500; the drawing runs along it to x 500, drops
        // to y 800 and runs on. Resampled at 25: 21 + 12 + 20 = 53 points;
        // inside a band of 50 are the first 21 and the drop's y 525 and 550: 23.
        // The guide's 41 points are covered from x 0 to 550: 23.
        let g = vec![(0, 500), (1000, 500)];
        let d = vec![(0, 500), (500, 500), (500, 800), (1000, 800)];
        let inside = resample(&d)
            .iter()
            .filter(|&&p| dist2_to_path(p, &g) <= 2500)
            .count();
        assert_eq!((inside, resample(&d).len()), (23, 53));
        let covered = resample(&g)
            .iter()
            .filter(|&&p| dist2_to_path(p, &d) <= 2500)
            .count();
        assert_eq!(covered, 23);
        // 23/53 → 43%, 23/41 → 56%. Nearness: the first 21 points are on the
        // line (0); the drop's 12 are 25, 50 and then capped at the band, 50
        // (575 in all); the last 20 are 300 off, capped at 50 (1,000). Mean
        // 1,575 / 53 = 29, so nearness 100 − 29 × 40 / 50 = 77.
        // 43 × 56 × 77 / 10,000 = 18
        assert_eq!(score(&g, &d, 50), 18);
    }

    #[test]
    fn the_band_narrows_as_the_audience_grows() {
        let curve = [[1, 48], [12, 22], [25, 12]];
        assert_eq!(band(&curve, 1, 5), 240);
        assert_eq!(band(&curve, 12, 5), 110);
        assert_eq!(band(&curve, 25, 5), 60);
        assert_eq!(band(&curve, 100, 5), 60);
        // between points by straight line: 6 students → 48 + (22−48)×5/11 = 37
        assert_eq!(band(&curve, 6, 1), 37);
        assert!(band(&curve, 1, 1) > band(&curve, 25, 1));
    }

    #[test]
    fn an_incantation_sets_its_glyphs_side_by_side() {
        let inc = incantation(&[vec!["line".into()], vec!["line".into()]]);
        assert_eq!(inc.first(), Some(&(0, 500)));
        assert_eq!(inc.last(), Some(&(BOX + GAP + BOX, 500)));
        assert_eq!(width(2), 2250);
    }
}
