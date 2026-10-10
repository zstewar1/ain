//! Targeted tests for pattern behaviour that the doc examples do not cover: regressions found
//! while developing the searchers, edge cases of the empty needle, the scan-character selection
//! of the substring searcher, and the `_mut` iterators.

use ain::{AinChar, AinStr, ain, ain_mut};

fn a(s: &str) -> &AinStr {
    AinStr::from_str(s).unwrap()
}

fn strs<'a>(it: impl Iterator<Item = &'a AinStr>) -> Vec<&'a str> {
    it.map(AinStr::as_str).collect()
}

fn digit(i: usize) -> AinChar {
    AinChar::from_u8(b'0' + u8::try_from(i).unwrap()).unwrap()
}

#[test]
fn splitn_stops_when_the_pieces_run_out() {
    // Regression: the last allowed piece was emitted as an empty remainder even after the split
    // had already finished, so `"a,b".splitn(3, ',')` yielded a trailing "".
    let s = ain!("a,b");
    assert_eq!(strs(s.splitn(3, ain!(','))), ["a", "b"]);
    assert_eq!(strs(s.rsplitn(3, ain!(','))), ["b", "a"]);
    assert_eq!(strs(s.splitn(2, ain!(','))), ["a", "b"]);
    assert_eq!(strs(s.splitn(1, ain!(','))), ["a,b"]);
    assert!(s.splitn(0, ain!(',')).next().is_none());
    assert_eq!(strs(ain!("a--b").splitn(4, ain!("--"))), ["a", "b"]);

    // A trailing separator does produce a trailing empty piece, with or without a limit to hit.
    assert_eq!(strs(ain!("a,b,").splitn(3, ain!(','))), ["a", "b", ""]);
    assert_eq!(strs(ain!("a,b,").splitn(5, ain!(','))), ["a", "b", ""]);
    assert_eq!(strs(ain!(",a,b").rsplitn(3, ain!(','))), ["b", "a", ""]);

    // The limit cuts the remainder off unsplit.
    assert_eq!(strs(ain!("a,b,c,d").splitn(2, ain!(','))), ["a", "b,c,d"]);
    assert_eq!(strs(ain!("a,b,c,d").rsplitn(2, ain!(','))), ["d", "a,b,c"]);
}

#[test]
fn split_iterators_stay_exhausted() {
    // `FusedIterator` is implemented, so `None` must be sticky in both directions.
    let s = ain!("a,b");

    let mut it = s.split(ain!(','));
    while it.next().is_some() {}
    assert_eq!(it.next(), None);
    assert_eq!(it.next_back(), None);

    let mut it = s.rsplit(ain!(','));
    while it.next().is_some() {}
    assert_eq!(it.next(), None);
    assert_eq!(it.next_back(), None);

    let mut it = s.splitn(5, ain!(','));
    while it.next().is_some() {}
    assert_eq!(it.next(), None);
    assert_eq!(it.next(), None);

    let mut it = s.rsplitn(5, ain!(','));
    while it.next().is_some() {}
    assert_eq!(it.next(), None);
    assert_eq!(it.next(), None);

    let mut it = ain!("a\nb\n").lines();
    while it.next().is_some() {}
    assert_eq!(it.next(), None);
    assert_eq!(it.next_back(), None);
}

#[test]
fn empty_str_needle() {
    let s = ain!("abc");
    let e = ain!("");

    assert_eq!(s.find(e), Some(0));
    assert_eq!(s.rfind(e), Some(3));
    assert!(s.contains(e));
    assert_eq!(strs(s.split(e)), ["", "a", "b", "c", ""]);
    assert_eq!(strs(s.rsplit(e)), ["", "c", "b", "a", ""]);
    assert_eq!(strs(s.splitn(3, e)), ["", "a", "bc"]);
    assert_eq!(strs(s.rsplitn(3, e)), ["", "c", "ab"]);
    assert_eq!(strs(e.split(e)), ["", ""]);
    assert_eq!(strs(e.rsplit(e)), ["", ""]);
    assert_eq!(s.strip_prefix(e), Some(s));
    assert_eq!(s.strip_suffix(e), Some(s));

    // Regression: these looped forever, since an empty needle matches at every position.
    assert_eq!(s.trim_start_matches(e), s);
    assert_eq!(s.trim_end_matches(e), s);
    assert_eq!(e.trim_start_matches(e), e);
    assert_eq!(e.trim_end_matches(e), e);
}

#[test]
fn reverse_search_with_distinct_first_and_last_chars() {
    // Regression: the reverse search scanned for the needle's first character where its last
    // character had to be, so no needle with different first and last characters ever matched
    // backwards.
    assert_eq!(ain!("ab").rfind(ain!("ab")), Some(0));
    assert_eq!(ain!("xabxab").rfind(ain!("AB")), Some(4));
    assert_eq!(
        strs(ain!("abcdabefAbghiaBjab").rsplit(ain!("ab"))),
        ["", "j", "ghi", "ef", "cd", ""]
    );
}

#[test]
fn substring_search_with_the_scan_char_at_each_offset() {
    // The substring searcher scans the haystack for the needle character that is rarest in
    // typical text and verifies the whole needle around each hit. 'z' outranks 'a' and 'b', so
    // these needles scan at offsets 0, 1 and 2 respectively.
    for (needle, hay, fwd, back) in [
        ("zab", "zabzab", Some(0), Some(3)),
        ("azb", "azbazb", Some(0), Some(3)),
        ("abz", "abzabz", Some(0), Some(3)),
        // The scan character is present, but not where a match could put it.
        ("abz", "zab", None, None),
        ("zab", "abz", None, None),
        // Candidates that fail verification before the real match, in each direction.
        ("abz", "xbzabz", Some(3), Some(3)),
        ("zab", "zabxzxb", Some(0), Some(0)),
        // Runs of the scan character exercise the adjacent-candidate path.
        ("zab", "zzzzzab", Some(4), Some(4)),
        ("abz", "abzzzzz", Some(0), Some(0)),
        ("zz", "zzzzz", Some(0), Some(3)),
        ("zb", "zzzzz", None, None),
        // Case-insensitive, needle as long as the haystack, needle longer than the haystack.
        ("ZaB", "zab", Some(0), Some(0)),
        ("zabz", "zab", None, None),
    ] {
        let (n, h) = (a(needle), a(hay));
        assert_eq!(h.find(n), fwd, "find {needle:?} in {hay:?}");
        assert_eq!(h.rfind(n), back, "rfind {needle:?} in {hay:?}");
        assert_eq!(
            h.contains(n),
            fwd.is_some(),
            "contains {needle:?} in {hay:?}"
        );
    }
}

#[test]
fn mut_iterators_write_through_to_the_buffer() {
    ain_mut!(s, "one two three");
    for word in s.split_mut(ain!(' ')) {
        word[0] = ain!('X');
    }
    assert_eq!(&*s, ain!("Xne Xwo Xhree"));

    ain_mut!(s, "a-b-c");
    for (i, piece) in s.rsplit_mut(ain!('-')).enumerate() {
        piece[0] = digit(i);
    }
    assert_eq!(&*s, ain!("2-1-0"));

    ain_mut!(s, "a b c");
    for piece in s.splitn_mut(2, ain!(' ')) {
        piece[0] = ain!('X');
    }
    assert_eq!(&*s, ain!("X X c"));

    ain_mut!(s, "a b c");
    for piece in s.rsplitn_mut(2, ain!(' ')) {
        piece[0] = ain!('X');
    }
    assert_eq!(&*s, ain!("X b X"));

    ain_mut!(s, "ab--cd--ef");
    for piece in s.split_mut(ain!("--")) {
        piece.make_uppercase();
    }
    assert_eq!(s.as_str(), "AB--CD--EF");

    ain_mut!(s, "ab\ncd\r\nef\n");
    for (i, line) in s.lines_mut().enumerate() {
        line[0] = digit(i);
    }
    assert_eq!(s.as_str(), "0b\n1d\r\n2f\n");

    ain_mut!(s, "xxabxx");
    s.trim_matches_mut(ain!('x')).make_uppercase();
    assert_eq!(s.as_str(), "xxABxx");
}

#[test]
fn lines_edge_cases_match_std() {
    for s in [
        "", "\n", "\n\n", "\r\n", "\r", "a", "a\n", "a\r\n", "a\r", "a\r\r\nb", "\ra\n\rb",
        "a\n\nb\n",
    ] {
        assert_eq!(strs(a(s).lines()), s.lines().collect::<Vec<_>>(), "{s:?}");
        assert_eq!(
            strs(a(s).lines().rev()),
            s.lines().rev().collect::<Vec<_>>(),
            "{s:?} rev"
        );
    }
}

#[test]
fn reference_wrappers_behave_like_the_str_pattern() {
    let h = ain!("xxAbxxab");
    let n: &AinStr = ain!("ab");

    assert_eq!(h.find(&n), Some(2));
    assert_eq!(h.rfind(&n), Some(6));
    assert_eq!(strs(h.split(&n)), ["xx", "xx", ""]);
    assert_eq!(h.strip_suffix(&n), Some(ain!("xxAbxx")));
    assert_eq!(h.trim_end_matches(&n), ain!("xxAbxx"));

    #[cfg(feature = "alloc")]
    {
        let owned = ain::AinString::from(n);
        assert_eq!(h.find(&owned), Some(2));
        assert_eq!(h.rfind(&owned), Some(6));
        assert_eq!(strs(h.rsplit(&owned)), ["", "xx", "xx"]);
        assert_eq!(h.strip_prefix(&owned), None);
    }
}

#[test]
fn split_iterators_are_clone_and_debug() {
    fn assert_clone_debug<T: Clone + std::fmt::Debug>(_: &T) {}
    let s = ain!("a,b");
    assert_clone_debug(&s.split(ain!(',')));
    assert_clone_debug(&s.split(ain!(",")));
    assert_clone_debug(&s.rsplit(&[ain!(','), ain!(';')][..]));
    assert_clone_debug(&s.splitn(2, ain!(',')));
    assert_clone_debug(&s.lines());

    // Cloning mid-iteration yields an independent iterator.
    let mut it = ain!("a,b,c").split(ain!(','));
    it.next();
    let copy = it.clone();
    assert_eq!(strs(it), ["b", "c"]);
    assert_eq!(strs(copy), ["b", "c"]);
}

#[test]
fn line_end_pattern() {
    use ain::pattern::LineEnd;

    // Matches "\n" and "\r\n"; a lone "\r" is not a line ending.
    assert_eq!(ain!("ab\ncd").find(LineEnd), Some(2));
    assert_eq!(ain!("ab\r\ncd").find(LineEnd), Some(2));
    assert_eq!(ain!("ab\rcd").find(LineEnd), None);
    assert_eq!(ain!("a\r\nb\n").rfind(LineEnd), Some(4));
    assert_eq!(ain!("a\nb\r\n").rfind(LineEnd), Some(3));
    assert!(ain!("\r\n").contains(LineEnd));
    assert!(!ain!("\r").contains(LineEnd));

    // Unlike `lines()`, `split(LineEnd)` keeps the empty piece after a trailing line ending.
    assert_eq!(strs(ain!("a\nb\r\nc").split(LineEnd)), ["a", "b", "c"]);
    assert_eq!(strs(ain!("a\n").split(LineEnd)), ["a", ""]);
    assert_eq!(strs(ain!("a\n").lines()), ["a"]);
    assert_eq!(strs(ain!("a\r\n\r\nb").split(LineEnd)), ["a", "", "b"]);
    assert_eq!(strs(ain!("a\rb").split(LineEnd)), ["a\rb"]);
    assert_eq!(strs(ain!("\r\n").split(LineEnd)), ["", ""]);
    assert_eq!(strs(ain!("a\nb\r\nc").rsplit(LineEnd)), ["c", "b", "a"]);
    assert_eq!(strs(ain!("a\nb\nc").splitn(2, LineEnd)), ["a", "b\nc"]);

    // Anchored matching is greedy about "\r\n", so a "\r" before "\n" is part of the ending.
    assert_eq!(ain!("a\r\n").strip_suffix(LineEnd), Some(ain!("a")));
    assert_eq!(ain!("a\n").strip_suffix(LineEnd), Some(ain!("a")));
    assert_eq!(ain!("a\r").strip_suffix(LineEnd), None);
    assert_eq!(ain!("\r\n").strip_suffix(LineEnd), Some(ain!("")));
    assert_eq!(ain!("\r\nb").strip_prefix(LineEnd), Some(ain!("b")));
    assert_eq!(ain!("\nb").strip_prefix(LineEnd), Some(ain!("b")));
    assert_eq!(ain!("\rb").strip_prefix(LineEnd), None);

    assert_eq!(ain!("a\n\r\n").trim_end_matches(LineEnd), ain!("a"));
    assert_eq!(ain!("a\r\r\n").trim_end_matches(LineEnd), ain!("a\r"));
    assert_eq!(ain!("a\r").trim_end_matches(LineEnd), ain!("a\r"));
    assert_eq!(ain!("\n\r\n").trim_end_matches(LineEnd), ain!(""));
    assert_eq!(ain!("\r\n\na").trim_start_matches(LineEnd), ain!("a"));
    assert_eq!(ain!("\n\ra").trim_start_matches(LineEnd), ain!("\ra"));
    assert_eq!(ain!("\r\n a \n").trim_matches(LineEnd), ain!(" a "));
}

#[test]
fn large_and_empty_char_sets() {
    // Sets of 16 or more characters take the bitset path in the anchored (trim) operations.
    let text = "0123456789,.;:!?-+";
    let set: Vec<AinChar> = text
        .chars()
        .map(|c| AinChar::from_char(c).unwrap())
        .collect();
    let fset: Vec<char> = text.chars().collect();
    assert!(set.len() >= 16);
    for s in ["12,,abc...de;;f!!g??98", "abc", "", "?!", "a1b2c3"] {
        let hs = a(s);
        assert_eq!(hs.find(&set[..]), s.find(&fset[..]), "{s:?}");
        assert_eq!(hs.rfind(&set[..]), s.rfind(&fset[..]), "{s:?}");
        assert_eq!(
            strs(hs.split(&set[..])),
            s.split(&fset[..]).collect::<Vec<_>>(),
            "{s:?}"
        );
        assert_eq!(
            strs(hs.rsplit(&set[..])),
            s.rsplit(&fset[..]).collect::<Vec<_>>(),
            "{s:?}"
        );
        assert_eq!(
            hs.trim_start_matches(&set[..]).as_str(),
            s.trim_start_matches(&fset[..]),
            "{s:?}"
        );
        assert_eq!(
            hs.trim_end_matches(&set[..]).as_str(),
            s.trim_end_matches(&fset[..]),
            "{s:?}"
        );
        assert_eq!(
            hs.trim_matches(&set[..]).as_str(),
            s.trim_matches(&fset[..]),
            "{s:?}"
        );
    }

    // An empty set never matches, as a slice and as an array.
    let empty: [AinChar; 0] = [];
    let s = ain!("abc");
    assert_eq!(s.find(&empty[..]), None);
    assert_eq!(s.rfind(&empty), None);
    assert_eq!(strs(s.split(&empty[..])), ["abc"]);
    assert_eq!(strs(s.rsplit(&empty)), ["abc"]);
    assert_eq!(s.trim_start_matches(&empty[..]), s);
    assert_eq!(s.trim_matches(&empty), s);
    assert_eq!(s.strip_prefix(&empty[..]), None);
}

#[test]
fn mut_pieces_can_be_held_simultaneously() {
    // Pieces handed out by the `_mut` iterators must not overlap: holding all of them at once and
    // overwriting each with its own index must leave every byte with the right value.
    ain_mut!(s, "ab,cde,,f");
    let pieces: Vec<&mut AinStr> = s.split_mut(ain!(',')).collect();
    assert_eq!(pieces.len(), 4);
    for (i, piece) in pieces.into_iter().enumerate() {
        for j in 0..piece.len() {
            piece[j] = digit(i);
        }
    }
    assert_eq!(s.as_str(), "00,111,,3");

    ain_mut!(s, "ab,cde,,f");
    let pieces: Vec<&mut AinStr> = s.rsplit_mut(ain!(',')).collect();
    for (i, piece) in pieces.into_iter().enumerate() {
        for j in 0..piece.len() {
            piece[j] = digit(i);
        }
    }
    assert_eq!(s.as_str(), "33,222,,0");

    ain_mut!(s, "ab--cde----f");
    let pieces: Vec<&mut AinStr> = s.split_mut(ain!("--")).collect();
    for (i, piece) in pieces.into_iter().enumerate() {
        for j in 0..piece.len() {
            piece[j] = digit(i);
        }
    }
    assert_eq!(s.as_str(), "00--111----3");

    ain_mut!(s, "l1\nl2\r\n\nl4");
    let lines: Vec<&mut AinStr> = s.lines_mut().collect();
    assert_eq!(lines.len(), 4);
    for (i, line) in lines.into_iter().enumerate() {
        for j in 0..line.len() {
            line[j] = digit(i);
        }
    }
    assert_eq!(s.as_str(), "00\n11\r\n\n33");
}

#[test]
fn stateful_closure_patterns_are_called_in_order() {
    // Closures are called once per character, front to back for forward operations and back to
    // front for reverse ones; the docs leave any forward/backward inconsistency to the caller.
    let mut n = 0;
    let every_third = move |_: AinChar| {
        n += 1;
        n % 3 == 0
    };
    assert_eq!(strs(ain!("abcdefg").split(every_third)), ["ab", "de", "g"]);

    let mut n = 0;
    assert_eq!(
        ain!("abcdefg").trim_start_matches(move |_: AinChar| {
            n += 1;
            n <= 2
        }),
        ain!("cdefg")
    );

    let mut n = 0;
    assert_eq!(
        ain!("abcdefg").trim_end_matches(move |_: AinChar| {
            n += 1;
            n <= 2
        }),
        ain!("abcde")
    );

    let mut n = 0;
    assert_eq!(
        ain!("abcdefg").find(move |_: AinChar| {
            n += 1;
            n == 5
        }),
        Some(4)
    );

    let mut n = 0;
    assert_eq!(
        ain!("abcdefg").rfind(move |_: AinChar| {
            n += 1;
            n == 2
        }),
        Some(5)
    );
}
