//! Escaped text back to UTF-8: `_XX` (underscore-hex) and `%XX` (percent) byte escapes.
//!
//! Only escape runs change; every other character is copied as it is. A run of escapes is one byte
//! sequence that must be complete, valid UTF-8, or the whole decode fails: nothing is replaced with
//! U+FFFD and nothing is guessed.

use std::fmt;

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
pub struct DecodeError {
    pub kind: Kind,
    /// The escape run that failed, as written in the input.
    pub run: String,
    /// Characters before the run.
    pub at: usize,
    pub incomplete: bool,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let what = if self.incomplete { "incomplete UTF-8 sequence" } else { "invalid UTF-8" };
        write!(f, "{what} in {} escape '{}' at character {}", self.kind.name(), self.run, self.at)
    }
}

#[derive(Debug, PartialEq)]
pub struct Decoded {
    pub text: String,
    /// Escaped bytes found; 0 means the text had nothing to decode and is returned unchanged.
    pub bytes: usize,
}

pub fn decode(kind: Kind, text: &str) -> Result<Decoded, DecodeError> {
    let b = text.as_bytes();
    let (mut out, mut bytes, mut i, mut copied) = (String::with_capacity(text.len()), 0, 0, 0);
    while i < b.len() {
        if kind.escape_at(b, i).is_none() {
            i += 1;
            continue;
        }
        // Markers and hex digits are ASCII, so `i` is a char boundary here.
        out.push_str(&text[copied..i]);
        let start = i;
        let mut run = Vec::new();
        while let Some(v) = kind.escape_at(b, i) {
            run.push(v);
            i += 3;
        }
        bytes += run.len();
        match std::str::from_utf8(&run) {
            Ok(s) => out.push_str(s),
            Err(e) => {
                return Err(DecodeError {
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
    Ok(Decoded { text: out, bytes })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Valid,
    /// Valid on its own, but the text also has escapes of the other kind, which this one leaves alone.
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
    pub kind: Kind,
    pub status: Status,
    /// `None` when the escapes do not decode.
    pub decoded: Option<String>,
    pub reason: String,
}

/// Every kind of escape found in `text`, each decoded on its own. Empty for plain text.
pub fn inspect(text: &str) -> Vec<Candidate> {
    let found: Vec<_> = KINDS.into_iter().map(|k| (k, decode(k, text))).filter(|(_, r)| r.as_ref().map_or(true, |d| d.bytes > 0)).collect();
    let several = found.len() > 1;
    found
        .into_iter()
        .map(|(kind, r)| match r {
            Ok(d) => Candidate {
                kind,
                status: if several { Status::Ambiguous } else { Status::Valid },
                reason: if several {
                    format!("{} escaped bytes are valid UTF-8, but the text also has other escapes, which this type keeps", d.bytes)
                } else {
                    format!("{} escaped bytes are valid UTF-8; other characters are kept", d.bytes)
                },
                decoded: Some(d.text),
            },
            Err(e) => Candidate { kind, status: Status::Invalid, decoded: None, reason: e.to_string() },
        })
        .collect()
}

/// The `hanfix inspect` report.
pub fn report(text: &str) -> String {
    let found = inspect(text);
    if found.is_empty() {
        return format!(
            "Detected: none\nReason: no underscore-hex or percent escape found; the text is kept as is\n\nOriginal:  {text}\n"
        );
    }
    let blocks: Vec<String> = found
        .iter()
        .map(|c| {
            let mut s = format!(
                "Detected: {}\nStatus: {}\nEncoding: UTF-8\nReason: {}\n\nOriginal:  {text}\n",
                c.kind.name(),
                c.status.name(),
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

    #[test]
    fn bad_sequences_fail() {
        let e = decode(Kind::UnderscoreHex, "_ED_8A").unwrap_err();
        assert!(e.incomplete && e.run == "_ED_8A" && e.at == 0);
        let e = decode(Kind::Percent, "x%ED%8A").unwrap_err();
        assert!(e.incomplete && e.run == "%ED%8A" && e.at == 1);
        let e = decode(Kind::Percent, "%FF").unwrap_err();
        assert!(!e.incomplete);
        let e = decode(Kind::UnderscoreHex, "트_8A").unwrap_err();
        assert_eq!((e.at, e.incomplete), (1, false));
        // A good run does not save a later bad one.
        assert!(decode(Kind::UnderscoreHex, "_ED_8A_B8 _ED").is_err());
        assert_eq!(
            decode(Kind::UnderscoreHex, "_ED_8A").unwrap_err().to_string(),
            "incomplete UTF-8 sequence in underscore-hex escape '_ED_8A' at character 0"
        );
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
    }
}
