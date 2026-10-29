//! Escaped text back to UTF-8: `_XX` (underscore-hex) and `%XX` (percent) byte escapes.
//!
//! Only escape runs change; every other character is copied as it is. A run of escapes is one byte
//! sequence that must be complete, valid UTF-8, or the whole decode fails: nothing is replaced with
//! U+FFFD and nothing is guessed. A result that contains U+FFFD fails too, since an escaped `EF BF BD`
//! means the text was damaged before it was escaped.

use std::fmt;

use crate::mojibake;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    UnderscoreHex,
    Percent,
}

pub const KINDS: [Kind; 2] = [Kind::UnderscoreHex, Kind::Percent];

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::UnderscoreHex => "underscore-hex",
            Kind::Percent => "percent",
        }
    }

    pub fn parse(name: &str) -> Option<Kind> {
        KINDS.into_iter().find(|k| k.name() == name)
    }

    /// The byte escaped at `i`, if an escape starts there.
    ///
    /// Underscore-hex takes upper-case digits and bytes 0x80-0xFF only: ASCII never needs the escape, and
    /// ordinary names are full of `_20`, `_10` or `_be` (`backup_2026_10_29`, `hello_bed`) that are not one.
    /// Percent follows RFC 3986: any byte, either case. A marker without two hex digits after it is text.
    fn escape_at(self, b: &[u8], i: usize) -> Option<u8> {
        let (marker, hex): (u8, fn(u8) -> bool) = match self {
            Kind::UnderscoreHex => (b'_', |c| c.is_ascii_digit() || (b'A'..=b'F').contains(&c)),
            Kind::Percent => (b'%', |c| c.is_ascii_hexdigit()),
        };
        if b.get(i) != Some(&marker) || !b.get(i + 1).is_some_and(|&c| hex(c)) || !b.get(i + 2).is_some_and(|&c| hex(c)) {
            return None;
        }
        let v = u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).ok()?, 16).ok()?;
        (self == Kind::Percent || v >= 0x80).then_some(v)
    }
}

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    Run {
        kind: Kind,
        /// The escape run that failed, as written in the input.
        run: String,
        /// Characters before the run.
        at: usize,
        incomplete: bool,
    },
    /// The decoded text has U+FFFD at character `at`: the bytes were damaged before they were escaped,
    /// so the result is not the original. `bytes` as in `Decoded`.
    Lossy { at: usize, bytes: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeError::Run { kind, run, at, incomplete } => {
                let what = if *incomplete { "incomplete UTF-8 sequence" } else { "invalid UTF-8" };
                // A run can be megabytes long on standard input; the start and the length locate it.
                let run = if run.len() > 40 {
                    format!("{}...' ({} characters)", &run[..40], run.len())
                } else {
                    format!("{run}'")
                };
                write!(f, "{what} in {} escape '{run} at character {at}", kind.name())
            }
            DecodeError::Lossy { at, .. } => write!(
                f,
                "lossy: the decoded text contains U+FFFD at character {at}: it was already damaged before it was escaped"
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Decoded {
    pub text: String,
    /// Escaped bytes found; 0 means the text had nothing to decode and is returned unchanged.
    pub bytes: usize,
}

/// Bytes that follow a UTF-8 lead byte; 0 for a byte no character starts with (80-BF, C0, C1, F5-FF).
fn continuation_bytes(lead: u8) -> usize {
    match lead {
        0x00..=0x7F => 0,
        0xC2..=0xDF => 1,
        0xE0..=0xEF => 2,
        0xF0..=0xF4 => 3,
        _ => usize::MAX,
    }
}

impl Kind {
    /// An escape that may start a run. In underscore-hex a byte no UTF-8 character starts with is text:
    /// `IMG_8123` and `test_99` are names, not escapes. Percent escapes are always escapes.
    fn starts_run(self, b: &[u8], i: usize) -> bool {
        self.escape_at(b, i).is_some_and(|v| self == Kind::Percent || continuation_bytes(v) != usize::MAX)
    }
}

pub fn decode(kind: Kind, text: &str) -> Result<Decoded, DecodeError> {
    let b = text.as_bytes();
    let (mut out, mut bytes, mut i, mut copied) = (String::with_capacity(text.len()), 0, 0, 0);
    while i < b.len() {
        if !kind.starts_run(b, i) {
            i += 1;
            continue;
        }
        // Markers and hex digits are ASCII, so `i` is a char boundary here.
        out.push_str(&text[copied..i]);
        let start = i;
        let mut run = Vec::new();
        // Continuation bytes the current character still needs. At a character start, an underscore-hex
        // byte that cannot begin one ends the run (`_ED_8A_B8_99` is 트 then `_99`); inside a character
        // every escape is taken, so a bad one is reported rather than left as text.
        let mut need = 0;
        while let Some(v) = kind.escape_at(b, i) {
            if need == 0 {
                if !kind.starts_run(b, i) {
                    break;
                }
                need = continuation_bytes(v).min(3);
            } else {
                need -= 1;
            }
            run.push(v);
            i += 3;
        }
        bytes += run.len();
        match std::str::from_utf8(&run) {
            Ok(s) => out.push_str(s),
            Err(e) => {
                return Err(DecodeError::Run {
                    kind,
                    run: text[start..i].to_string(),
                    at: text[..start].chars().count(),
                    incomplete: e.error_len().is_none(),
                })
            }
        }
        copied = i;
    }
    out.push_str(&text[copied..]);
    if let Some(at) = out.chars().position(|c| c == '\u{FFFD}') {
        return Err(DecodeError::Lossy { at, bytes });
    }
    Ok(Decoded { text: out, bytes })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Valid,
    /// Valid on its own, but the text also fits another type, which this one leaves alone.
    Ambiguous,
    Invalid,
}

impl Status {
    fn name(self) -> &'static str {
        match self {
            Status::Valid => "valid",
            Status::Ambiguous => "ambiguous",
            Status::Invalid => "invalid",
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Candidate {
    /// The `--type` that decodes it.
    pub detected: &'static str,
    /// How the text came to be: escaped UTF-8, or UTF-8 read as a code page.
    pub encoding: String,
    pub status: Status,
    /// `None` when it does not decode.
    pub decoded: Option<String>,
    pub reason: String,
}

/// Every type the text fits, each decoded on its own: escapes of either kind, and mojibake per code
/// page. Empty for plain text, including plain Korean.
pub fn inspect(text: &str) -> Vec<Candidate> {
    let mut found: Vec<(&'static str, String, mojibake::Outcome)> = Vec::new();
    for kind in KINDS {
        match decode(kind, text) {
            Ok(Decoded { bytes: 0, .. }) | Err(DecodeError::Lossy { bytes: 0, .. }) => {}
            Ok(d) => found.push((
                kind.name(),
                "UTF-8".into(),
                Ok((d.text, format!("{} escaped bytes are valid UTF-8; other characters are kept", d.bytes))),
            )),
            Err(e) => found.push((kind.name(), "UTF-8".into(), Err(e.to_string()))),
        }
    }
    for cp in mojibake::CODEPAGES {
        if let Some(r) = mojibake::inspect(cp, text) {
            found.push(("mojibake", format!("UTF-8 read as {}", cp.name()), r));
        }
    }
    let several = found.len() > 1;
    found
        .into_iter()
        .map(|(detected, encoding, r)| match r {
            Ok((decoded, reason)) => Candidate {
                detected,
                encoding,
                status: if several { Status::Ambiguous } else { Status::Valid },
                reason: if several { reason + "; the text also fits another type, which this one does not decode" } else { reason },
                decoded: Some(decoded),
            },
            Err(reason) => Candidate { detected, encoding, status: Status::Invalid, decoded: None, reason },
        })
        .collect()
}

/// Two `_xx` in a row, each 0x80 or more, with a lower-case hex letter: what an upper-case escape run
/// would be. One such byte alone is never UTF-8, and `hello_bed` should not get the note.
fn has_lower_case_underscore(text: &str) -> bool {
    let byte = |w: &[u8]| {
        w[0] == b'_' && w[1] >= b'8' && w[1..3].iter().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(c))
    };
    text.as_bytes().windows(6).any(|w| byte(&w[..3]) && byte(&w[3..]) && w[1..].iter().any(|c| c.is_ascii_lowercase()))
}

/// The `hanfix inspect` report.
pub fn report(text: &str) -> String {
    let found = inspect(text);
    let note = if has_lower_case_underscore(text) {
        "Note: lower-case _xx is not treated as an escape (underscore-hex is upper case only)\n"
    } else {
        ""
    };
    if found.is_empty() {
        return format!(
            "Detected: none\nReason: no underscore-hex or percent escape and no mojibake found; the text is kept as is\n{note}\nOriginal:  {text}\n"
        );
    }
    let blocks: Vec<String> = found
        .iter()
        .map(|c| {
            let mut s = format!(
                "Detected: {}\nStatus: {}\nEncoding: {}\nReason: {}\n{note}\nOriginal:  {text}\n",
                c.detected,
                c.status.name(),
                c.encoding,
                c.reason
            );
            if let Some(d) = &c.decoded {
                s += &format!("Candidate: {d}\n");
            }
            s
        })
        .collect();
    blocks.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(kind: Kind, text: &str) -> String {
        decode(kind, text).unwrap().text
    }

    #[test]
    fn decodes() {
        assert_eq!(ok(Kind::UnderscoreHex, "_ED_8A_B8"), "트");
        assert_eq!(ok(Kind::UnderscoreHex, "_ED_8A_B8_EB_A6_AC"), "트리");
        assert_eq!(ok(Kind::Percent, "%ED%8A%B8"), "트");
        assert_eq!(ok(Kind::Percent, "%ED%8A%B8%EB%A6%AC"), "트리");
        assert_eq!(ok(Kind::Percent, "%ed%8a%b8"), "트");
        assert_eq!(ok(Kind::Percent, "a%20b"), "a b");
    }

    #[test]
    fn keeps_the_text_around_escapes() {
        assert_eq!(ok(Kind::UnderscoreHex, "file__ED_8A_B8.txt"), "file_트.txt");
        assert_eq!(ok(Kind::UnderscoreHex, "report__ED_8A_B8.pdf"), "report_트.pdf");
        assert_eq!(ok(Kind::UnderscoreHex, "_ED_8A_B8x_EB_A6_AC"), "트x리");
        assert_eq!(ok(Kind::UnderscoreHex, "a__b___ED_8A_B8__"), "a__b__트__");
        assert_eq!(ok(Kind::Percent, "%ED%8A%B8%EB%A6%AC_ED_8A_B8"), "트리_ED_8A_B8");
        assert_eq!(ok(Kind::UnderscoreHex, "%ED%8A%B8_EB_A6_AC"), "%ED%8A%B8리");
        assert_eq!(ok(Kind::Percent, "a+b%2Bc"), "a+b+c");
    }

    #[test]
    fn plain_text_is_unchanged() {
        for text in ["hello_world", "backup_2026_10_29", "100% done", "normal.txt", "hello_bed", "_", "%", "%G", "%GG", "_ed_8a", "트리", ""] {
            for kind in KINDS {
                if kind == Kind::Percent && text == "%GG" {
                    continue;
                }
                assert_eq!(decode(kind, text), Ok(Decoded { text: text.to_string(), bytes: 0 }), "{text}");
            }
        }
        assert_eq!(ok(Kind::Percent, "%GG"), "%GG");
    }

    /// The failed run, its position and whether it was incomplete.
    fn run_error(kind: Kind, text: &str) -> (String, usize, bool) {
        match decode(kind, text) {
            Err(DecodeError::Run { run, at, incomplete, .. }) => (run, at, incomplete),
            r => panic!("{text}: {r:?}"),
        }
    }

    #[test]
    fn bad_sequences_fail() {
        assert_eq!(run_error(Kind::UnderscoreHex, "_ED_8A"), ("_ED_8A".into(), 0, true));
        assert_eq!(run_error(Kind::Percent, "x%ED%8A"), ("%ED%8A".into(), 1, true));
        assert_eq!(run_error(Kind::Percent, "%FF"), ("%FF".into(), 0, false));
        // A lead byte with what follows it missing still fails, also before plain text.
        assert_eq!(run_error(Kind::UnderscoreHex, "트리_DB_backup.sql"), ("_DB".into(), 2, true));
        // Inside a character every escape is taken, so a wrong one is reported, not kept as text.
        assert_eq!(run_error(Kind::UnderscoreHex, "_ED_C3_A9"), ("_ED_C3_A9".into(), 0, false));
        assert!(decode(Kind::UnderscoreHex, "_ED_A0_80").is_err());
        // A good run does not save a later bad one.
        assert!(decode(Kind::UnderscoreHex, "_ED_8A_B8 _ED").is_err());
        assert_eq!(
            decode(Kind::UnderscoreHex, "_ED_8A").unwrap_err().to_string(),
            "incomplete UTF-8 sequence in underscore-hex escape '_ED_8A' at character 0"
        );
        let long = "_ED_8A_B8".repeat(1000) + "_ED";
        assert_eq!(
            decode(Kind::UnderscoreHex, &long).unwrap_err().to_string(),
            "incomplete UTF-8 sequence in underscore-hex escape '_ED_8A_B8_ED_8A_B8_ED_8A_B8_ED_8A_B8_ED_...' \
             (9003 characters) at character 0"
        );
    }

    #[test]
    fn replacement_character_in_the_result_fails() {
        // EF BF BD is U+FFFD: the bytes were already damaged when they were escaped.
        for (kind, text, at) in [
            (Kind::Percent, "%EF%BF%BD", 0),
            (Kind::Percent, "%EC%95%88%EF%BF%BD", 1),
            (Kind::UnderscoreHex, "_EF_BF_BD", 0),
            (Kind::UnderscoreHex, "x_EC_95_88_EF_BF_BD.txt", 2),
            (Kind::Percent, "a\u{FFFD} %41", 1),
        ] {
            assert!(matches!(decode(kind, text), Err(DecodeError::Lossy { at: a, .. }) if a == at), "{text}");
        }
        assert_eq!(decode(Kind::Percent, "\u{FFFD}"), Err(DecodeError::Lossy { at: 0, bytes: 0 }));
        assert_eq!(
            decode(Kind::Percent, "%EF%BF%BD").unwrap_err().to_string(),
            "lossy: the decoded text contains U+FFFD at character 0: it was already damaged before it was escaped"
        );
        let c = inspect("%EC%95%88%EF%BF%BD");
        assert!(c.len() == 1 && c[0].status == Status::Invalid && c[0].reason.starts_with("lossy: "));
        // Plain text with U+FFFD and no escapes is not an escape candidate.
        assert!(inspect("a\u{FFFD}b").is_empty());
    }

    #[test]
    fn underscore_bytes_that_start_no_character_are_text() {
        for text in ["IMG_8123.JPG", "DSC_9001.JPG", "test_99.txt", "_8A", "트_8A", "_C0_80", "_C1_BF", "_F5_80_80_80", "_FF"] {
            assert_eq!(decode(Kind::UnderscoreHex, text), Ok(Decoded { text: text.to_string(), bytes: 0 }), "{text}");
        }
        assert_eq!(ok(Kind::UnderscoreHex, "_ED_8A_B8_99.txt"), "트_99.txt");
        assert_eq!(ok(Kind::UnderscoreHex, "IMG_8123__ED_8A_B8.jpg"), "IMG_8123_트.jpg");
        assert_eq!(ok(Kind::UnderscoreHex, "_ED_8A_B8_99_EB_A6_AC"), "트_99리");
        assert_eq!(ok(Kind::UnderscoreHex, "_C3_A9_F0_9F_98_80"), "é😀");
        assert!(inspect("IMG_8123.JPG").is_empty());
        // Percent stays strict: every %XX is an escape.
        assert!(decode(Kind::Percent, "%ED%8A%B8%99").is_err());
    }

    #[test]
    fn inspects() {
        assert!(inspect("hello_world").is_empty());
        assert_eq!(
            report("_ED_8A_B8_EB_A6_AC"),
            "Detected: underscore-hex\nStatus: valid\nEncoding: UTF-8\n\
             Reason: 6 escaped bytes are valid UTF-8; other characters are kept\n\n\
             Original:  _ED_8A_B8_EB_A6_AC\nCandidate: 트리\n"
        );
        let c = inspect("_ED_8A");
        assert_eq!((c.len(), c[0].status, c[0].decoded.as_deref()), (1, Status::Invalid, None));
        let c = inspect("%ED%8A%B8_EB_A6_AC");
        assert!(c.len() == 2 && c.iter().all(|c| c.status == Status::Ambiguous));
        assert!(report("100% done").starts_with("Detected: none\n"));
        let note = "Note: lower-case _xx is not treated as an escape";
        assert!(report("_ed_8a_b8").contains(note));
        assert!(report("%ED%8A%B8_eb_a6_ac").contains(note));
        for text in ["hello_bed", "backup_2026_10_29", "_ED_8A_B8", "a_7f"] {
            assert!(!report(text).contains(note), "{text}");
        }
    }
}
