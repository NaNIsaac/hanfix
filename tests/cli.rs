//! The binary as a shell sees it: arguments, standard input, standard output and error, exit status.

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn hanfix(args: &[&str], stdin: Option<&[u8]>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_hanfix"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut pipe = child.stdin.take().unwrap();
    pipe.write_all(stdin.unwrap_or_default()).unwrap();
    drop(pipe);
    child.wait_with_output().unwrap()
}

fn stdout(args: &[&str]) -> String {
    let out = hanfix(args, None);
    assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn decode_prints_only_the_result() {
    assert_eq!(stdout(&["decode", "--type", "underscore-hex", "_ED_8A_B8_EB_A6_AC"]), "트리\n");
    assert_eq!(stdout(&["decode", "--type=percent", "%ED%8A%B8%EB%A6%AC"]), "트리\n");
    assert_eq!(stdout(&["decode", "--type", "underscore-hex", "--", "file__ED_8A_B8.txt"]), "file_트.txt\n");
    assert_eq!(stdout(&["decode", "--type", "underscore-hex", "backup_2026_10_29"]), "backup_2026_10_29\n");
    assert_eq!(stdout(&["decode", "--type", "percent", "100%", "done"]), "100% done\n");
}

#[test]
fn decode_reads_stdin() {
    let out = hanfix(&["decode", "--type", "underscore-hex"], Some(b"_ED_8A_B8\n"));
    assert!(out.status.success());
    assert_eq!(out.stdout, "트\n".as_bytes());
    let out = hanfix(&["inspect"], Some(b"\xff_ED"));
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty() && String::from_utf8_lossy(&out.stderr).contains("not UTF-8"));
}

#[test]
fn decode_failure_writes_nothing_to_stdout() {
    for (kind, text) in [("underscore-hex", "_ED_8A"), ("percent", "%ED%8A"), ("percent", "ok %ED%8A%B8 %FF")] {
        let out = hanfix(&["decode", "--type", kind, text], None);
        assert_eq!(out.status.code(), Some(1), "{text}");
        assert!(out.stdout.is_empty(), "{text}");
        assert!(String::from_utf8_lossy(&out.stderr).starts_with("hanfix: error: "), "{text}");
    }
}

#[test]
fn inspect_never_fails() {
    for text in ["hello_world", "100% done", "_ED_8A", "%GG", "트리"] {
        let out = stdout(&["inspect", text]);
        assert!(out.starts_with("Detected: "), "{text}: {out}");
    }
    assert!(stdout(&["inspect", "_ED_8A_B8"]).ends_with("Candidate: 트\n"));
}

#[test]
fn usage_errors() {
    for args in [
        &["decode", "_ED_8A_B8"][..],
        &["decode", "--type", "base64", "x"],
        &["decode", "--type"],
        &["inspect", "--type", "percent", "x"],
        &["inspect", "-x"],
    ] {
        assert_eq!(hanfix(args, None).status.code(), Some(2), "{args:?}");
    }
}

// The words still convert when they are not the first argument: after "--", after an option, and the
// way the shell functions call hanfix ("hanfix --to en -- $last").
#[test]
fn subcommand_words_as_text() {
    let inspect_ko = stdout(&["--to", "ko", "--", "inspect"]);
    assert_eq!(stdout(&["--", "inspect"]), inspect_ko);
    assert_eq!(stdout(&["--", "decode"]), stdout(&["--to", "ko", "--", "decode"]));
    assert!(!inspect_ko.starts_with("Detected"));
    assert_eq!(stdout(&["--to", "en", "--", inspect_ko.trim_end()]), "inspect\n");
    assert_eq!(stdout(&["--to", "en", "--", "decode --type percent x"]), "decode --type percent x\n");
    assert_eq!(stdout(&["ㅑㅜ넫ㅊㅅ"]), "inspect\n");
}

#[test]
fn keyboard_conversion_unchanged() {
    assert_eq!(stdout(&["ㅣㄴ", "-ㅁ"]), "ls -a\n");
    assert_eq!(stdout(&["dkssudgktpdy"]), "안녕하세요\n");
    let out = hanfix(&[], Some("햣 ㄴㅅㅁ션\n".as_bytes()));
    assert_eq!(out.stdout, b"git status\n");
}
