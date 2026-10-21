//! hanfix: fix commands and text typed in the wrong keyboard layout (Korean 2-set <-> QWERTY).

use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

// 두벌식 (KS X 5002): key -> jamo. Shifted keys that are not listed fall back to the lower-case key.
const KEYS: &str = "rRseEfaqQtTdwWczxvgkoiOjpuPhynbml";
const JAMO: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎㅏㅐㅑㅒㅓㅔㅕㅖㅗㅛㅜㅠㅡㅣ";
const CHO: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
const JUNG: &str = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ";
const JONG: &str = "ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ"; // index + 1; 0 = no final
const VOWEL_PAIRS: [(char, char, char); 7] = [
    ('ㅗ', 'ㅏ', 'ㅘ'), ('ㅗ', 'ㅐ', 'ㅙ'), ('ㅗ', 'ㅣ', 'ㅚ'), ('ㅜ', 'ㅓ', 'ㅝ'),
    ('ㅜ', 'ㅔ', 'ㅞ'), ('ㅜ', 'ㅣ', 'ㅟ'), ('ㅡ', 'ㅣ', 'ㅢ'),
];
const FINAL_PAIRS: [(char, char, char); 11] = [
    ('ㄱ', 'ㅅ', 'ㄳ'), ('ㄴ', 'ㅈ', 'ㄵ'), ('ㄴ', 'ㅎ', 'ㄶ'), ('ㄹ', 'ㄱ', 'ㄺ'), ('ㄹ', 'ㅁ', 'ㄻ'),
    ('ㄹ', 'ㅂ', 'ㄼ'), ('ㄹ', 'ㅅ', 'ㄽ'), ('ㄹ', 'ㅌ', 'ㄾ'), ('ㄹ', 'ㅍ', 'ㄿ'), ('ㄹ', 'ㅎ', 'ㅀ'),
    ('ㅂ', 'ㅅ', 'ㅄ'),
];
const NO_FINAL: [char; 3] = ['ㄸ', 'ㅃ', 'ㅉ']; // these never close a syllable
const BASE: u32 = 0xAC00;

fn pos(set: &str, c: char) -> Option<usize> {
    set.chars().position(|x| x == c)
}

fn key_to_jamo(k: char) -> Option<char> {
    let i = pos(KEYS, k).or_else(|| pos(KEYS, k.to_ascii_lowercase()))?;
    JAMO.chars().nth(i)
}

fn join(pairs: &[(char, char, char)], a: char, b: char) -> Option<char> {
    pairs.iter().find(|p| p.0 == a && p.1 == b).map(|p| p.2)
}

fn split(c: char) -> Option<(char, char)> {
    VOWEL_PAIRS.iter().chain(FINAL_PAIRS.iter()).find(|p| p.2 == c).map(|p| (p.0, p.1))
}

fn is_hangul(c: char) -> bool {
    ('가'..='힣').contains(&c) || ('ㄱ'..='ㅣ').contains(&c)
}

/// QWERTY keystrokes -> Hangul, with the usual 2-set automaton (a final consonant moves on before a vowel).
pub fn compose(keys: &str) -> String {
    let mut out = String::new();
    let (mut cho, mut jung, mut jong): (Option<char>, Option<char>, Option<char>) = (None, None, None);
    let flush = |out: &mut String, cho: &mut Option<char>, jung: &mut Option<char>, jong: &mut Option<char>| {
        match (*cho, *jung) {
            (Some(c), Some(v)) => {
                let t = jong.map_or(0, |j| pos(JONG, j).unwrap() + 1) as u32;
                let n = (pos(CHO, c).unwrap() * 21 + pos(JUNG, v).unwrap()) as u32 * 28 + t;
                out.push(char::from_u32(BASE + n).unwrap());
            }
            _ => out.extend(cho.iter().chain(jung.iter())),
        }
        (*cho, *jung, *jong) = (None, None, None);
    };
    for k in keys.chars() {
        let Some(j) = key_to_jamo(k) else {
            flush(&mut out, &mut cho, &mut jung, &mut jong);
            out.push(k);
            continue;
        };
        if pos(CHO, j).is_some() {
            if cho.is_some() && jung.is_some() && jong.is_none() && !NO_FINAL.contains(&j) {
                jong = Some(j);
            } else if let Some(pair) = jong.and_then(|f| join(&FINAL_PAIRS, f, j)) {
                jong = Some(pair);
            } else {
                flush(&mut out, &mut cho, &mut jung, &mut jong);
                cho = Some(j);
            }
        } else if let Some(f) = jong {
            // the last final consonant starts the next syllable
            let (first, moved) = split(f).map_or((None, f), |(a, b)| (Some(a), b));
            jong = first;
            flush(&mut out, &mut cho, &mut jung, &mut jong);
            (cho, jung) = (Some(moved), Some(j));
        } else if let Some(pair) = jung.and_then(|v| join(&VOWEL_PAIRS, v, j)) {
            jung = Some(pair);
        } else if cho.is_some() && jung.is_none() {
            jung = Some(j);
        } else {
            flush(&mut out, &mut cho, &mut jung, &mut jong);
            jung = Some(j);
        }
    }
    flush(&mut out, &mut cho, &mut jung, &mut jong);
    out
}

fn jamo_keys(out: &mut String, j: char) {
    let parts = split(j).map_or(vec![j], |(a, b)| vec![a, b]);
    for p in parts {
        match pos(JAMO, p) {
            Some(i) => out.push(KEYS.chars().nth(i).unwrap()),
            None => out.push(p),
        }
    }
}

/// Hangul -> the QWERTY keystrokes that type it.
pub fn decompose(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if ('가'..='힣').contains(&c) {
            let n = c as u32 - BASE;
            jamo_keys(&mut out, CHO.chars().nth((n / 588) as usize).unwrap());
            jamo_keys(&mut out, JUNG.chars().nth((n / 28 % 21) as usize).unwrap());
            if n % 28 > 0 {
                jamo_keys(&mut out, JONG.chars().nth((n % 28 - 1) as usize).unwrap());
            }
        } else if ('ㄱ'..='ㅣ').contains(&c) {
            jamo_keys(&mut out, c);
        } else {
            out.push(c);
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Ko,
    En,
}

fn fix(text: &str, to: Option<Dir>) -> String {
    match to.unwrap_or(if text.chars().any(is_hangul) { Dir::En } else { Dir::Ko }) {
        Dir::En => decompose(text),
        Dir::Ko => compose(text),
    }
}

// Shell functions that fix and re-run the previous command. Each defines the function __HF__ and an
// alias __KO__: what typing the name gives when the Korean layout is still on (hf -> ㅗㄹ).
// The previous command is the last history entry that is not a call of the function itself, so it
// works whether or not the shell has already recorded the current line.
const SHELLS: [&str; 4] = ["bash", "zsh", "fish", "powershell"];

const BASH: &str = r#"__HF__() {
    local last fixed
    last=$(fc -ln -3 -1 2>/dev/null | sed 's/^[[:space:]]*//' | grep -v -E '^(__HF__|__KO__)([[:space:]]|$)' | tail -n 1)
    [ -z "$last" ] && { echo 'hanfix: no previous command' >&2; return 1; }
    fixed=$(hanfix --to en -- "$last") || return
    [ "$fixed" = "$last" ] && { echo 'hanfix: nothing to fix' >&2; return 1; }
    if [ "$1" != -y ] && [ "$1" != --yes ]; then
        printf '\033[32m%s\033[0m  [Enter / Ctrl+C] ' "$fixed"; read -r _ || return 130
    fi
    __HIST__ "$fixed"
    eval "$fixed"
}
alias __KO__=__HF__
"#;

const FISH: &str = r#"function __HF__ --description 'hanfix: fix and re-run the previous command'
    set -l last
    for h in $history[1..3]
        if not string match -qr '^\s*(__HF__|__KO__)(\s|$)' -- $h
            set last $h
            break
        end
    end
    if test -z "$last"
        echo 'hanfix: no previous command' >&2; return 1
    end
    set -l fixed (hanfix --to en -- $last); or return
    if test "$fixed" = "$last"
        echo 'hanfix: nothing to fix' >&2; return 1
    end
    if not contains -- -y $argv; and not contains -- --yes $argv
        set_color green; printf '%s' $fixed; set_color normal
        read -P '  [Enter / Ctrl+C] ' _; or return 130
    end
    # no history entry: fish 3 has no `history append` (an unknown subcommand opens the help in a pager)
    eval $fixed
end
alias __KO__ __HF__
"#;

// PowerShell decodes a native command's output with the console code page (cp949 on Korean Windows),
// so the script itself stays ASCII (the Korean alias is built from code points) and the function reads
// hanfix's UTF-8 output with the UTF-8 decoder switched on.
const POWERSHELL: &str = r#"function global:__HF__ {
    param([Alias('y')][switch]$Yes)
    # Codigest's capture hook hands the engine `. __cdg { <line> }`, and history keeps that form.
    $last = Get-History | Select-Object -Last 3 | ForEach-Object CommandLine |
        ForEach-Object { $_ -replace '^\. __cdg \{ ([\s\S]*?)(?: |\n)\}$', '$1' } |
        Where-Object { $_ -notmatch '^\s*(__HF__|__KO_RE__)(\s|$)' } | Select-Object -Last 1
    if (-not $last) { Write-Host 'hanfix: no previous command'; return }
    $enc = [Console]::OutputEncoding
    try {
        [Console]::OutputEncoding = [Text.Encoding]::UTF8
        $fixed = (hanfix --to en -- $last | Out-String).TrimEnd()
    } finally { [Console]::OutputEncoding = $enc }
    if ($fixed -eq $last) { Write-Host 'hanfix: nothing to fix'; return }
    if (-not $Yes) {
        Write-Host "$fixed  [Enter / Ctrl+C] " -ForegroundColor Green -NoNewline
        $null = [Console]::ReadLine()
    }
    try { [Microsoft.PowerShell.PSConsoleReadLine]::AddToHistory($fixed) } catch {}
    Invoke-Expression $fixed
}
Set-Alias -Name (__KO_PS__) -Value __HF__ -Scope Global
"#;

fn init_script(shell: &str, name: &str) -> String {
    let ko = fix(name, Some(Dir::Ko));
    match shell {
        "bash" => BASH.replace("__HIST__", "history -s --"),
        "zsh" => BASH.replace("__HIST__", "print -s --"),
        "fish" => FISH.to_string(),
        _ => POWERSHELL
            .replace("__KO_RE__", &ko.chars().map(|c| format!("\\u{:04X}", c as u32)).collect::<String>())
            .replace(
                "__KO_PS__",
                &ko.chars().map(|c| format!("[string][char]0x{:04X}", c as u32)).collect::<Vec<_>>().join(" + "),
            ),
    }
    .replace("__HF__", name)
    .replace("__KO__", &ko)
}

const USAGE: &str = "usage: hanfix [-h] [-V] [--to {ko,en}] [--] [TEXT ...]
       hanfix --init {bash,zsh,fish,powershell} [--alias NAME]
       hanfix --update
";

fn help() -> String {
    format!(
        "{USAGE}
Fix commands and text typed in the wrong keyboard layout (Korean 2-set <-> QWERTY).

  hanfix ㅣㄴ -ㅁ              ls -a
  hanfix dkssudgktpdy         안녕하세요
  echo 햣 ㄴㅅㅁ션 | hanfix     git status

The direction is automatic: text with Hangul goes back to QWERTY keys, anything else is
composed into Hangul. Options are read only at the start and everything after them is text,
so the \"-ㅁ\" in \"hanfix ㅣㄴ -ㅁ\" is text (= \"-a\"). Put \"--\" before text that starts with \"-\".

options:
  -h, --help          show this help and exit
  -V, -v, --version   show the version and exit
  --update            install the latest release over this copy (re-runs the installer)
  --to {{ko,en}}        force the direction: ko = keys to Hangul, en = Hangul to keys
  --init SHELL        print the shell function that fixes and re-runs the previous
                      command; SHELL is one of: {}
  --alias NAME        name of that function (default: hf); its Korean-layout twin
                      (hf -> ㅗㄹ) is defined too

shell setup (add one line to your shell's start-up file):
  bash        eval \"$(hanfix --init bash)\"                            ~/.bashrc
  zsh         eval \"$(hanfix --init zsh)\"                             ~/.zshrc
  fish        hanfix --init fish | source                             ~/.config/fish/config.fish
  PowerShell  Invoke-Expression (hanfix --init powershell | Out-String)   $PROFILE

  then, after a mistyped command, type hf (or ㅗㄹ); hf -y runs it without asking.

exit status: 0 on success, 2 on a usage error.
",
        SHELLS.join(", ")
    )
}

fn usage_error(msg: &str) -> ExitCode {
    eprint!("{USAGE}hanfix: error: {msg}\n");
    ExitCode::from(2)
}

// From the latest release rather than raw.githubusercontent.com, whose CDN can serve a stale copy for minutes.
const INSTALLER: &str = "https://github.com/NaNIsaac/hanfix/releases/latest/download/install";

/// Re-run the release installer into the folder this binary lives in. The installer checks the
/// SHA-256 and, on Windows, renames the running hanfix.exe out of the way instead of overwriting it.
fn update() -> ExitCode {
    let dir = match std::env::current_exe().map(|p| p.parent().map(|d| d.to_path_buf())) {
        Ok(Some(d)) => d,
        _ => {
            eprintln!("hanfix: error: cannot find where hanfix is installed");
            return ExitCode::FAILURE;
        }
    };
    println!("hanfix: updating {} in {}", env!("CARGO_PKG_VERSION"), dir.display());
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("powershell");
        c.env_remove("PSModulePath"); // inherited from PowerShell 7, it breaks Windows PowerShell's modules
        c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &format!("irm {INSTALLER}.ps1 | iex")]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", &format!(
            "set -e; if command -v curl >/dev/null 2>&1; then s=$(curl -fsSL {INSTALLER}.sh); else s=$(wget -qO- {INSTALLER}.sh); fi; printf '%s\n' \"$s\" | sh"
        )]);
        c
    };
    match cmd.env("HANFIX_INSTALL_DIR", &dir).env("HANFIX_UPDATE", "1").status() {
        Ok(st) if st.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("hanfix: error: cannot start the installer: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(mut args: Vec<String>) -> ExitCode {
    let (mut to, mut shell, mut name) = (None, None, "hf".to_string());
    // Leading options only: the text itself may start with "-" ("-ㅁ" is a mistyped "-a").
    while let Some(a) = args.first().cloned() {
        if a == "--" {
            args.remove(0);
            break;
        }
        match a.as_str() {
            "-h" | "--help" => {
                print!("{}", help());
                return ExitCode::SUCCESS;
            }
            "-V" | "-v" | "--version" => {
                println!("hanfix {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--update" => return update(),
            _ => {}
        }
        let (opt, val) = match a.split_once('=') {
            Some((o, v)) => (o.to_string(), Some(v.to_string())),
            None => (a.clone(), None),
        };
        if !["--to", "--init", "--alias"].contains(&opt.as_str()) {
            if a.starts_with('-') && a.len() > 1 && a.is_ascii() {
                return usage_error(&format!("unknown option '{a}' (put -- before text that starts with -)"));
            }
            break;
        }
        args.remove(0);
        let val = match val {
            Some(v) => v,
            None if !args.is_empty() => args.remove(0),
            None => return usage_error(&format!("{opt} needs a value")),
        };
        match opt.as_str() {
            "--to" => match val.as_str() {
                "ko" => to = Some(Dir::Ko),
                "en" => to = Some(Dir::En),
                _ => return usage_error(&format!("--to: choose ko or en, not '{val}'")),
            },
            "--init" => {
                if !SHELLS.contains(&val.as_str()) {
                    return usage_error(&format!("--init: choose one of {}, not '{val}'", SHELLS.join(", ")));
                }
                shell = Some(val);
            }
            _ => {
                let ok = val.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                    && val.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                if !ok {
                    return usage_error(&format!("--alias: use letters, digits and _, not '{val}'"));
                }
                name = val;
            }
        }
    }

    let out = if let Some(sh) = shell {
        init_script(&sh, &name)
    } else if !args.is_empty() {
        fix(&args.join(" "), to) + "\n"
    } else if io::stdin().is_terminal() {
        return usage_error("no text given (pass it as arguments or on standard input)");
    } else {
        let mut buf = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut buf) {
            eprintln!("hanfix: error: cannot read standard input: {e}");
            return ExitCode::FAILURE;
        }
        let text = String::from_utf8_lossy(&buf);
        fix(text.trim_end_matches(['\n', '\r']), to) + "\n"
    };
    // A closed pipe (hanfix ... | head) is not an error worth a panic.
    let _ = io::stdout().write_all(out.as_bytes());
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    run(std::env::args().skip(1).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASES: [(&str, &str); 18] = [
        ("dkssud", "안녕"), ("dkssudgktpdy", "안녕하세요"), ("rkr", "각"), ("rkrk", "가가"),
        ("rhk", "과"), ("dkfr", "앍"), ("dkfrl", "알기"), ("dkfrdl", "앍이"), ("qkqhrk", "바보가"),
        ("dmlwk", "의자"), ("rkqt", "값"), ("rkqtdl", "값이"), ("Tkd", "쌍"), ("ekfr", "닭"),
        ("gksrmf", "한글"), ("dkssud, gktpdy!", "안녕, 하세요!"), ("rr", "ㄱㄱ"), ("k", "ㅏ"),
    ];

    #[test]
    fn composes() {
        for (keys, ko) in CASES {
            assert_eq!(compose(keys), ko, "{keys}");
        }
    }

    #[test]
    fn round_trips() {
        for (_, ko) in CASES {
            assert_eq!(compose(&decompose(ko)), ko);
        }
        // every syllable block, not only the cases above
        for n in BASE..=0xD7A3 {
            let s = char::from_u32(n).unwrap().to_string();
            assert_eq!(compose(&decompose(&s)), s);
        }
    }

    #[test]
    fn commands() {
        assert_eq!(fix("ㅣㄴ -ㅁ", None), "ls -a");
        assert_eq!(fix("햣 ㄴㅅㅁ션", None), "git status");
        assert_eq!(fix("ㅊㅇ ..", None), "cd ..");
        assert_eq!(fix("dkssud", None), "안녕");
        assert_eq!(fix("ls", Some(Dir::Ko)), "ㅣㄴ");
    }

    #[test]
    fn init_scripts() {
        assert!(init_script("bash", "hf").contains("alias ㅗㄹ=hf"));
        assert!(init_script("zsh", "fk").contains("alias 라=fk"));
        let ps = init_script("powershell", "hf");
        assert!(ps.is_ascii() && ps.contains("[char]0x3157 + [string][char]0x3139"));
    }
}
