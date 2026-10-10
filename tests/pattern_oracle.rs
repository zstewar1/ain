//! Exhaustive cross-check of ain's pattern operations against `std::str`.
//!
//! Matching in ain is ASCII-case-insensitive, so std is run on lowercased copies of both the
//! haystack and the needle. The results must then agree exactly: indices as they are, pieces after
//! lowercasing. Every string over a small alphabet up to a given length is used as a haystack and
//! as a needle, which covers the empty needle, needles longer than the haystack, overlapping
//! candidates, and the scan character sitting at every offset of the needle.
//!
//! The default-size tests take a few seconds in debug builds. The large variants are `#[ignore]`d;
//! run them with `cargo test --release -- --ignored`.

use std::fmt::Debug;

use ain::{AinChar, AinStr};

fn fold(s: &str) -> String {
    s.to_ascii_lowercase()
}

fn a(s: &str) -> &AinStr {
    AinStr::from_str(s).unwrap()
}

fn ac(b: u8) -> AinChar {
    AinChar::from_u8(b).unwrap()
}

/// All strings over `alphabet` of length `0..=max_len`, shortest first.
fn strings(alphabet: &[u8], max_len: usize) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut layer = vec![String::new()];
    for _ in 0..max_len {
        let mut next = Vec::with_capacity(layer.len() * alphabet.len());
        for s in &layer {
            for &c in alphabet {
                let mut t = s.clone();
                t.push(c as char);
                next.push(t);
            }
        }
        out.extend(next.iter().cloned());
        layer = next;
    }
    out
}

fn pieces<'a>(it: impl Iterator<Item = &'a AinStr>) -> Vec<String> {
    it.map(|s| fold(s.as_str())).collect()
}

fn pieces_mut<'a>(it: impl Iterator<Item = &'a mut AinStr>) -> Vec<String> {
    it.map(|s| fold(s.as_str())).collect()
}

fn owned<'a>(it: impl Iterator<Item = &'a str>) -> Vec<String> {
    it.map(String::from).collect()
}

/// Pulls from the front and the back alternately until exhausted, in retrieval order.
fn alternate<I: DoubleEndedIterator>(mut it: I) -> Vec<I::Item> {
    let mut out = Vec::new();
    let mut front = true;
    loop {
        let item = if front { it.next() } else { it.next_back() };
        match item {
            Some(x) => out.push(x),
            None => break,
        }
        front = !front;
    }
    out
}

/// Collects mismatches so one run reports everything that is wrong, not just the first case.
#[derive(Default)]
struct Mismatches {
    count: usize,
    samples: Vec<String>,
}

impl Mismatches {
    fn check<T: PartialEq + Debug>(&mut self, ctx: impl Fn() -> String, got: T, want: T) {
        if got != want {
            self.count += 1;
            if self.samples.len() < 20 {
                self.samples
                    .push(format!("{}: got {got:?}, want {want:?}", ctx()));
            }
        }
    }

    fn finish(self, what: &str) {
        assert!(
            self.count == 0,
            "{what}: {} mismatches against std, e.g.:\n{}",
            self.count,
            self.samples.join("\n")
        );
    }
}

fn str_oracle(alphabet: &[u8], max_haystack: usize, max_needle: usize) {
    let mut m = Mismatches::default();
    let needles = strings(alphabet, max_needle);
    for h in &strings(alphabet, max_haystack) {
        let fh = fold(h);
        let hs = a(h);
        let mut buf: Vec<AinChar> = hs.chars().collect();
        for n in &needles {
            let fnd = fold(n);
            let fnd = fnd.as_str();
            let ns = a(n);
            let ctx = |op: &'static str| move || format!("{h:?}.{op}({n:?})");

            m.check(ctx("find"), hs.find(ns), fh.find(fnd));
            m.check(ctx("rfind"), hs.rfind(ns), fh.rfind(fnd));
            m.check(ctx("contains"), hs.contains(ns), fh.contains(fnd));
            m.check(ctx("split"), pieces(hs.split(ns)), owned(fh.split(fnd)));
            m.check(ctx("rsplit"), pieces(hs.rsplit(ns)), owned(fh.rsplit(fnd)));
            for k in 0..=3 {
                let ctx = |op: &'static str| move || format!("{h:?}.{op}({k}, {n:?})");
                m.check(
                    ctx("splitn"),
                    pieces(hs.splitn(k, ns)),
                    owned(fh.splitn(k, fnd)),
                );
                m.check(
                    ctx("rsplitn"),
                    pieces(hs.rsplitn(k, ns)),
                    owned(fh.rsplitn(k, fnd)),
                );
            }
            m.check(
                ctx("strip_prefix"),
                hs.strip_prefix(ns).map(|s| fold(s.as_str())),
                fh.strip_prefix(fnd).map(String::from),
            );
            m.check(
                ctx("strip_suffix"),
                hs.strip_suffix(ns).map(|s| fold(s.as_str())),
                fh.strip_suffix(fnd).map(String::from),
            );
            m.check(
                ctx("trim_prefix"),
                fold(hs.trim_prefix(ns).as_str()),
                fh.strip_prefix(fnd).unwrap_or(&fh).to_string(),
            );
            m.check(
                ctx("trim_suffix"),
                fold(hs.trim_suffix(ns).as_str()),
                fh.strip_suffix(fnd).unwrap_or(&fh).to_string(),
            );
            m.check(
                ctx("trim_start_matches"),
                fold(hs.trim_start_matches(ns).as_str()),
                fh.trim_start_matches(fnd).to_string(),
            );
            m.check(
                ctx("trim_end_matches"),
                fold(hs.trim_end_matches(ns).as_str()),
                fh.trim_end_matches(fnd).to_string(),
            );

            // The `_mut` iterators share the implementation with the shared ones, but check that the
            // plumbing produces the same pieces.
            let hm = AinStr::from_slice_mut(&mut buf);
            m.check(
                ctx("split_mut"),
                pieces_mut(hm.split_mut(ns)),
                owned(fh.split(fnd)),
            );
            m.check(
                ctx("rsplit_mut"),
                pieces_mut(hm.rsplit_mut(ns)),
                owned(fh.rsplit(fnd)),
            );
        }
    }
    m.finish("&AinStr patterns");
}

fn char_oracle(alphabet: &[u8], max_haystack: usize) {
    let mut m = Mismatches::default();
    for h in &strings(alphabet, max_haystack) {
        let fh = fold(h);
        let hs = a(h);
        for &c in alphabet {
            let cc = ac(c);
            let fc = (c as char).to_ascii_lowercase();
            let ctx = |op: &'static str| move || format!("{h:?}.{op}({:?})", c as char);

            m.check(ctx("find"), hs.find(cc), fh.find(fc));
            m.check(ctx("rfind"), hs.rfind(cc), fh.rfind(fc));
            m.check(ctx("contains"), hs.contains(cc), fh.contains(fc));
            m.check(ctx("split"), pieces(hs.split(cc)), owned(fh.split(fc)));
            m.check(ctx("rsplit"), pieces(hs.rsplit(cc)), owned(fh.rsplit(fc)));
            m.check(
                ctx("split rev"),
                pieces(hs.split(cc).rev()),
                owned(fh.split(fc).rev()),
            );
            m.check(
                ctx("split alternate"),
                pieces(alternate(hs.split(cc)).into_iter()),
                owned(alternate(fh.split(fc)).into_iter()),
            );
            m.check(
                ctx("rsplit alternate"),
                pieces(alternate(hs.rsplit(cc)).into_iter()),
                owned(alternate(fh.rsplit(fc)).into_iter()),
            );
            for k in 0..=3 {
                let ctx = |op: &'static str| move || format!("{h:?}.{op}({k}, {:?})", c as char);
                m.check(
                    ctx("splitn"),
                    pieces(hs.splitn(k, cc)),
                    owned(fh.splitn(k, fc)),
                );
                m.check(
                    ctx("rsplitn"),
                    pieces(hs.rsplitn(k, cc)),
                    owned(fh.rsplitn(k, fc)),
                );
            }
            m.check(
                ctx("strip_prefix"),
                hs.strip_prefix(cc).map(|s| fold(s.as_str())),
                fh.strip_prefix(fc).map(String::from),
            );
            m.check(
                ctx("strip_suffix"),
                hs.strip_suffix(cc).map(|s| fold(s.as_str())),
                fh.strip_suffix(fc).map(String::from),
            );
            m.check(
                ctx("trim_start_matches"),
                fold(hs.trim_start_matches(cc).as_str()),
                fh.trim_start_matches(fc).to_string(),
            );
            m.check(
                ctx("trim_end_matches"),
                fold(hs.trim_end_matches(cc).as_str()),
                fh.trim_end_matches(fc).to_string(),
            );
            m.check(
                ctx("trim_matches"),
                fold(hs.trim_matches(cc).as_str()),
                fh.trim_matches(fc).to_string(),
            );

            // Character sets, as a slice and as an array.
            for &c2 in alphabet {
                let set = [cc, ac(c2)];
                let fset = [fc, (c2 as char).to_ascii_lowercase()];
                let ctx = |op: &'static str| {
                    move || format!("{h:?}.{op}([{:?}, {:?}])", c as char, c2 as char)
                };
                m.check(ctx("set find"), hs.find(&set[..]), fh.find(&fset[..]));
                m.check(ctx("set rfind"), hs.rfind(&set[..]), fh.rfind(&fset[..]));
                m.check(
                    ctx("set split"),
                    pieces(hs.split(&set[..])),
                    owned(fh.split(&fset[..])),
                );
                m.check(
                    ctx("set rsplit"),
                    pieces(hs.rsplit(&set[..])),
                    owned(fh.rsplit(&fset[..])),
                );
                m.check(
                    ctx("set trim_matches"),
                    fold(hs.trim_matches(&set[..]).as_str()),
                    fh.trim_matches(&fset[..]).to_string(),
                );
                m.check(ctx("array find"), hs.find(&set), fh.find(fset));
                m.check(
                    ctx("array split"),
                    pieces(hs.split(&set)),
                    owned(fh.split(fset)),
                );
                m.check(
                    ctx("array trim_start_matches"),
                    fold(hs.trim_start_matches(&set).as_str()),
                    fh.trim_start_matches(fset).to_string(),
                );
            }

            // Closures.
            let ctx = |op: &'static str| move || format!("{h:?}.{op}(|ch| ch == {:?})", c as char);
            m.check(ctx("fn find"), hs.find(|ch: AinChar| ch == cc), fh.find(fc));
            m.check(
                ctx("fn rfind"),
                hs.rfind(|ch: AinChar| ch == cc),
                fh.rfind(fc),
            );
            m.check(
                ctx("fn split"),
                pieces(hs.split(|ch: AinChar| ch == cc)),
                owned(fh.split(fc)),
            );
            m.check(
                ctx("fn rsplit"),
                pieces(hs.rsplit(|ch: AinChar| ch == cc)),
                owned(fh.rsplit(fc)),
            );
            m.check(
                ctx("fn trim_matches"),
                fold(hs.trim_matches(|ch: AinChar| ch == cc).as_str()),
                fh.trim_matches(fc).to_string(),
            );
        }
    }
    m.finish("AinChar, &[AinChar] and FnMut patterns");
}

fn lines_oracle(max_len: usize) {
    let mut m = Mismatches::default();
    for h in &strings(b"a\n\r", max_len) {
        let hs = a(h);
        let mut buf: Vec<AinChar> = hs.chars().collect();
        let ctx = |op: &'static str| move || format!("{h:?}.{op}");
        m.check(ctx("lines"), pieces(hs.lines()), owned(h.lines()));
        m.check(
            ctx("lines rev"),
            pieces(hs.lines().rev()),
            owned(h.lines().rev()),
        );
        m.check(
            ctx("lines alternate"),
            pieces(alternate(hs.lines()).into_iter()),
            owned(alternate(h.lines()).into_iter()),
        );
        let hm = AinStr::from_slice_mut(&mut buf);
        m.check(
            ctx("lines_mut"),
            pieces_mut(hm.lines_mut()),
            owned(h.lines()),
        );
    }
    m.finish("lines");
}

// The alphabet mixes both cases of a letter, a second letter and a non-letter. Under ain's scan
// character ranking '.' < 'b' < 'a', so needles over it put the scanned character at every offset,
// and '.' takes the single-byte memchr path while the letters take the two-byte one.
const ALPHABET: [u8; 4] = *b"aAb.";

#[test]
fn str_patterns_match_std() {
    str_oracle(&ALPHABET, 4, 3);
}

#[test]
fn char_set_and_closure_patterns_match_std() {
    char_oracle(&ALPHABET, 4);
}

#[test]
fn lines_match_std() {
    lines_oracle(6);
}

#[test]
#[ignore = "slow: run with `cargo test --release -- --ignored`"]
fn str_patterns_match_std_large() {
    str_oracle(b"aAbz.", 6, 4);
}

#[test]
#[ignore = "slow: run with `cargo test --release -- --ignored`"]
fn char_set_and_closure_patterns_match_std_large() {
    char_oracle(b"aAbz.", 6);
}

#[test]
#[ignore = "slow: run with `cargo test --release -- --ignored`"]
fn lines_match_std_large() {
    lines_oracle(9);
}

// ------------------------------------------------------------------------------------------------
// Long inputs. The exhaustive tests above only use haystacks of a few bytes, which never leaves
// memchr's short-input fallback and never exercises the chunked case-insensitive compare on
// needles longer than a vector register. These plant needles at every offset of haystacks around
// the 16/32/64-byte boundaries, with the case flipped at every needle position.
// ------------------------------------------------------------------------------------------------

/// Filler that contains none of the needle's characters, so every stop of the scan is a real
/// candidate.
fn filler(len: usize) -> String {
    "xyXY".chars().cycle().take(len).collect()
}

fn flip_case_at(s: &str, i: usize) -> String {
    s.char_indices()
        .map(|(j, c)| if j == i { c.to_ascii_uppercase() } else { c })
        .collect()
}

#[test]
fn long_inputs_match_std() {
    let mut m = Mismatches::default();
    let needles = [
        "ab",
        "elizabeth",
        "pride and prejudice",
        "it is a truth universally acknowledged",
    ];
    for needle in needles {
        let flipped: Vec<String> = (0..needle.len()).map(|i| flip_case_at(needle, i)).collect();
        for &len in &[31usize, 32, 33, 63, 64, 65, 100, 257] {
            if len < needle.len() {
                continue;
            }
            for pos in 0..=len - needle.len() {
                // One occurrence planted at `pos`, with the case of one needle character flipped
                // in the haystack, cycling through every position.
                let planted = &flipped[pos % flipped.len()];
                let mut h = filler(len);
                h.replace_range(pos..pos + needle.len(), planted);
                let h = &h;
                let fh = fold(h);
                let hs = a(h);
                let ns = a(needle);
                let ctx = |op: &'static str| move || format!("{op}({needle:?}) in {h:?}");
                m.check(ctx("find"), hs.find(ns), fh.find(needle));
                m.check(ctx("rfind"), hs.rfind(ns), fh.rfind(needle));
                m.check(ctx("split"), pieces(hs.split(ns)), owned(fh.split(needle)));
                m.check(
                    ctx("rsplit"),
                    pieces(hs.rsplit(ns)),
                    owned(fh.rsplit(needle)),
                );
            }
        }
    }

    // Adjacent, overlapping and back-to-back occurrences on long haystacks.
    for (needle, hay) in [
        (
            "aba",
            format!("{}aba{}ababa{}abaaba", filler(40), filler(20), filler(30)),
        ),
        (
            "aa",
            format!("{}{}{}", filler(33), "a".repeat(70), filler(33)),
        ),
        ("Ab", "ab".repeat(100)),
        ("ab", format!("{}{}", "ab".repeat(100), filler(31))),
        ("zab", format!("{}{}", filler(30), "z".repeat(100) + "ab")),
    ] {
        let hay = &hay;
        let fh = fold(hay);
        let hs = a(hay);
        let ns = a(needle);
        let fnd = fold(needle);
        let fnd = fnd.as_str();
        let ctx = |op: &'static str| move || format!("{op}({needle:?}) in {hay:?}");
        m.check(ctx("find"), hs.find(ns), fh.find(fnd));
        m.check(ctx("rfind"), hs.rfind(ns), fh.rfind(fnd));
        m.check(ctx("split"), pieces(hs.split(ns)), owned(fh.split(fnd)));
        m.check(ctx("rsplit"), pieces(hs.rsplit(ns)), owned(fh.rsplit(fnd)));
        m.check(
            ctx("splitn"),
            pieces(hs.splitn(3, ns)),
            owned(fh.splitn(3, fnd)),
        );
        m.check(
            ctx("rsplitn"),
            pieces(hs.rsplitn(3, ns)),
            owned(fh.rsplitn(3, fnd)),
        );
    }

    // Long needles: a 40-byte and a 300-byte needle inside a 4 KB haystack, found and absent, with
    // one character of case difference.
    let base = filler(4096);
    for needle_len in [40usize, 300] {
        let needle: String = "the quick brown fox jumps over the lazy dog "
            .chars()
            .cycle()
            .take(needle_len)
            .collect();
        for pos in [0usize, 1, 1000, 4096 - needle_len] {
            let mut h = base.clone();
            h.replace_range(
                pos..pos + needle_len,
                &flip_case_at(&needle, needle_len / 2),
            );
            let fh = fold(&h);
            let hs = a(&h);
            let ns = a(&needle);
            let ctx = |op: &'static str| {
                move || format!("{op}(len {needle_len} needle) planted at {pos}")
            };
            m.check(ctx("find"), hs.find(ns), fh.find(needle.as_str()));
            m.check(ctx("rfind"), hs.rfind(ns), fh.rfind(needle.as_str()));
            m.check(ctx("contains"), hs.contains(ns), true);
        }
        let hs = a(&base);
        let ns = a(&needle);
        m.check(
            || format!("absent len {needle_len} needle"),
            hs.find(ns),
            None,
        );
        m.check(
            || format!("absent len {needle_len} needle (rfind)"),
            hs.rfind(ns),
            None,
        );
    }

    // Single characters and character sets on long haystacks, which take different paths
    // (memchr/memchr2 directly, and the u128 set scan).
    let hay: String = format!(
        "{}A{}b{}.{}",
        filler(37),
        filler(64),
        filler(129),
        filler(31)
    );
    let fh = fold(&hay);
    let hs = a(&hay);
    for c in ['a', 'B', '.', 'q'] {
        let cc = ac(c as u8);
        let fc = c.to_ascii_lowercase();
        let ctx = |op: &'static str| move || format!("{op}({c:?}) on long haystack");
        m.check(ctx("find"), hs.find(cc), fh.find(fc));
        m.check(ctx("rfind"), hs.rfind(cc), fh.rfind(fc));
        m.check(ctx("split"), pieces(hs.split(cc)), owned(fh.split(fc)));
        m.check(ctx("rsplit"), pieces(hs.rsplit(cc)), owned(fh.rsplit(fc)));
    }
    let set = [ac(b'a'), ac(b'.')];
    let fset = ['a', '.'];
    m.check(
        || "set split on long haystack".to_string(),
        pieces(hs.split(&set[..])),
        owned(fh.split(&fset[..])),
    );
    m.check(
        || "set rsplit on long haystack".to_string(),
        pieces(hs.rsplit(&set[..])),
        owned(fh.rsplit(&fset[..])),
    );

    m.finish("long inputs");
}
