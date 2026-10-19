//! Spell space (TRIAGE T4): every chain of spell pieces, played in one
//! short fixed scene and checked against rules that hold for every spell,
//! after Galactic Survivors' rig space and rig log.
//!
//! A chain is a row of 1 to `max_chain` pieces from the 33 that can go in a
//! spell (helpers are summoned, never chained). Rows are numbered in base
//! 33, piece order as in `data/content.json`; a row the laboratory would
//! refuse (`spell::check`) is counted by its reason and not played.
//!
//! The everyday test plays every chain of one and two pieces. The ignored
//! logbook plays longer ones and keeps a line for each:
//!
//!   cargo test --release -p tool_core --test spell_space -- --ignored --nocapture logbook
//!
//! SPELL_BUDGET_S (seconds, default 120) and SPELL_LENGTHS (default 1,2,3).
//! Lengths up to three are logged in number order; four and longer in a
//! fixed shuffled order, so a run that stops early has logged an even
//! sample. The scene and the rules are in `support/spell_rules.rs`.

#[path = "support/spell_rules.rs"]
mod spell_rules;
use spell_rules::*;

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tool_core::content::{Content, PieceKind};
use tool_core::spell;

/// The pieces a chain may hold, as indices into the content's pieces.
fn tokens(c: &Content) -> Vec<u16> {
    (0..c.file.pieces.len())
        .filter(|&i| {
            !matches!(
                c.file.pieces[i].kind,
                PieceKind::Helper { .. } | PieceKind::SelfCast { .. } | PieceKind::Jinx { .. }
            )
        })
        .map(|i| i as u16)
        .collect()
}

fn rows(t: &[u16], len: usize) -> u64 {
    (t.len() as u64).pow(len as u32)
}

/// The row with number `i` among all rows of `len` pieces: the first piece
/// is the lowest digit. Every number is one row and every row one number.
fn row_at(t: &[u16], len: usize, mut i: u64) -> Vec<u16> {
    let n = t.len() as u64;
    let mut row = Vec::with_capacity(len);
    for _ in 0..len {
        row.push(t[(i % n) as usize]);
        i /= n;
    }
    row
}

fn number_of(t: &[u16], row: &[u16]) -> u64 {
    let n = t.len() as u64;
    row.iter().rev().fold(0, |acc, p| {
        acc * n + t.iter().position(|q| q == p).unwrap() as u64
    })
}

/// The row as piece ids, for a person reading the log.
fn ids(c: &Content, row: &[u16]) -> String {
    row.iter()
        .map(|&p| c.file.pieces[p as usize].id.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Default)]
struct Tally {
    played: usize,
    refused: BTreeMap<&'static str, usize>,
    failures: Vec<String>,
    notes: BTreeMap<String, usize>,
    most_units: usize,
}

fn quiet_hook() {
    std::panic::set_hook(Box::new(|_| {}));
}

/// Plays rows on every core; each result is the row's judgement, or the
/// reason the laboratory refuses it.
fn play_all(c: &Content, rows: &[Vec<u16>], deep_every: usize) -> Vec<Result<Played, &'static str>> {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let chunk = rows.len().div_ceil(threads).max(1);
    let mut out = Vec::with_capacity(rows.len());
    std::thread::scope(|s| {
        let jobs: Vec<_> = rows
            .chunks(chunk)
            .enumerate()
            .map(|(j, part)| {
                s.spawn(move || {
                    part.iter()
                        .enumerate()
                        .map(|(i, row)| {
                            spell::check(c, row).map_err(|e| e.key()).map(|_| {
                                let deep = deep_every > 0 && (j * chunk + i).is_multiple_of(deep_every);
                                judge(c, row, deep)
                            })
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for j in jobs {
            out.extend(j.join().unwrap());
        }
    });
    out
}

fn tally(c: &Content, rows: &[Vec<u16>], results: Vec<Result<Played, &'static str>>) -> Tally {
    let mut t = Tally::default();
    for (row, r) in rows.iter().zip(results) {
        match r {
            Err(why) => *t.refused.entry(why).or_default() += 1,
            Ok(p) => {
                t.played += 1;
                t.most_units = t.most_units.max(p.most_units);
                for f in p.failures {
                    t.failures.push(format!("[{}] {f}", ids(c, row)));
                }
                for n in p.notes {
                    *t.notes.entry(n).or_default() += 1;
                }
            }
        }
    }
    t
}

/// Everyday: every chain of one and two pieces, each played twice for the
/// determinism rule.
#[test]
fn every_chain_of_one_and_two_pieces_keeps_every_rule() {
    let c = content();
    let t = tokens(&c);
    let all: Vec<Vec<u16>> = (1..=2)
        .flat_map(|len| (0..rows(&t, len)).map(move |i| (len, i)))
        .map(|(len, i)| row_at(&t, len, i))
        .collect();
    quiet_hook();
    let start = Instant::now();
    let results = play_all(&c, &all, 1);
    let _ = std::panic::take_hook();
    let secs = start.elapsed().as_secs_f64();
    let tally = tally(&c, &all, results);
    println!(
        "everyday: {} rows, {} played, refused {:?}, in {secs:.2} s; most units {}; notes {:?}",
        all.len(),
        tally.played,
        tally.refused,
        tally.most_units,
        tally.notes
    );
    assert_eq!(
        t.len(),
        34,
        "the pieces a spell may hold, Branch among them (MEASUREMENTS.md)"
    );
    assert_eq!(all.len(), 34 + 34 * 34);
    assert!(tally.played > 100, "most short rows are spells: {}", tally.played);
    assert!(
        tally.failures.is_empty(),
        "{} chains broke a rule:\n{}",
        tally.failures.len(),
        tally.failures.join("\n")
    );
}

/// The numbering turns a row into its number and back, and the shuffle for
/// long rows visits each number once.
#[test]
fn row_numbers_and_the_shuffle_are_one_to_one() {
    let c = content();
    let t = tokens(&c);
    for len in 1..=3 {
        for i in (0..rows(&t, len)).step_by(97) {
            assert_eq!(number_of(&t, &row_at(&t, len, i)), i);
        }
    }
    let firsts: std::collections::BTreeSet<u64> = (0..50_000).map(|s| shuffled(&t, 4, s)).collect();
    assert_eq!(firsts.len(), 50_000);
    assert!(
        firsts
            .iter()
            .filter(|n| **n < rows(&t, 4) / 2)
            .count()
            .abs_diff(25_000)
            < 1_000,
        "an even sample"
    );
}

/// A panic inside a battle is caught and reported, not lost: the other
/// rules are proved by the break rows in tests/breaks.json.
#[test]
fn the_judge_reports_a_crash_as_a_failure() {
    let c = content();
    // a piece id past the end panics inside the battle: the judge says so
    let p = judge(&c, &[9999], false);
    assert!(
        p.failures.iter().any(|f| f.starts_with("panic")),
        "{:?}",
        p.failures
    );
}

// ---- the logbook --------------------------------------------------------------
//
// The full log stays on this computer, in `spell-log/` (not in git): one
// line for every row played, in files of 4,096 row numbers. The repository
// keeps a summary in `tests/spell-log/`.

const BLOCK: u64 = 4096;
/// The files a spell's play depends on; a change to any makes every line stale.
const RULE_FILES: &[&str] = &[
    "crates/core/src/battle.rs",
    "crates/core/src/body.rs",
    "crates/core/src/ragdoll.rs",
    "crates/core/src/arena.rs",
    "crates/core/src/spell.rs",
    "crates/core/src/content.rs",
    "crates/core/src/geom.rs",
    "crates/core/src/fx.rs",
    "crates/core/src/rng.rs",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// What every line depends on (the rule code, the scene, the piece list,
/// the content that is not a piece) and each piece's own data.
struct Keys {
    code: u64,
    piece: BTreeMap<u16, u64>,
}

fn keys(c: &Content, t: &[u16]) -> Keys {
    let r = root();
    let mut code = FNV;
    for f in RULE_FILES {
        code = fnv(code, &std::fs::read(r.join(f)).unwrap());
    }
    code = fnv(
        code,
        &std::fs::read(r.join("crates/core/tests/support/spell_rules.rs")).unwrap(),
    );
    code = fnv(code, ids(c, t).as_bytes());
    let text = std::fs::read_to_string(r.join("data/content.json")).unwrap();
    let mut data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut entry = BTreeMap::new();
    for e in data["pieces"].as_array().unwrap() {
        entry.insert(e["id"].as_str().unwrap().to_string(), e.to_string());
    }
    data["pieces"] = serde_json::Value::Null;
    code = fnv(code, data.to_string().as_bytes());
    let piece = t
        .iter()
        .map(|&p| (p, fnv(FNV, entry[&c.file.pieces[p as usize].id].as_bytes())))
        .collect();
    Keys { code, piece }
}

fn row_key(k: &Keys, row: &[u16]) -> u64 {
    row.iter().fold(k.code, |h, p| fnv(h, &k.piece[p].to_le_bytes()))
}

/// The row number logged in place `slot`: in order up to three pieces;
/// from four, `slot * a mod 33^len` with `a` near 0.618 of the space and
/// sharing no factor with it (33 is 3 × 11), so every number comes once.
fn shuffled(t: &[u16], len: usize, slot: u64) -> u64 {
    if len <= 3 {
        return slot;
    }
    let n = rows(t, len);
    let mut a = (n as u128 * 618 / 1000) as u64;
    while a.is_multiple_of(3) || a.is_multiple_of(11) {
        a += 1;
    }
    ((slot as u128 * a as u128) % n as u128) as u64
}

#[derive(Clone)]
struct Line {
    number: u64,
    row: String,
    result: String,
    behaviour: String,
    print: u64,
    key: u64,
    tested: String,
}

impl Line {
    fn text(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{:016x}\t{:016x}\t{}",
            self.number, self.row, self.result, self.behaviour, self.print, self.key, self.tested
        )
    }

    fn parse(s: &str) -> Option<Line> {
        let f: Vec<&str> = s.split('\t').collect();
        if f.len() != 7 {
            return None;
        }
        Some(Line {
            number: f[0].parse().ok()?,
            row: f[1].to_string(),
            result: f[2].to_string(),
            behaviour: f[3].to_string(),
            print: u64::from_str_radix(f[4], 16).ok()?,
            key: u64::from_str_radix(f[5], 16).ok()?,
            tested: f[6].to_string(),
        })
    }
}

fn result_text(r: &Result<Played, &'static str>) -> String {
    let Ok(p) = r else {
        return format!("refused {}", r.as_ref().err().unwrap());
    };
    let mut v: Vec<String> = p.failures.iter().map(|f| format!("FAIL {f}")).collect();
    v.extend(p.notes.iter().map(|n| format!("note {n}")));
    if v.is_empty() {
        "ok".into()
    } else {
        v.join("; ").replace('\t', " ")
    }
}

fn behaviour_text(r: &Result<Played, &'static str>) -> String {
    match r {
        Err(_) => "-".into(),
        Ok(p) => format!(
            "mana {} casts {} dealt {} kills {} status {} allies {} invaders {} raised {} fields {} ward {} units {}",
            p.mana, p.casts, p.dealt, p.kills, p.statuses, p.allies, p.invaders, p.raised, p.zones, p.ward, p.most_units
        ),
    }
}

/// Today's date from the system clock (UTC), as 2026-10-16.
fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // days to a civil date (Howard Hinnant's algorithm)
    let z = secs / 86_400 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

/// The commit the rules were read from, with + when the tree has changes.
fn commit() -> String {
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(root())
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    };
    let head = git(&["rev-parse", "--short", "HEAD"]);
    let dirty = !git(&[
        "status",
        "--porcelain",
        "--untracked-files=no",
        "--",
        ".",
        ":!tests/spell-log",
    ])
    .is_empty();
    format!(
        "{}{}",
        if head.is_empty() { "unknown" } else { &head },
        if dirty { "+" } else { "" }
    )
}

#[derive(Clone, Default)]
struct Block {
    key: u64,
    rows: u64,
    played: u64,
    failed: u64,
    noted: u64,
    print: u64,
    tested: String,
}

type Index = BTreeMap<(usize, u64), Block>;

fn read_index(p: &Path, list: u64) -> Index {
    let mut out = Index::new();
    let Ok(f) = std::fs::File::open(p) else { return out };
    let mut lines = std::io::BufReader::new(f).lines().map_while(Result::ok);
    // a different list of pieces numbers rows differently: start again
    if lines.next() != Some(format!("pieces\t{list:016x}")) {
        return out;
    }
    for l in lines {
        let f: Vec<&str> = l.split('\t').collect();
        if f.len() != 9 {
            continue;
        }
        let (Ok(len), Ok(b)) = (f[0].parse(), f[1].parse()) else {
            continue;
        };
        out.insert(
            (len, b),
            Block {
                key: u64::from_str_radix(f[2], 16).unwrap_or(0),
                rows: f[3].parse().unwrap_or(0),
                played: f[4].parse().unwrap_or(0),
                failed: f[5].parse().unwrap_or(0),
                noted: f[6].parse().unwrap_or(0),
                print: u64::from_str_radix(f[7], 16).unwrap_or(0),
                tested: f[8].to_string(),
            },
        );
    }
    out
}

fn write_index(p: &Path, list: u64, index: &Index) {
    let mut s = format!("pieces\t{list:016x}\n");
    for ((len, b), e) in index {
        s += &format!(
            "{len}\t{b}\t{:016x}\t{}\t{}\t{}\t{}\t{:016x}\t{}\n",
            e.key, e.rows, e.played, e.failed, e.noted, e.print, e.tested
        );
    }
    std::fs::write(p, s).unwrap();
}

fn block_path(dir: &Path, len: usize, b: u64) -> PathBuf {
    dir.join(format!("L{len}")).join(format!("{b:06}.tsv"))
}

fn read_block(p: &Path) -> BTreeMap<u64, Line> {
    let Ok(f) = std::fs::File::open(p) else {
        return BTreeMap::new();
    };
    std::io::BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| Line::parse(&l))
        .map(|l| (l.number, l))
        .collect()
}

fn write_block(p: &Path, lines: &BTreeMap<u64, Line>) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    let mut f = std::io::BufWriter::new(std::fs::File::create(p).unwrap());
    for l in lines.values() {
        writeln!(f, "{}", l.text()).unwrap();
    }
}

#[test]
#[ignore]
fn logbook() {
    let get = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
    let budget = get("SPELL_BUDGET_S", "120").parse::<f64>().unwrap_or(120.0);
    let lengths: Vec<usize> = get("SPELL_LENGTHS", "1,2,3")
        .split(',')
        .filter_map(|x| x.trim().parse().ok())
        .collect();
    let c = content();
    let t = tokens(&c);
    let k = keys(&c, &t);
    let list = fnv(FNV, ids(&c, &t).as_bytes());
    let dir = root().join("spell-log");
    let repo = root().join("tests/spell-log");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    let mut index = read_index(&dir.join("index.tsv"), list);
    if index.is_empty() {
        // a new list of pieces renumbers every row: the old files go
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            if e.path().is_dir() {
                std::fs::remove_dir_all(e.path()).unwrap();
            }
        }
    }
    let (date, at) = (today(), commit());
    let tested = format!("{date} {at}");
    let start = Instant::now();
    let (mut played, mut fresh, mut changed) = (0u64, 0u64, Vec::new());
    let mut notable: BTreeMap<String, String> = std::fs::read_to_string(dir.join("notable.tsv"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.splitn(3, '\t').collect();
            (f.len() == 3).then(|| (format!("{}\t{}", f[0], f[1]), f[2].to_string()))
        })
        .collect();
    quiet_hook();
    'all: for &len in &lengths {
        let blocks = rows(&t, len).div_ceil(BLOCK);
        for b in 0..blocks {
            if start.elapsed().as_secs_f64() > budget {
                break 'all;
            }
            let span = b * BLOCK..((b + 1) * BLOCK).min(rows(&t, len));
            let want: Vec<(u64, Vec<u16>, u64)> = span
                .map(|slot| shuffled(&t, len, slot))
                .map(|n| {
                    let r = row_at(&t, len, n);
                    let key = row_key(&k, &r);
                    (n, r, key)
                })
                .collect();
            let block_key = want.iter().fold(FNV, |h, (_, _, key)| fnv(h, &key.to_le_bytes()));
            if index.get(&(len, b)).map(|e| e.key == block_key) == Some(true) {
                fresh += want.len() as u64;
                continue;
            }
            let path = block_path(&dir, len, b);
            let mut lines = read_block(&path);
            let todo: Vec<(u64, Vec<u16>, u64)> = want
                .iter()
                .filter(|(n, _, key)| lines.get(n).map(|l| l.key != *key).unwrap_or(true))
                .cloned()
                .collect();
            let todo_rows: Vec<Vec<u16>> = todo.iter().map(|(_, r, _)| r.clone()).collect();
            for ((n, row, key), r) in todo.iter().zip(play_all(&c, &todo_rows, 4)) {
                let line = Line {
                    number: *n,
                    row: ids(&c, row),
                    result: result_text(&r),
                    behaviour: behaviour_text(&r),
                    print: r.as_ref().map(|p| p.print).unwrap_or(0),
                    key: *key,
                    tested: tested.clone(),
                };
                let id = format!("{len}\t{n}");
                if let Some(old) = lines.get(n) {
                    if old.print != line.print {
                        changed.push(format!(
                            "{id}\t{}\twas: {} | {}\tnow: {} | {}",
                            line.row, old.result, old.behaviour, line.result, line.behaviour
                        ));
                    }
                }
                if line.result == "ok" || line.result.starts_with("refused") {
                    notable.remove(&id);
                } else {
                    notable.insert(id, format!("{}\t{}\t{}", line.row, line.result, tested));
                }
                lines.insert(*n, line);
                if r.is_ok() {
                    played += 1;
                }
            }
            write_block(&path, &lines);
            let is_spell = |l: &Line| !l.result.starts_with("refused");
            index.insert(
                (len, b),
                Block {
                    key: block_key,
                    rows: lines.len() as u64,
                    played: lines.values().filter(|l| is_spell(l)).count() as u64,
                    failed: lines.values().filter(|l| l.result.contains("FAIL")).count() as u64,
                    noted: lines
                        .values()
                        .filter(|l| is_spell(l) && l.result != "ok" && !l.result.contains("FAIL"))
                        .count() as u64,
                    print: lines.values().fold(FNV, |h, l| fnv(h, &l.print.to_le_bytes())),
                    tested: tested.clone(),
                },
            );
        }
    }
    let _ = std::panic::take_hook();
    let secs = start.elapsed().as_secs_f64();
    write_index(&dir.join("index.tsv"), list, &index);
    let notable_text: String = notable.iter().map(|(k, v)| format!("{k}\t{v}\n")).collect();
    std::fs::write(dir.join("notable.tsv"), &notable_text).unwrap();
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("changes.tsv"))
        .unwrap();
    for l in &changed {
        writeln!(log, "{date}\t{at}\t{l}").unwrap();
    }
    write_summary(&repo, &t, &index, &notable_text, &changed, &tested, played, secs);
    let logged: u64 = index.values().map(|e| e.rows).sum();
    let spells: u64 = index.values().map(|e| e.played).sum();
    println!(
        "logbook: played {played} spells in {secs:.1} s, {fresh} rows already fresh; {} changed; \
         {logged} rows in the log, {spells} of them spells; {} failing or noted",
        changed.len(),
        notable.len()
    );
    let failing = notable.values().filter(|v| v.contains("FAIL")).count();
    assert_eq!(
        failing, 0,
        "spells in the log break a rule: see spell-log/notable.tsv"
    );
}

#[allow(clippy::too_many_arguments)]
fn write_summary(
    repo: &Path,
    t: &[u16],
    index: &Index,
    notable: &str,
    changed: &[String],
    tested: &str,
    played: u64,
    secs: f64,
) {
    let mut s = String::from(
        "# One line for each file of 4,096 row numbers in the local log (spell-log/).\n\
         # Four pieces and longer are summed over 64 blocks a line.\n\
         # length\tblock\trows\tspells\tfailed\tnoted\tbehaviour\tkey\tlast tested\n",
    );
    let mut by_len: BTreeMap<usize, (u64, u64)> = BTreeMap::new();
    let mut chapters: BTreeMap<(usize, u64), Block> = BTreeMap::new();
    for ((len, b), e) in index {
        if *len >= 4 {
            let ch = chapters.entry((*len, b / 64)).or_default();
            ch.rows += e.rows;
            ch.played += e.played;
            ch.failed += e.failed;
            ch.noted += e.noted;
            ch.print = fnv(ch.print.max(1), &e.print.to_le_bytes());
            ch.key = fnv(ch.key.max(1), &e.key.to_le_bytes());
            ch.tested = ch.tested.clone().max(e.tested.clone());
        } else {
            s += &format!(
                "{len}\t{b}\t{}\t{}\t{}\t{}\t{:016x}\t{:016x}\t{}\n",
                e.rows, e.played, e.failed, e.noted, e.print, e.key, e.tested
            );
        }
        let x = by_len.entry(*len).or_default();
        x.0 += e.rows;
        x.1 += e.played;
    }
    for ((len, ch), e) in &chapters {
        s += &format!(
            "{len}\tblocks {}-{}\t{}\t{}\t{}\t{}\t{:016x}\t{:016x}\t{}\n",
            ch * 64,
            ch * 64 + 63,
            e.rows,
            e.played,
            e.failed,
            e.noted,
            e.print,
            e.key,
            e.tested
        );
    }
    std::fs::write(repo.join("summary.tsv"), s).unwrap();
    std::fs::write(
        repo.join("notable.tsv"),
        format!("# length\trow number\tchain\tresult\tlast tested\n{notable}"),
    )
    .unwrap();
    let mut runs = std::fs::read_to_string(repo.join("runs.tsv")).unwrap_or_else(|_| {
        "# date and commit\tplayed\tseconds\tchanged\tlogged by length (rows, of which spells)\n".into()
    });
    let cover: Vec<String> = by_len
        .iter()
        .map(|(len, (n, sp))| {
            let all = rows(t, *len);
            format!(
                "L{len}: {n} of {all} rows ({}.{:02}%), {sp} spells",
                n * 100 / all,
                n * 10_000 / all % 100
            )
        })
        .collect();
    runs += &format!(
        "{tested}\t{played}\t{secs:.0}\t{}\t{}\n",
        changed.len(),
        cover.join(", ")
    );
    std::fs::write(repo.join("runs.tsv"), runs).unwrap();
    let mut ch = String::from(
        "# Spells whose behaviour changed in the latest run (at most 200; all are in spell-log/changes.tsv)\n",
    );
    for l in changed.iter().take(200) {
        ch += l;
        ch += "\n";
    }
    std::fs::write(repo.join("changes.tsv"), ch).unwrap();
}
