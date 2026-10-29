//! Mojibake: UTF-8 bytes that were read as a single-byte code page (`ì•ˆë…•` for `안녕`).
//!
//! Recovery runs the misreading backwards: every character goes back to its byte in the code page,
//! and the bytes are read as UTF-8. Both steps are strict. A character the code page does not have,
//! or bytes that are not UTF-8, fail the whole text; U+FFFD, in the text or in the result, means a
//! byte was already replaced, and nothing can say which one, so such text is never reported as
//! recovered.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Codepage {
    Windows1252,
}

pub const CODEPAGES: [Codepage; 1] = [Codepage::Windows1252];

// windows-1252 bytes 0x80-0x9F, as the WHATWG Encoding Standard maps them: the five bytes Microsoft
// leaves undefined (81, 8D, 8F, 90, 9D) keep the C1 control of the same number, so all 256 bytes
// round-trip. 0x00-0x7F and 0xA0-0xFF are the code points of the same number.
const WINDOWS_1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

impl Codepage {
    pub fn name(self) -> &'static str {
        match self {
            Codepage::Windows1252 => "windows-1252",
        }
    }

    pub fn parse(name: &str) -> Option<Codepage> {
        match name.to_ascii_lowercase().as_str() {
            "windows-1252" | "cp1252" => Some(Codepage::Windows1252),
            _ => None,
        }
    }

    /// The byte that shows as `c`, if the code page has `c`.
    fn byte(self, c: char) -> Option<u8> {
        match c as u32 {
            n @ (0..=0x7F | 0xA0..=0xFF) => Some(n as u8),
            _ => WINDOWS_1252_HIGH.iter().position(|&h| h == c).map(|i| 0x80 + i as u8),
        }
    }

    #[cfg(test)]
    fn char_of(self, b: u8) -> char {
        match b {
            0x80..=0x9F => WINDOWS_1252_HIGH[(b - 0x80) as usize],
            _ => b as char,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    /// U+FFFD at this character.
    Lossy(usize),
    /// U+FFFD at this character of the result (`ï¿½` is its three bytes): the text was damaged before
    /// it was misread.
    Damaged(usize),
    NotInCodepage { codepage: Codepage, c: char, at: usize },
    /// The bytes are not UTF-8 from this character on (one byte per character, so the byte offset).
    NotUtf8 { at: usize, incomplete: bool },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Lossy(at) => write!(
                f,
                "lossy: U+FFFD at character {at} means a byte was already replaced; the original cannot be recovered"
            ),
            Error::Damaged(at) => write!(
                f,
                "lossy: the decoded text contains U+FFFD at character {at}: it was already damaged before it was misread"
            ),
            Error::NotInCodepage { codepage, c, at } => {
                write!(f, "'{c}' (U+{:04X}) at character {at} is not in {}", c as u32, codepage.name())
            }
            Error::NotUtf8 { at, incomplete } => {
                let what = if incomplete { "an incomplete UTF-8 sequence" } else { "invalid UTF-8" };
                write!(f, "the bytes are {what} at character {at}")
            }
        }
    }
}

pub fn decode(codepage: Codepage, text: &str) -> Result<String, Error> {
    if let Some(at) = text.chars().position(|c| c == '\u{FFFD}') {
        return Err(Error::Lossy(at));
    }
    let bytes = text
        .chars()
        .enumerate()
        .map(|(at, c)| codepage.byte(c).ok_or(Error::NotInCodepage { codepage, c, at }))
        .collect::<Result<Vec<u8>, Error>>()?;
    let s = String::from_utf8(bytes).map_err(|e| Error::NotUtf8 {
        at: e.utf8_error().valid_up_to(),
        incomplete: e.utf8_error().error_len().is_none(),
    })?;
    match s.chars().position(|c| c == '\u{FFFD}') {
        Some(at) => Err(Error::Damaged(at)),
        None => Ok(s),
    }
}

/// Whether `text` looks like UTF-8 read as `codepage`: some characters map to one complete multi-byte
/// UTF-8 character, a lead byte (C2-F4) and exactly the continuation bytes it needs. An accented letter
/// that happens to be a lead byte before punctuation in 80-BF (`café’s`, `résumé…`, `«Café»`) is not
/// enough, and neither is U+FFFD; plain Korean and ASCII never are.
fn looks_like(codepage: Codepage, text: &str) -> bool {
    let bytes: Vec<Option<u8>> = text.chars().map(|c| codepage.byte(c)).collect();
    bytes.iter().enumerate().any(|(i, b)| {
        let len = match b {
            Some(0xC2..=0xDF) => 2,
            Some(0xE0..=0xEF) => 3,
            Some(0xF0..=0xF4) => 4,
            _ => return false,
        };
        let seq: Option<Vec<u8>> = bytes.get(i..i + len).and_then(|w| w.iter().copied().collect());
        seq.is_some_and(|s| std::str::from_utf8(&s).is_ok())
    })
}

/// The recovered text and why, or why it does not recover.
pub type Outcome = Result<(String, String), String>;

/// What `hanfix inspect` reports for this code page: `None` when the text does not look like
/// mojibake, otherwise its outcome.
pub fn inspect(codepage: Codepage, text: &str) -> Option<Outcome> {
    if !looks_like(codepage, text) {
        return None;
    }
    Some(match decode(codepage, text) {
        Ok(s) => {
            let mut reason = format!(
                "every character is in {0}, and its {1} bytes are valid UTF-8; valid UTF-8 does not prove this is the original text",
                codepage.name(),
                text.chars().count()
            );
            if looks_like(codepage, &s) {
                reason += "; the result looks like mojibake again (decoded twice?)";
            }
            Ok((s, reason))
        }
        Err(e) => Err(e.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Codepage = Codepage::Windows1252;

    /// UTF-8 text as a windows-1252 reader shows it.
    fn misread(text: &str) -> String {
        text.bytes().map(|b| W.char_of(b)).collect()
    }

    #[test]
    fn table_round_trips_every_byte() {
        for b in 0..=255u8 {
            assert_eq!(W.byte(W.char_of(b)), Some(b), "{b:02X}");
        }
        assert_eq!(W.byte('\u{0080}'), None);
        assert_eq!(W.byte('€'), Some(0x80));
    }

    #[test]
    fn recovers_real_cases() {
        assert_eq!(misread("안녕"), "ì•ˆë…•");
        assert_eq!(decode(W, "ì•ˆë…•"), Ok("안녕".to_string()));
        assert_eq!(decode(W, "íŠ¸ë¦¬"), Ok("트리".to_string()));
        assert_eq!(decode(W, "í•œê¸€"), Ok("한글".to_string()));
        assert_eq!(decode(W, "cafÃ©"), Ok("café".to_string()));
        assert_eq!(decode(W, "report_ì•ˆë…•.pdf"), Ok("report_안녕.pdf".to_string()));
        // Bytes 81, 8D, 8F, 90 and 9D show as C1 controls, invisible but present.
        for text in ["안녕하세요", "가나다라마바사아자차카타파하", "회의록 2026-10-29.txt", "😀"] {
            assert_eq!(decode(W, &misread(text)).as_deref(), Ok(text), "{text}");
        }
    }

    #[test]
    fn fails_strictly() {
        assert_eq!(decode(W, "ì•ˆ�"), Err(Error::Lossy(3)));
        assert_eq!(decode(W, "안녕 ì•ˆë…•"), Err(Error::NotInCodepage { codepage: W, c: '안', at: 0 }));
        assert_eq!(decode(W, "ì•ˆë…"), Err(Error::NotUtf8 { at: 3, incomplete: true }));
        assert_eq!(decode(W, "café"), Err(Error::NotUtf8 { at: 3, incomplete: true }));
        assert_eq!(decode(W, "naïve"), Err(Error::NotUtf8 { at: 2, incomplete: false }));
        // A dropped C1 control (copy and paste often loses them) cannot be guessed back.
        // 곁 is EA B3 81.
        assert_eq!(misread("곁"), "ê³\u{81}");
        assert_eq!(decode(W, "ê³"), Err(Error::NotUtf8 { at: 0, incomplete: true }));
        assert_eq!(decode(W, "plain ascii"), Ok("plain ascii".to_string()));
        // `ï¿½` is EF BF BD, U+FFFD: the text was decoded lossily, saved, and then misread.
        assert_eq!(decode(W, "ì•ˆë…•ï¿½"), Err(Error::Damaged(2)));
        assert_eq!(decode(W, "cafÃ©ï¿½"), Err(Error::Damaged(4)));
    }

    #[test]
    fn inspects() {
        for text in [
            "안녕하세요", "hello_world", "café", "naïve", "©2026", "100% done", "",
            // A lead byte before punctuation in 80-BF is not a complete sequence.
            "café’s", "résumé…", "„Café“", "«Café»", "naïve’s", "×÷", "España°", "naïve �",
        ] {
            assert_eq!(inspect(W, text), None, "{text}");
        }
        assert!(matches!(inspect(W, "ì•ˆë…•"), Some(Ok((s, _))) if s == "안녕"));
        assert!(matches!(inspect(W, "Â·"), Some(Ok((s, _))) if s == "·"));
        assert!(matches!(inspect(W, "ðŸ˜€"), Some(Ok((s, _))) if s == "😀"));
        assert!(matches!(inspect(W, "ì•ˆë…"), Some(Err(_))));
        let lossy = inspect(W, "ì•ˆ�").unwrap().unwrap_err();
        assert!(lossy.starts_with("lossy: "));
        // U+FFFD in the result: lossy, and no "decoded twice" note on what is left.
        for text in ["ì•ˆë…•ï¿½", "cafÃ©ï¿½"] {
            let e = inspect(W, text).unwrap().unwrap_err();
            assert!(e.starts_with("lossy: the decoded text contains U+FFFD") && !e.contains("twice"), "{text}");
        }
        // Decoded twice: the first pass gives mojibake again, and the reason says so.
        let twice = misread(&misread("안녕"));
        let (once, reason) = inspect(W, &twice).unwrap().unwrap();
        assert_eq!(once, misread("안녕"));
        assert!(reason.contains("looks like mojibake again"));
    }
}
